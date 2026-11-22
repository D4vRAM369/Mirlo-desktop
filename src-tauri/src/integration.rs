//! Integrar Mirlo con X (SPEC, parte E): lleva el userscript dentro, lo entrega al navegador, mira
//! quién abre los enlaces mirlo:// y reconoce el saludo del script. Nada se instala por su cuenta.

use serde::Serialize;
use std::hash::{BuildHasher, Hasher, RandomState};
use std::io::{Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, PoisonError};
use std::time::{Duration, Instant};

/// El mismo archivo de `userscript/`, metido en el binario al compilar: no hay una segunda copia.
pub const SCRIPT: &str = include_str!("../../userscript/mirlo-x.user.js");
/// Los gestores de userscripts reconocen una instalación por el final `.user.js` de la dirección.
const PATH: &str = "/mirlo-x.user.js";

/// El saludo: Mirlo inventa un número, lo pone en la dirección de X y solo acepta ese, una vez.
/// Así una web cualquiera no puede hacer que Mirlo diga "Conectado" abriendo mirlo://hello/…
#[derive(Default)]
pub struct Hello {
    pending: Mutex<Option<String>>,
    connected: AtomicBool,
}

impl Hello {
    /// Repetir Comprobar devuelve el mismo código: la primera pestaña sigue valiendo.
    pub fn begin(&self) -> String {
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        // RandomState se siembra con el azar del sistema; basta para un número que vive en memoria
        pending.get_or_insert_with(|| RandomState::new().build_hasher().finish().to_string()).clone()
    }

    pub fn accept(&self, code: &str) -> bool {
        let mut pending = self.pending.lock().unwrap_or_else(PoisonError::into_inner);
        if pending.as_deref() != Some(code) {
            return false;
        }
        *pending = None;
        self.connected.store(true, Ordering::SeqCst);
        true
    }

    /// Para la interfaz que no estaba escuchando cuando llegó el saludo.
    pub fn take_connected(&self) -> bool {
        self.connected.swap(false, Ordering::SeqCst)
    }
}

/// Dirección local y temporal desde la que el navegador recoge el script.
#[derive(Default)]
pub struct Delivery(Mutex<Option<(String, Instant)>>);

impl Delivery {
    pub fn url(&self, ttl: Duration) -> std::io::Result<String> {
        let mut current = self.0.lock().unwrap_or_else(PoisonError::into_inner);
        if let Some((url, until)) = current.as_ref() {
            // con menos de un minuto por delante no da tiempo a instalar: se abre otra
            if *until > Instant::now() + Duration::from_secs(60) {
                return Ok(url.clone());
            }
        }
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        listener.set_nonblocking(true)?;
        let url = format!("http://{}{PATH}", listener.local_addr()?);
        let until = Instant::now() + ttl;
        std::thread::Builder::new().spawn(move || {
            while Instant::now() < until {
                match listener.accept() {
                    // ponytail: un hilo por conexión, sin tope; cada uno muere a los 2 segundos
                    Ok((stream, _)) => drop(std::thread::Builder::new().spawn(move || answer(stream))),
                    Err(_) => std::thread::sleep(Duration::from_millis(100)),
                }
            }
        })?;
        *current = Some((url.clone(), until));
        Ok(url)
    }
}

