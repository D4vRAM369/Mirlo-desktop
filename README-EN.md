<p align="center">
  <img src="assets/banner-en.png" width="900" alt="Mirlo: your local vault for X posts. Icon of a blue card-catalog drawer holding a dark-mode post with a blackbird perched on top.">
</p>

<p align="center"><a href="README.md">Español</a> · <b>English</b></p>

A local vault for X (Twitter) posts. Paste a post link and Mirlo saves the text, author, date and every file to your own disk: photos at original resolution, videos, GIFs (as MP4) and linked documents (PDF, DOCX…).

You can also [send it from X's Share menu](#send-from-x), without switching windows.

No account, no cloud, no telemetry.

> The interface is available in English and Spanish. Mirlo follows your system language the first time; switch it anytime with the button next to the theme toggle.

## How to use it

1. Copy the link of a post on X.
2. Press <kbd>Ctrl</kbd>+<kbd>V</kbd> anywhere in the Mirlo window.
3. The post appears in the list. If something could not be downloaded, it is marked **◐ Partial** or **✕ Error**, and you can retry it with <kbd>R</kbd>.

| Shortcut | Action |
|---|---|
| <kbd>J</kbd> / <kbd>K</kbd> | Next and previous |
| <kbd>/</kbd> | Search (full text, accent-insensitive) |
| <kbd>Ctrl</kbd>+<kbd>V</kbd> | Save the copied link |
| <kbd>O</kbd> | Open the post's folder |
| <kbd>R</kbd> | Retry a partial or failed post |
| <kbd>F</kbd> | Mark or unmark as favorite |
| <kbd>T</kbd> / <kbd>M</kbd> / <kbd>N</kbd> | The post's tags, collection and note |

## Send from X

Save a post with **Share → Send to Mirlo**, whether Mirlo is open, hidden or closed.

<p align="center">
  <img src="assets/share-menu-en.png" width="290" alt="X's Share menu with three options: Send via Chat, Copy link and Send to Mirlo.">
</p>

With X in Spanish the button says "Enviar a Mirlo"; in any other language, "Send to Mirlo".

In versions after 0.2.0, Mirlo guides you: it offers this the first time, and it is always under **Integrate with X** in the top bar.

1. Pick your browser and install the userscript manager it points to (an extension that adds small programs to websites): Violentmonkey on Firefox or Tampermonkey on Chrome, Edge, Brave and Chromium.
2. On Chromium browsers only: turn on "Allow User Scripts" in the extension's details.
3. Click **Install script** and confirm it in the browser. The script ships inside Mirlo; you can copy it too.
4. Click **Check** and then, on X, **Connect to Mirlo**. When Mirlo receives the greeting it shows "✓ Connected to X".

With the AppImage, the guide first asks permission to register `mirlo://` links with the **Allow Mirlo links** button. If you move the file to another folder, click it again.

<details>
<summary>Install it by hand (or with 0.2.0, which has no guide)</summary>

1. Install Mirlo with an [installer](https://github.com/D4vRAM369/Mirlo-desktop/releases/latest): the 0.2.0 AppImage does not register `mirlo://` links.
2. Install [Violentmonkey](https://violentmonkey.github.io/) on Firefox or [Tampermonkey](https://www.tampermonkey.net/) on Chromium, with "Allow User Scripts" turned on.
3. Open **[mirlo-x.user.js](https://github.com/D4vRAM369/Mirlo-desktop/raw/master/userscript/mirlo-x.user.js)** and confirm the installation.
4. Reload X and use **Share → Send to Mirlo**.

</details>

The first time, the browser asks whether it may open Mirlo. When a send works, Mirlo saves without stealing focus. When it fails, it shows its window with the error.

## Organizing

- **Favorites** (★) for what you want close at hand.
- **Collections**, like folders: each post goes in one or none. Create them with **+** in the sidebar, rename or delete them with a double click (or <kbd>F2</kbd>) and reorder them with <kbd>Alt</kbd>+<kbd>↑</kbd>/<kbd>↓</kbd>.
- **Tags**, as many as you like per post, with a palette color or any other from the color wheel.
- Personal **notes** on each post.

Notes and tags are searchable. Saving or retrying a post again never erases what you added: it lives in the database and in the `user` block of its `post.json`.

## Backups

Under **Backup** (top bar):

- **Export vault…** creates a `.zip` with everything. Untick *Include photos, videos and documents* for a light copy with text only.
- **Import backup…** brings it back, on this computer or another. **It merges and never overwrites:** posts you already have do not change, tags and favorites are added, and your notes and collections stay. Every file is checked against its SHA-256 fingerprint, and a backup with suspicious paths is rejected as a whole.

If the backup was light, imported posts are marked partial: <kbd>R</kbd> downloads what is missing.

## Where it is saved

```
~/Mirlo/
├── mirlo.db                          index for search and filters
└── <user>/<YYYY-MM-DD>_<id>/
    ├── post.json                     post, files with their SHA-256 and the original response
    ├── post.md                       the post as readable text
    ├── media/01.jpg, 02.mp4…         in the post's order (q01… = quoted post)
    └── attachments/                  linked documents
```

The folders are the real copy: you can copy them or read them without Mirlo.

## What leaves your computer

Mirlo only makes these requests, and only when you save or retry a post:

- **X** (`cdn.syndication.twimg.com`, `publish.x.com`, `pbs.twimg.com`, `video.twimg.com`): the post and its files.
- **yt-dlp** (bundled with the app): asks X for the post when the route above fails.
- **FxTwitter** (`api.fxtwitter.com`): **only** for posts longer than 280 characters, because X delivers them cut short. It receives the post's number.
- **Sites linked in the post**: Mirlo checks whether a link is a document (by its `Content-Type`) and, if it is and weighs less than 50 MB, downloads it. It never touches addresses on your local network.

The userscript makes no requests and reads no cookies: it only hands Mirlo the post's number or, when checking the connection, the single-use code Mirlo created. Any website could try to open a `mirlo://` link, which is why the browser asks first and Mirlo accepts only those two forms, the greeting only with its code. If you choose "always allow", do it only for x.com.

The guide only acts when you click its buttons: it opens the extension store or X in your browser, and for **Install script** it listens for up to 10 minutes on an address on your own computer (`127.0.0.1`) that serves nothing but that script.

The font ships inside the app: opening Mirlo contacts no one.

## Limitations

- It does not save full threads, or protected posts that require signing in.
- "Send to Mirlo" depends on X's page and may disappear when X changes it. It was tested on Linux with X signed out: end to end on Firefox with Violentmonkey; on Chromium, Tampermonkey's install screen and the check with the script loaded by hand. Signed-in X, Tampermonkey with its permission turned on, Windows and macOS are still untested.
- With the AppImage in a path with spaces, some desktops other than GNOME and KDE do not open `mirlo://` links: move it to a path without spaces.
- The check needs Mirlo running: if you quit it, click **Check** again.
- If FxTwitter does not respond, a long post stays partial (text cut short by X).

## Development

Requirements: Rust, Node 20+ and the system [dependencies for Tauri 2](https://tauri.app/start/prerequisites/).

```bash
npm install
npm run sidecar        # downloads yt-dlp for your platform and checks its SHA-256
npm run tauri dev      # quit the installed Mirlo first: only one instance can run
```

```bash
npm run check                                   # types, translations and userscript
cd src-tauri && cargo test                      # tests against saved real responses
cd src-tauri && cargo test -- --ignored         # live tests against X
cd src-tauri && cargo clippy --all-targets -- -D warnings
npm run tauri build                             # .deb, .rpm and AppImage packages
```

Changes per version: [`CHANGELOG.md`](CHANGELOG.md).

## License

[GPL-3.0 or later](LICENSE). You may use, study, change and share Mirlo; if you distribute a modified version, its source code must also be free under this same license.
