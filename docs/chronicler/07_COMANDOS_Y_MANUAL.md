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