/// No lee el disco: o es exactamente `GET /mirlo-x.user.js` y va la constante, o es un 404.
fn answer(mut stream: TcpStream) -> std::io::Result<()> {
    stream.set_nonblocking(false)?;
    stream.set_write_timeout(Some(Duration::from_secs(2)))?;
    let until = Instant::now() + Duration::from_secs(2);
    let (mut head, mut chunk) = (Vec::new(), [0u8; 1024]);
    // la cabecera se lee entera: cerrar con datos sin leer corta la respuesta
    while !head.windows(4).any(|w| w == b"\r\n\r\n") {
        let left = until.saturating_duration_since(Instant::now());
        if left.is_zero() || head.len() > 16 * 1024 {
            return Ok(());
        }
        stream.set_read_timeout(Some(left))?;
        match stream.read(&mut chunk)? {
            0 => return Ok(()),
            n => head.extend_from_slice(&chunk[..n]),
        }
    }
    let wanted = head.starts_with(format!("GET {PATH} HTTP/1.").as_bytes());
    let (status, body) = if wanted { ("200 OK", SCRIPT) } else { ("404 Not Found", "") };
    write!(
        stream,
        "HTTP/1.1 {status}\r\nContent-Type: text/javascript; charset=utf-8\r\nContent-Length: {}\r\n\
         Cache-Control: no-store\r\nX-Content-Type-Options: nosniff\r\nConnection: close\r\n\r\n{body}",
        body.len()
    )
}

/// Qué copia abre los enlaces mirlo://
#[derive(Debug, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
#[cfg_attr(not(target_os = "linux"), allow(dead_code))] // cada sistema sabe responder una parte
pub enum Owner {
    Here,
    Other,
    Nobody,
    Unknown,
}

/// En Linux no vale `is_registered()` del plugin: solo compara el nombre del .desktop, así que
/// falla con el deb (se llama Mirlo.desktop) y acierta por error con otra copia del mismo nombre.
#[cfg(target_os = "linux")]
pub mod linux {
    use super::Owner;
    use std::path::{Path, PathBuf};
    use std::process::Command;

    /// (archivo .AppImage, carpeta donde está montada). Se exige que este binario viva dentro:
    /// las variables también se heredan al abrir Mirlo desde otra aplicación AppImage.
    pub fn appimage() -> Option<(PathBuf, PathBuf)> {
        let file = PathBuf::from(std::env::var_os("APPIMAGE")?);
        let dir = PathBuf::from(std::env::var_os("APPDIR")?);
        let exe = std::env::current_exe().ok()?;
        exe.starts_with(dir.canonicalize().ok()?).then_some((file, dir))
    }

    /// `None` = la variable entera era de la AppImage.
    pub fn without_appdir(value: &str, appdir: &Path) -> Option<String> {
        if !value.split(':').any(|part| Path::new(part).starts_with(appdir)) {
            return Some(value.to_string());
        }
        let kept: Vec<&str> =
            value.split(':').filter(|part| !part.is_empty() && !Path::new(part).starts_with(appdir)).collect();
        (!kept.is_empty()).then(|| kept.join(":"))
    }

    /// Un programa del sistema no debe heredar las librerías ni los .desktop de dentro de la AppImage.
    pub fn system_command(program: &str) -> Command {
        let mut command = Command::new(program);
        let Some((_, dir)) = appimage() else { return command };
        for (key, value) in std::env::vars_os() {
            let Some(value) = value.to_str() else { continue };
            match without_appdir(value, &dir) {
                None => command.env_remove(&key),
                Some(clean) if clean != value => command.env(&key, clean),
                Some(_) => &mut command,
            };
        }
        for key in ["APPIMAGE", "APPDIR", "ARGV0", "OWD"] {
            command.env_remove(key);
        }
        command
    }

    /// El programa de la línea `Exec` de `[Desktop Entry]`, sin comillas ni argumentos.
    pub fn exec_program(desktop: &str) -> Option<String> {
        let line = desktop
            .lines()
            .map(str::trim)
            .skip_while(|line| *line != "[Desktop Entry]")
            .skip(1)
            .take_while(|line| !line.starts_with('['))
            .filter_map(|line| line.split_once('='))
            .find(|(key, _)| key.trim_end() == "Exec")?
            .1
            .trim_start()
            .replace("\\\\", "\\"); // primer nivel de escapes: el de los archivos .desktop
        let program = match line.strip_prefix('"') {
            None => line.split_whitespace().next()?.to_string(),
            Some(quoted) => {
                let (mut out, mut chars) = (String::new(), quoted.chars());
                loop {
                    match chars.next()? {
                        '"' => break out,
                        '\\' => out.push(chars.next()?),
                        c => out.push(c),
                    }
                }
            }
        };
        (!program.is_empty()).then_some(program)
    }

