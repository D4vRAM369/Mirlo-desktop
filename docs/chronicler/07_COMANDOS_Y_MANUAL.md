# Comandos y manual

## Probar el endpoint de syndication a mano

```bash
ID=20
TOKEN=$(node -e "console.log(((Number('$ID')/1e15)*Math.PI).toString(36).replace(/(0+|\.)/g,''))")
curl -s "https://cdn.syndication.twimg.com/tweet-result?id=$ID&token=$TOKEN"
```

## Tests y calidad

```bash
cd src-tauri
cargo test                              # tests rápidos con fixtures (sin red)
cargo test -- --ignored                 # pruebas en vivo contra X
cargo clippy --all-targets -- -D warnings
cd .. && npm run check && npm run build
```

## Arrancar la app (pantalla de prueba de la fase 1)

```bash
npm run tauri dev
```

## Añadir un fixture nuevo

```bash
ID=1234567890
curl -s "https://cdn.syndication.twimg.com/tweet-result?id=$ID&token=a" | python3 -m json.tool > tests/fixtures/nombre.json
```


## Cómo revisar el diseño sin la ventana de Tauri

El panel del navegador integrado es estrecho, así que se usan capturas headless con datos reales:

```bash
cd src-tauri && cargo test live_save -- --ignored     # guarda 5 posts reales en target/test-tmp/live
```

Después: servidor estático con CORS sobre `target/test-tmp/live` (`python3 -I scripts/review-server.py 4318 src-tauri/target/test-tmp/live`), un `_mock.js` que simula `window.__TAURI_INTERNALS__` y `target/test-tmp/review.html?row=1&theme=dark&lightbox=1`, capturado con:

```bash
chromium --headless=new --window-size=1280,840 --virtual-time-budget=6000 --screenshot=out.png "http://localhost:1420/src-tauri/target/test-tmp/review.html?row=1"
```

Todo vive en `target/` (ignorado por git): es andamio de revisión, no parte de la app.


## CI y releases (GitHub Actions)

- `.github/workflows/ci.yml`: en cada push y PR → `npm run check`, `cargo test`, `cargo clippy -D warnings`. Las pruebas en vivo no corren en CI.
- `.github/workflows/release.yml`: al subir una etiqueta `v*` compila en Linux, Windows y macOS y deja los instaladores en un **borrador** de Release.

```bash
git tag v0.1.0
git push --tags        # lanza release.yml
```

Antes de subir cambios a los workflows:

```bash
actionlint             # revisa .github/workflows/*.yml (instalado con: go install github.com/rhysd/actionlint/cmd/actionlint@v1.7.12)
```


## Quitar datos personales del historial de Git

Solo si es imprescindible (reescribe todos los commits; cualquier clon anterior queda desincronizado).

```bash
uv tool install git-filter-repo==2.47.0
git clone --mirror . /ruta/temporal/copia.git          # copia de seguridad ANTES de nada
# reglas: un texto por línea, "literal:" o "regex:", con ==> para el reemplazo
git filter-repo --force --replace-text reglas.txt
git log --all -S "texto-a-quitar" --oneline             # debe salir vacío
git remote add origin https://github.com/<usuario>/<repo>.git   # filter-repo quita el remoto a propósito
git push --force-with-lease=master:<commit-que-tenía-GitHub> origin master
```

`--force-with-lease` solo sobrescribe si GitHub sigue en el commit que conoces: evita pisar cambios ajenos.
