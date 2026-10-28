# Mirlo como clase de DAM

> Guía para recorrer el proyecto desde cero, como una unidad didáctica de Desarrollo de Aplicaciones Multiplataforma (DAM) o Web (DAW).
> A diferencia de `../chronicler/03_PASO_A_PASO.md`, que es técnico y reproducible, aquí se explica **por qué importa cada pieza**.

## A quién va dirigida

A alguien que sabe programar un poco (variables, funciones, bucles, en Python o Java) y nunca ha hecho una app de escritorio. No hace falta saber Rust, Svelte ni Tauri: se presentan al llegar.

## La idea de la unidad

Una app de escritorio real que resuelve un problema real: guardar posts de X en tu disco para que no desaparezcan. Cada lección es una pieza de esa app, y cada pieza enseña una idea que sirve para cualquier otro proyecto.

```
Usuario pega un enlace
   → Svelte (la pantalla)  → Tauri (el puente)  → Rust (el motor)
                                                   ├── red: pedir el post a X
                                                   ├── disco: guardar archivos
                                                   └── SQLite: indexar para buscar
```

**Analogía de toda la unidad: un restaurante.** Svelte es la sala (lo que ve el cliente). Rust es la cocina (donde se trabaja con fuego y cuchillos). Tauri es la ventanilla por la que pasan los pedidos. La sala nunca entra en la cocina: si el cliente pudiera coger un cuchillo, alguien acabaría cortándose. Por eso **la red y el disco solo se tocan desde Rust**.

---

## Lección 1: leer datos sin inventarse nada

**Objetivo:** extraer de un post solo su contenido, o dar un error claro.
**Por qué importa:** el proyecto anterior (ThreadsVault) buscaba el texto con regex dentro de la página web y a veces cogía el de una respuesta. Un programa que "casi siempre acierta" es peor que uno que dice "no puedo": el primero te engaña.
**Lo que se aprende:** JSON frente a HTML, funciones puras, `Result` en Rust, TDD con fixtures.
**Para practicar:** abre `tests/fixtures/photo_1.json` y localiza a mano el texto, la foto y el enlace. Después lee `parser::clean_text` y comprueba que hace lo mismo que tú.

## Lección 2: guardar archivos sin dejar ninguno roto

**Objetivo:** descargar fotos, vídeos y PDF de forma fiable.
**Por qué importa:** si la descarga se corta a la mitad y el archivo ya tiene su nombre final, parece bueno y no lo es. Lo descubrirías meses después.
**Lo que se aprende:** escritura atómica (`.part` + renombrar), SHA-256, reintentos solo cuando tiene sentido, Content-Type, y no dejar que un enlace te haga tocar tu red local (SSRF).
**Para practicar:** en `vault.rs`, busca qué errores se reintentan y cuáles no. ¿Por qué un 404 no se reintenta y un 503 sí?

## Lección 3: buscar rápido entre miles de cosas

**Objetivo:** una base de datos que encuentre "canción" escribiendo "cancion".
**Por qué importa:** recorrer todos los posts para buscar funciona con 10 y se arrastra con 10.000. Un índice no.
**Lo que se aprende:** SQLite, el índice invertido de FTS5, triggers, consultas parametrizadas (por qué los datos van separados del SQL), migraciones de esquema.
**Para practicar:** en `db.rs`, lee `fts_query`. Escribe en el buscador de Mirlo `"hola` (con una comilla) y comprueba que no se rompe. ¿Qué lo protege?

## Lección 4: una interfaz que se usa con teclado

**Objetivo:** tres paneles, tema claro y oscuro, todo accesible.
**Por qué importa:** una interfaz bonita que no se puede usar sin ratón, o que solo funciona en un navegador, no está terminada.
**Lo que se aprende:** CSS Grid, variables de diseño (tokens), `<dialog>`, roving tabindex, y que **cada sistema pinta con un motor distinto**: Ctrl+V, los vídeos y el icono fallaron solo en Linux.
**Para practicar:** cambia un color en `src/app.css` (por ejemplo `--accent`) y observa cuántos sitios cambian a la vez. Eso es un token.

## Lección 5: empaquetar y automatizar

**Objetivo:** que otra persona instale Mirlo con un clic.
**Por qué importa:** si una app solo funciona en el ordenador de quien la programó, no es una app: es un experimento.
**Lo que se aprende:** sidecar (un programa dentro de otro), verificar descargas con checksum, conflictos entre paquetes, CI y releases automáticas.
**Para practicar:** abre `.github/workflows/ci.yml` y explica con tus palabras cada paso. ¿Qué pasaría si quitaras `npm run sidecar`?

## Lección 6: identidad y dos idiomas

**Objetivo:** nombre, icono y una interfaz en español e inglés.
**Por qué importa:** el nombre "XVault" podía leerse como contenido para adultos y "TwitterVault" usaba una marca ajena. Y una app que solo habla un idioma deja fuera a la mayoría.
**Lo que se aprende:** uso descriptivo de marcas, legibilidad de iconos por tamaños, i18n, y separar **qué** pasó (Rust) de **cómo** se cuenta (interfaz).
**Para practicar:** añade a `src/lib/i18n.svelte.ts` una clave nueva solo en español y ejecuta `npm run check`. Lee el error de TypeScript: así es como el proyecto se protege de traducciones olvidadas.

---

## Evaluación sugerida

1. Explica con un dibujo el recorrido de un clic en "Guardar post" hasta que el archivo está en disco.
2. Di tres cosas que podrían salir mal al descargar un vídeo y cómo responde Mirlo a cada una.
3. ¿Por qué `post.json` es "la verdad" y `mirlo.db` solo un índice? ¿Qué ganarías y qué perderías si fuera al revés?