    /// `id`: lo que responde xdg-mime. Se compara el programa al que lleva, no el nombre del archivo.
    pub fn owner(id: &str, data_dirs: &[PathBuf], path: &[PathBuf], me: &Path) -> Owner {
        if id.is_empty() {
            return Owner::Nobody;
        }
        // "vendor-app.desktop" también puede estar en vendor/app.desktop
        let names = [id.to_string(), id.replacen('-', "/", 1)];
        let Some(entry) = data_dirs
            .iter()
            .flat_map(|dir| names.iter().map(move |name| dir.join("applications").join(name)))
            .find(|file| file.is_file())
        else {
            return Owner::Nobody;
        };
        let Ok(text) = std::fs::read_to_string(entry) else { return Owner::Unknown };
        let Some(program) = exec_program(&text) else { return Owner::Unknown };
        let found = if program.contains('/') {
            Some(PathBuf::from(&program))
        } else {
            path.iter().map(|dir| dir.join(&program)).find(|file| file.is_file())
        };
        let Some(found) = found.and_then(|file| file.canonicalize().ok()) else { return Owner::Nobody };
        match me.canonicalize() {
            Ok(me) if me == found => Owner::Here,
            Ok(_) => Owner::Other,
            Err(_) => Owner::Unknown,
        }
    }

    /// `register` del plugin escribe `Exec="ruta" %u`, y el xdg-open de los escritorios sin GNOME
    /// ni KDE busca un programa con las comillas en el nombre: no abre nada.
    /// ponytail: una ruta con espacios necesita las comillas y ahí seguirá fallando en esos escritorios.
    pub fn plain_exec(desktop: &str, program: &str) -> String {
        if !program.chars().all(|c| c.is_alphanumeric() || "/._-+@".contains(c)) {
            return desktop.to_string();
        }
        desktop.replace(&format!("Exec=\"{program}\" %u"), &format!("Exec={program} %u"))
    }

    pub fn links(data_home: &Path) -> Owner {
        let bundle = appimage();
        let appdir = bundle.as_ref().map(|(_, dir)| dir.as_path());
        let Some(me) = bundle.as_ref().map(|(file, _)| file.clone()).or_else(|| std::env::current_exe().ok()) else {
            return Owner::Unknown;
        };
        let list = |key: &str, default: &str| -> Vec<PathBuf> {
            let value = std::env::var(key).unwrap_or_default();
            let value = match appdir {
                Some(dir) => without_appdir(&value, dir).unwrap_or_default(),
                None => value,
            };
            let value = if value.is_empty() { default } else { &value };
            std::env::split_paths(value).collect()
        };
        let mut data_dirs = vec![data_home.to_path_buf()];
        data_dirs.extend(list("XDG_DATA_DIRS", "/usr/local/share:/usr/share"));

        let Ok(out) = system_command("xdg-mime").args(["query", "default", "x-scheme-handler/mirlo"]).output() else {
            return Owner::Unknown;
        };
        // xdg-utils sale con 4 cuando no hay ninguna aplicación
        if !out.status.success() && out.status.code() != Some(4) {
            return Owner::Unknown;
        }
        let id = String::from_utf8_lossy(&out.stdout);
        owner(id.lines().next().unwrap_or_default().trim(), &data_dirs, &list("PATH", "/usr/bin:/bin"), &me)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};
    use std::net::TcpStream;
    use std::time::Instant;

    #[test]
    fn script_is_the_file_in_userscript() {
        let file = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../userscript/mirlo-x.user.js");
        assert_eq!(SCRIPT.as_bytes(), std::fs::read(file).unwrap());
    }

