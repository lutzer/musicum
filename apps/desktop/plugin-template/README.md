# Musicum plugin template

## Build
```
npm install
npm run build
```

## Install
Copy `plugin.json` and `dist/index.js` into a subfolder of the Musicum user plugin dir:

- macOS: `~/Library/Application Support/app.musicum.desktop/plugins/<your-id>/`
- Linux: `~/.local/share/app.musicum.desktop/plugins/<your-id>/`
- Windows: `%APPDATA%/app.musicum.desktop/plugins/<your-id>/`

Relaunch Musicum. Your view appears in the nav.
