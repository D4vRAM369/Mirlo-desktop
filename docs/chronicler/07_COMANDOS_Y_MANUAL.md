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

Después: servidor estático con CORS sobre `target/test-tmp/live` (puerto 4318, config `vault-files` en `.claude/launch.json`), un `_mock.js` que simula `window.__TAURI_INTERNALS__` y `target/test-tmp/review.html?row=1&theme=dark&lightbox=1`, capturado con:

```bash
chromium --headless=new --window-size=1280,840 --virtual-time-budget=6000 --screenshot=out.png "http://localhost:1420/src-tauri/target/test-tmp/review.html?row=1"
```

Todo vive en `target/` (ignorado por git): es andamio de revisión, no parte de la app.