    #[test]
    fn hello_needs_the_pending_code_and_works_once() {
        let hello = Hello::default();
        assert!(!hello.accept("123"), "nadie ha pulsado Comprobar");
        let code = hello.begin();
        assert!(!code.is_empty() && code.len() <= 20 && code.bytes().all(|b| b.is_ascii_digit()), "{code}");
        assert_eq!(hello.begin(), code, "pulsar dos veces no invalida la primera pestaña");
        assert!(!hello.accept("123"));
        assert!(!hello.take_connected());
        assert!(hello.accept(&code));
        assert!(!hello.accept(&code), "un solo uso");
        assert!(hello.take_connected());
        assert!(!hello.take_connected(), "se recoge una vez");
        assert_ne!(hello.begin(), code, "la siguiente comprobación usa otro código");
    }

    fn request(url: &str, head: &str) -> String {
        let address = url.trim_start_matches("http://").split('/').next().unwrap();
        let mut stream = TcpStream::connect(address).unwrap();
        stream.write_all(head.as_bytes()).unwrap();
        let mut answer = String::new();
        stream.read_to_string(&mut answer).unwrap();
        answer
    }

    #[test]
    fn delivery_serves_only_the_script_on_loopback() {
        let delivery = Delivery::default();
        let url = delivery.url(Duration::from_secs(120)).unwrap();
        assert!(url.starts_with("http://127.0.0.1:") && url.ends_with("/mirlo-x.user.js"), "{url}");
        assert_eq!(delivery.url(Duration::from_secs(120)).unwrap(), url, "otro clic no abre otro puerto");

        let ok = request(&url, "GET /mirlo-x.user.js HTTP/1.1\r\nHost: 127.0.0.1\r\n\r\n");
        assert!(ok.starts_with("HTTP/1.1 200 "), "{ok}");
        assert_eq!(ok.split_once("\r\n\r\n").unwrap().1, SCRIPT);

        for line in [
            "GET / HTTP/1.1",
            "GET /../../etc/passwd HTTP/1.1",
            "GET /mirlo-x.user.js/../mirlo.db HTTP/1.1",
            "POST /mirlo-x.user.js HTTP/1.1",
            "GET /mirlo-x.user.jsx HTTP/1.1",
        ] {
            let no = request(&url, &format!("{line}\r\n\r\n"));
            assert!(no.starts_with("HTTP/1.1 404 "), "{line}");
            assert!(!no.contains("==UserScript=="), "{line}");
        }
    }

    #[test]
    fn a_silent_connection_does_not_block_the_next_one() {
        let url = Delivery::default().url(Duration::from_secs(120)).unwrap();
        let address = url.trim_start_matches("http://").split('/').next().unwrap();
        let _silent = TcpStream::connect(address).unwrap();
        let start = Instant::now();
        assert!(request(&url, "GET /mirlo-x.user.js HTTP/1.1\r\n\r\n").starts_with("HTTP/1.1 200 "));
        assert!(start.elapsed() < Duration::from_secs(1));
    }

    #[test]
    fn delivery_expires() {
        let url = Delivery::default().url(Duration::from_millis(200)).unwrap();
        let address = url.trim_start_matches("http://").split('/').next().unwrap();
        std::thread::sleep(Duration::from_millis(800));
        assert!(TcpStream::connect(address).is_err());
    }

    #[cfg(target_os = "linux")]
    mod linux {
        use super::super::{linux::*, Owner};
        use std::path::{Path, PathBuf};

        #[test]
        fn appimage_paths_are_removed_from_the_environment() {
            let dir = Path::new("/tmp/.mount_MirAbc");
            let clean = |v| without_appdir(v, dir);
            assert_eq!(clean("/tmp/.mount_MirAbc/usr/share/:/usr/share").as_deref(), Some("/usr/share"));
            assert_eq!(clean("/tmp/.mount_MirAbc/usr/share/:"), None);
            assert_eq!(clean("/tmp/.mount_MirAbc"), None);
            assert_eq!(clean("/tmp/.mount_MirAbcd/x:/usr/bin").as_deref(), Some("/tmp/.mount_MirAbcd/x:/usr/bin"));
            assert_eq!(clean("es_ES.UTF-8").as_deref(), Some("es_ES.UTF-8"));
            assert_eq!(clean("").as_deref(), Some(""));
        }

        #[test]
        fn exec_program_reads_the_main_entry_only() {
            for (desktop, program) in [
                ("[Desktop Entry]\nExec=mirlo %u\n", Some("mirlo")),
                ("[Desktop Entry]\nExec = /opt/Mirlo.AppImage %u\n", Some("/opt/Mirlo.AppImage")),
                ("[Desktop Entry]\nExec=\"/home/u/mis apps/Mirlo.AppImage\" %u\n", Some("/home/u/mis apps/Mirlo.AppImage")),
                ("[Desktop Entry]\nExec=\"/a/\\\\$b\" %u\n", Some("/a/$b")),
                ("[Desktop Action x]\nExec=otro\n[Desktop Entry]\nExec=mirlo\n", Some("mirlo")),
                ("[Desktop Entry]\nTryExec=otro\n", None),
                ("[Desktop Entry]\nExec=\n", None),
                ("[Desktop Entry]\nExec=\"sin cerrar\n", None),
            ] {
                assert_eq!(exec_program(desktop).as_deref(), program, "{desktop}");
            }
        }

        #[test]
        fn owner_says_which_copy_opens_the_links() {
            let root = std::env::temp_dir().join(format!("mirlo-owner-{}", std::process::id()));
            let _ = std::fs::remove_dir_all(&root);
            let write = |rel: &str, body: &str| {
                let file = root.join(rel);
                std::fs::create_dir_all(file.parent().unwrap()).unwrap();
                std::fs::write(&file, body).unwrap();
                file
            };
            let me = write("mis apps/Mirlo.AppImage", "");
            let installed = write("bin/mirlo", "");
            let quoted = format!("[Desktop Entry]\nExec=\"{}\" %u\n", me.display());
            write("user/applications/mirlo-handler.desktop", &quoted);
            write("system/applications/Mirlo.desktop", "[Desktop Entry]\nExec=mirlo %u\n");
            write("system/applications/moved.desktop", "[Desktop Entry]\nExec=/no/existe/Mirlo.AppImage %u\n");
            write("system/applications/vendor/app.desktop", "[Desktop Entry]\nExec=mirlo %u\n");
            let data: Vec<PathBuf> = vec![root.join("user"), root.join("system")];
            let path = vec![root.join("bin")];
            let check = |id, me: &Path| owner(id, &data, &path, me);

            assert_eq!(check("mirlo-handler.desktop", &me), Owner::Here);
            assert_eq!(check("Mirlo.desktop", &installed), Owner::Here, "el deb no se llama mirlo-handler");
            assert_eq!(check("Mirlo.desktop", &me), Owner::Other);
            assert_eq!(check("vendor-app.desktop", &installed), Owner::Here);
            assert_eq!(check("moved.desktop", &me), Owner::Nobody, "una copia borrada no abre nada");
            assert_eq!(check("no-existe.desktop", &me), Owner::Nobody);
            assert_eq!(check("", &me), Owner::Nobody);
            std::fs::remove_dir_all(&root).unwrap();
        }

        #[test]
        fn exec_loses_quotes_only_when_they_are_not_needed() {
            let file = |path| format!("[Desktop Entry]\nExec=\"{path}\" %u\nTerminal=false\n");
            let safe = "/home/u/Apps/Mirlo_0.2.0-1.AppImage";
            assert_eq!(plain_exec(&file(safe), safe), format!("[Desktop Entry]\nExec={safe} %u\nTerminal=false\n"));
            for path in ["/home/u/mis apps/M.AppImage", "/home/u/a$b/M", "/home/u/a;b/M", "/home/u/a%b/M"] {
                assert_eq!(plain_exec(&file(path), path), file(path), "{path}");
            }
        }
    }
}
