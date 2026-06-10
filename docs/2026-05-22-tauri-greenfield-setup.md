# Musicum Tauri — Greenfield Setup Spec

**Date:** 2026-05-22
**Status:** Reviewed
**Purpose:** Bootstrap guide for a new repo. Only the audio plugin crates and structural processor crates are carried over from the old repo. Everything else is written from scratch.

---

## Processor and SDK Crates

All crates in this repo are written from scratch. The processor SDK and processor implementations live in:

```
libs/musicum-processor-sdk/   # Unified processor trait crate (StreamProcessor, StructuralProcessor,
                               # BaseProcessor, FFI layer, export macro)
libs/musicum-processors/      # Processor implementations
    gain/                     # StreamProcessor — volume gain
    reverb/                   # StreamProcessor — Freeverb-style reverb
    trim/                     # StructuralProcessor — time-trim start/end
```

Each processor crate uses dual `crate-type` so it can be linked natively and compiled to a dynamic library:

```toml
[lib]
crate-type = ["cdylib", "rlib"]   # cdylib = dynamic library, rlib = native linkage
```

---

## Repo Structure

```
musicum-tauri/
├── Cargo.toml                  # Cargo workspace root
├── Cargo.lock
├── package.json                # npm workspace root (frontend)
├── nx.json                     # Nx monorepo config (optional, for build orchestration)
│
├── apps/
│   ├── desktop/                # Tauri application (planned)
│   │   ├── src-tauri/
│   │   │   ├── Cargo.toml
│   │   │   ├── tauri.conf.json
│   │   │   ├── icons/
│   │   │   └── src/
│   │   │       ├── main.rs
│   │   │       ├── state.rs
│   │   │       ├── commands/
│   │   │       │   ├── mod.rs
│   │   │       │   ├── files.rs
│   │   │       │   ├── clips.rs
│   │   │       │   ├── collections.rs
│   │   │       │   ├── presets.rs
│   │   │       │   ├── sync.rs
│   │   │       │   ├── playback.rs
│   │   │       │   └── settings.rs
│   │   │       └── http/
│   │   │           ├── mod.rs
│   │   │           └── routes/
│   │   └── package.json        # frontend dev server integration for Tauri
│   │
│   ├── frontend/               # SvelteKit 5 app (planned)
│   │   ├── package.json
│   │   ├── svelte.config.js
│   │   ├── vite.config.ts
│   │   └── src/
│   │       ├── app.html
│   │       ├── routes/
│   │       ├── lib/
│   │       └── ...
│   │
│   └── cli/                    # Standalone Rust CLI (musicum binary)
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           └── commands/
│               ├── mod.rs
│               ├── sync.rs
│               ├── files.rs
│               ├── clips.rs
│               ├── collections.rs
│               └── presets.rs
│
└── libs/
    ├── musicum-core/           # All business logic
    │   ├── Cargo.toml
    │   └── src/
    │       ├── lib.rs
    │       ├── db/
    │       ├── services/
    │       ├── audio/
    │       ├── processor_loader.rs
    │       ├── edit_registry.rs
    │       └── error.rs
    │
    ├── musicum-processor-sdk/  # Unified processor trait crate
    └── musicum-processors/     # Processor implementations
        ├── gain/
        ├── reverb/
        └── trim/
```

---

## Cargo Workspace

**`Cargo.toml` (workspace root):**

```toml
[workspace]
resolver = "2"
members = [
    "apps/cli",
    "libs/musicum-core",
    "libs/musicum-processor-sdk",
    "libs/musicum-processors/gain",
    "libs/musicum-processors/reverb",
    "libs/musicum-processors/trim",
    # "apps/desktop/src-tauri",   # planned
]

[workspace.dependencies]
serde       = { version = "1",    features = ["derive"] }
serde_json  = "1"
uuid        = { version = "1",    features = ["v4", "serde"] }
tokio       = { version = "1",    features = ["full"] }
sea-orm     = { version = "1",    features = ["sqlx-sqlite", "runtime-tokio-rustls", "macros"] }
thiserror   = "1"
anyhow      = "1"
tracing     = "1"
```

---

## `musicum-core`

### `Cargo.toml`

```toml
[package]
name = "musicum-core"
version = "0.1.0"
edition = "2021"

[dependencies]
# workspace
serde.workspace      = true
serde_json.workspace = true
uuid.workspace       = true
tokio.workspace      = true
sea-orm.workspace    = true
thiserror.workspace  = true
anyhow.workspace     = true
tracing.workspace    = true

# audio
symphonia   = { version = "0.5", features = ["all"] }
cpal        = "0.17"
rtrb        = "0.3"

# utils
slug        = "0.1"
chrono      = { version = "0.4", features = ["serde"] }
walkdir     = "2"

# processor SDK + built-in processors (linked natively)
musicum-processor-sdk    = { path = "../musicum-processor-sdk" }
musicum-processor-gain   = { path = "../musicum-processors/gain" }
musicum-processor-reverb = { path = "../musicum-processors/reverb" }
musicum-processor-trim   = { path = "../musicum-processors/trim" }
```

### Module layout

```
musicum-core/src/
├── lib.rs                  # pub mod declarations, re-exports
├── error.rs                # ServiceError enum (thiserror)
├── processor_loader.rs     # ProcessorRegistry — loads .dylib processors at runtime
├── edit_registry.rs        # EditRegistry, EditRegistryEntry — UI-facing descriptor layer
│
├── db/
│   ├── mod.rs              # connect() → DatabaseConnection, run_create_all()
│   ├── schema.rs           # SCHEMA_VERSION constant
│   └── entities/
│       ├── mod.rs
│       ├── file.rs
│       ├── file_metadata.rs
│       ├── file_attachment.rs
│       ├── clip.rs
│       ├── collection.rs
│       ├── collection_clip.rs
│       ├── preset.rs
│       └── edit.rs         # ProcessorEdit, ProcessorEditList, ProcessorEditType
│
├── services/
│   ├── mod.rs
│   ├── file_service.rs
│   ├── file_metadata_service.rs
│   ├── file_attachment_service.rs
│   ├── clip_service.rs
│   ├── collection_service.rs
│   ├── preset_service.rs
│   ├── export_service.rs   # stub — pending reimplementation
│   └── sync_service.rs
│
└── audio/
    ├── mod.rs              # pub use AudioEngine, AudioPlayer, etc.
    ├── engine.rs           # AudioEngine trait + CpalEngine concrete impl
    ├── player.rs           # AudioPlayer — holds ProcessorRegistry, manages queue + engine
    ├── source.rs           # AudioSource trait (fill_buffer, position_secs, duration_secs, …)
    ├── chain.rs            # ProcessorChain — folds ProcessorEdit list into StreamProcessorNode chain
    ├── node.rs             # StreamProcessorNode — wraps upstream AudioSource + one StreamProcessor
    └── decoder.rs          # SymphoniaSource, decode_file() → AudioInfo
```

---

## Database Schema

SeaORM entities. No migration system — `create_table_from_entity()` on every startup. Schema version bump in `schema.rs` signals a breaking change (drop + recreate all tables in dev).

### `file`

| Column | Type | Notes |
|--------|------|-------|
| id | TEXT (UUID) | PK |
| slug | TEXT | unique |
| name | TEXT | display name (filename without extension) |
| path | TEXT | absolute path to source audio file |
| duration | REAL | seconds |
| sample_rate | INTEGER | |
| channels | INTEGER | |
| mime_type | TEXT | |
| hash | TEXT | SHA-256 of file contents (detect changes) |
| mtime | TEXT (ISO8601) | file modification time |
| size_bytes | INTEGER | file size in bytes |
| created_at | TEXT (ISO8601) | |
| updated_at | TEXT (ISO8601) | |

### `file_metadata`

| Column | Type | Notes |
|--------|------|-------|
| file_id | TEXT (UUID) | PK, FK → file |
| bpm | REAL | nullable |
| key | TEXT | nullable |
| rating | INTEGER | nullable, 1–5 |
| color | TEXT | nullable, hex |
| notes | TEXT | |
| tags | TEXT | comma-separated |

### `file_attachment`

| Column | Type | Notes |
|--------|------|-------|
| id | TEXT (UUID) | PK |
| file_id | TEXT (UUID) | FK → file |
| type | TEXT | "text" \| "image" \| "video" |
| text | TEXT | nullable (text attachments) |
| path | TEXT | nullable (file attachments) |
| mime_type | TEXT | nullable |
| created_at | TEXT (ISO8601) | |
| updated_at | TEXT (ISO8601) | |

### `clip`

| Column | Type | Notes |
|--------|------|-------|
| id | TEXT (UUID) | PK |
| slug | TEXT | unique |
| file_id | TEXT (UUID) | FK → file |
| title | TEXT | |
| processors | TEXT (JSON) | ordered list of `ProcessorEdit` entries (see format below) |
| cached | TEXT | "no_cache" \| "caching" \| "ready" \| "error" |
| cached_path | TEXT | nullable, path to cached MP3 |
| duration | REAL | nullable, duration of cached output |
| notes | TEXT | |
| created_at | TEXT (ISO8601) | |
| updated_at | TEXT (ISO8601) | |

### `collection`

| Column | Type | Notes |
|--------|------|-------|
| id | TEXT (UUID) | PK |
| slug | TEXT | unique |
| title | TEXT | |
| description | TEXT | |
| background_path | TEXT | nullable |
| created_at | TEXT (ISO8601) | |
| updated_at | TEXT (ISO8601) | |

### `collection_clip`

| Column | Type | Notes |
|--------|------|-------|
| id | TEXT (UUID) | PK |
| collection_id | TEXT (UUID) | FK → collection |
| clip_id | TEXT (UUID) | FK → clip |
| position | INTEGER | ordering index |

Unique constraint on `(collection_id, clip_id)`.

### `preset`

| Column | Type | Notes |
|--------|------|-------|
| id | TEXT (UUID) | PK |
| slug | TEXT | unique |
| title | TEXT | |
| description | TEXT | |
| processors | TEXT (JSON) | ordered list of `ProcessorEdit` entries (same format as clip.processors) |
| created_at | TEXT (ISO8601) | |
| updated_at | TEXT (ISO8601) | |

---

## Sidecar File Formats

Sidecars are the source of truth. The DB is a queryable index rebuilt from sidecars.

### Audio file sidecar — `{filename}.musicum.json`

Lives next to the source audio file.

```json
{
  "version": 1,
  "metadata": {
    "bpm": null,
    "key": null,
    "rating": null,
    "color": null,
    "notes": "",
    "tags": ""
  },
  "attachments": [
    {
      "uuid": "550e8400-e29b-41d4-a716-446655440000",
      "type": "image",
      "mime_type": "image/jpeg"
    }
  ],
  "clips": [
    {
      "slug": "recording-clean",
      "title": "Clean",
      "notes": "",
      "processors": []
    },
    {
      "slug": "recording-reverb",
      "title": "With Reverb",
      "notes": "",
      "processors": [
        {
          "uuid": "550e8400-e29b-41d4-a716-446655440001",
          "processor_id": "reverb",
          "enabled": true,
          "kind": "StreamProcessor",
          "params": { "room_size": 0.6, "wet": 0.3 }
        }
      ]
    }
  ]
}
```

### Collection sidecar — `collections/{slug}.musicum.json`

```json
{
  "version": 1,
  "slug": "my-album",
  "title": "My Album",
  "description": "",
  "clips": ["recording-clean", "beat-reverb"]
}
```

`clips` is an ordered array of clip slugs.

### Preset sidecar — `presets/{slug}.musicum-preset.json`

```json
{
  "version": 1,
  "slug": "reverb-master",
  "title": "Reverb Master",
  "description": "",
  "processors": [
    { "uuid": "...", "processor_id": "reverb",    "enabled": true, "kind": "StreamProcessor", "params": { "room_size": 0.8 } },
    { "uuid": "...", "processor_id": "normalize", "enabled": true, "kind": "StreamProcessor", "params": { "target_lufs": -14 } }
  ]
}
```

### Processor entry format (shared by `clip.processors` and `preset.processors`)

Each entry is a `ProcessorEdit` struct serialized to JSON:

```json
{ "uuid": "550e8400-...", "processor_id": "gain",  "enabled": true, "kind": "StreamProcessor",    "params": { "gain": 0.8 } }
{ "uuid": "550e8400-...", "processor_id": "trim",  "enabled": true, "kind": "StructuralProcessor", "params": { "start": 0.2, "end": 0.0 } }
```

Fields:
- `uuid` — unique ID for this entry (allows stable parameter references across edits)
- `processor_id` — registered processor name (matched against `ProcessorRegistry`)
- `enabled` — whether the processor is active
- `kind` — `"StreamProcessor"` | `"StructuralProcessor"` | `"Analyzer"`
- `params` — `HashMap<String, f64>` keyed by parameter id

---

## Filesystem Layout (Runtime)

```
<library_dir>/                        # set by user in settings, persisted in app config
  files/                              # can be manually overridem in the config
    drums.wav
    drums.musicum.json
    synths/
      pad.wav
      pad.musicum.json
  catalog/                            # location can be manually overridem in the config
    musicum.db
    collections/
      ep-01.musicum.json
    presets/
      lo-fi.musicum-preset.json
    attachments/
      550e8400-e29b-41d4-a716-446655440000.jpg
  .generated/                         # location can be manually overriden in the config
    waveforms/
      file_{slug}.waveform.json         # raw file waveform
      clip_{slug}.waveform.json         # processed clip waveform
    cache/
      clip_{slug}.mp3
```

App config (Tauri app data dir):

```json
{
  "library_dir": "/Users/lutz/Music/Musicum",
  "generated_dir": null,
  "http_server_enabled": false,
  "http_server_port": 8000
}
```

---

## Tauri Shell

### `src-tauri/Cargo.toml`

```toml
[package]
name = "musicum-desktop"
version = "0.1.0"
edition = "2021"

[dependencies]
tauri        = { version = "2", features = ["shell-open"] }
musicum-core = { path = "../../../libs/musicum-core" }
axum         = { version = "0.7", optional = true }
tokio.workspace = true
serde.workspace = true
serde_json.workspace = true
uuid.workspace  = true

[features]
http-server = ["axum"]
```

### `state.rs`

```rust
use musicum_core::audio::PlaybackEngine;
use sea_orm::DatabaseConnection;
use std::sync::Arc;
use tokio::sync::Mutex;

pub struct AppState {
    pub db: DatabaseConnection,
    pub engine: Arc<Mutex<PlaybackEngine>>,
    pub settings: Arc<Mutex<AppSettings>>,
}

#[derive(serde::Serialize, serde::Deserialize, Clone)]
pub struct AppSettings {
    pub library_dir: String,
    pub generated_dir: Option<String>,
    pub http_server_enabled: bool,
    pub http_server_port: u16,
}
```

`AppSettings` is persisted as JSON at `{tauri_app_config_dir}/settings.json` (resolved via `tauri::api::path::app_config_dir`). On startup, `main.rs` reads and deserializes this file (defaulting to `AppSettings::default()` if absent). Any `settings::set_*` command writes the full struct back to the same file after mutating the in-memory value.

### `main.rs` skeleton

```rust
fn main() {
    tauri::Builder::default()
        .manage(/* build AppState: open DB, init engine */)
        .invoke_handler(tauri::generate_handler![
            // files
            commands::files::get_files,
            commands::files::get_file,
            commands::files::update_file,
            commands::files::delete_file,
            // clips
            commands::clips::get_clips,
            commands::clips::get_clip,
            commands::clips::create_clip,
            commands::clips::update_clip,
            commands::clips::delete_clip,
            commands::clips::cache_clip,
            // collections
            commands::collections::get_collections,
            commands::collections::get_collection,
            commands::collections::create_collection,
            commands::collections::update_collection,
            commands::collections::delete_collection,
            commands::collections::reorder_clips,
            // presets
            commands::presets::get_presets,
            commands::presets::create_preset,
            commands::presets::update_preset,
            commands::presets::delete_preset,
            commands::presets::apply_preset,
            // sync
            commands::sync::sync_library,
            // playback
            commands::playback::play,
            commands::playback::pause,
            commands::playback::stop,
            commands::playback::seek,
            commands::playback::set_processor_param,
            commands::playback::get_playback_state,
            // settings
            commands::settings::get_settings,
            commands::settings::set_library_dir,
            commands::settings::set_generated_dir,
        ])
        .run(tauri::generate_context!())
        .expect("error running Tauri app");
}
```

### Command pattern

```rust
// commands/clips.rs
#[tauri::command]
pub async fn get_clips(
    file_id: String,
    state: tauri::State<'_, AppState>,
) -> Result<Vec<ClipResponse>, String> {
    let id = Uuid::parse_str(&file_id).map_err(|e| e.to_string())?;
    musicum_core::services::clip_service::get_clips_for_file(&state.db, id)
        .await
        .map_err(|e| e.to_string())
}
```

### Tauri events (Rust → frontend)

Emitted via `app_handle.emit(event, payload)`:

| Event | Payload type | Description |
|-------|-------------|-------------|
| `playback:position` | `{ seconds: number }` | Current playhead position |
| `playback:state` | `"playing" \| "paused" \| "stopped"` | State changes |
| `clip:cache_progress` | `{ clip_id: string, percent: number }` | Caching progress |
| `clip:cache_done` | `{ clip_id: string, status: string }` | Cache complete |
| `sync:progress` | `{ message: string }` | Sync status message |
| `sync:done` | `{ added: number, updated: number, removed: number }` | Sync complete |

---

## Frontend (SvelteKit 5 — written fresh)

### Setup

```bash
npm create svelte@latest apps/frontend
# choose: SvelteKit, TypeScript, no additional tooling
```

### Key dependencies

```json
{
  "@tauri-apps/api": "^2",
  "@tauri-apps/plugin-shell": "^2"
}
```

### Structure

```
apps/frontend/src/
├── app.html
├── routes/
│   ├── +layout.svelte          # top nav, global state init
│   ├── +page.svelte            # home / library overview
│   ├── files/
│   │   ├── +page.svelte        # file browser
│   │   └── [f_slug]/
│   │       └── +page.svelte    # file detail + clips list
│   ├── clips/
│   │   └── [c_slug]/
│   │       └── +page.svelte    # clip editor
│   ├── collections/
│   │   ├── +page.svelte        # collection browser
│   │   └── [col_slug]/
│   │       └── +page.svelte    # collection detail + playback
│   ├── presets/
│   │   └── +page.svelte        # preset browser + batch apply
│   └── settings/
│       └── +page.svelte        # library dir, generated dir
│
└── lib/
    ├── api/
    │   ├── client.ts           # invoke() wrapper
    │   ├── files.ts
    │   ├── clips.ts
    │   ├── collections.ts
    │   ├── presets.ts
    │   ├── sync.ts
    │   ├── playback.ts
    │   └── settings.ts
    ├── stores/
    │   ├── playback.svelte.ts      # listens to playback:* events
    │   ├── clip-processors.svelte.ts  # undo/redo + debounced persist
    │   └── plugin-registry.svelte.ts  # plugin descriptors (id, params, ranges)
    ├── components/
    │   ├── audio/
    │   │   ├── ProcessorRack.svelte    # ordered list of active processors
    │   │   ├── ProcessorItem.svelte    # single processor (params UI)
    │   │   ├── ProcessorPicker.svelte  # add processor dialog
    │   │   ├── Waveform.svelte         # waveform visualization
    │   │   └── PlaybackBar.svelte      # play/pause/seek controls + position
    │   ├── FileRow.svelte
    │   ├── ClipRow.svelte
    │   ├── CollectionRow.svelte
    │   ├── PresetRow.svelte
    │   └── forms/
    ├── types/
    │   ├── file.ts
    │   ├── clip.ts
    │   ├── collection.ts
    │   ├── preset.ts
    │   └── playback.ts
    └── utils.ts
```

### API client (`lib/api/client.ts`)

```typescript
import { invoke } from '@tauri-apps/api/core'

export async function call<T>(command: string, args?: Record<string, unknown>): Promise<T> {
  return invoke<T>(command, args)
}
```

All API modules use `call()`:

```typescript
// lib/api/clips.ts
import { call } from './client'
import type { ClipResponse, CreateClipRequest, UpdateClipRequest } from '$lib/types/clip'

export const getClips      = (fileId: string) => call<ClipResponse[]>('get_clips', { fileId })
export const getClip       = (slug: string)   => call<ClipResponse>('get_clip', { slug })
export const createClip    = (req: CreateClipRequest) => call<ClipResponse>('create_clip', req)
export const updateClip    = (slug: string, req: UpdateClipRequest) => call<ClipResponse>('update_clip', { slug, ...req })
export const deleteClip    = (slug: string)   => call<void>('delete_clip', { slug })
export const cacheClip     = (slug: string)   => call<void>('cache_clip', { slug })
```

### Playback store (`lib/stores/playback.svelte.ts`)

```typescript
import { listen } from '@tauri-apps/api/event'
import { play, pause, stop, seek } from '$lib/api/playback'

let position = $state(0)
let state    = $state<'playing' | 'paused' | 'stopped'>('stopped')

listen<{ seconds: number }>('playback:position', e => { position = e.payload.seconds })
listen<string>('playback:state', e => { state = e.payload as typeof state })

export const playback = { get position() { return position }, get state() { return state }, play, pause, stop, seek }
```

### Plugin descriptor loading (`lib/stores/plugin-registry.svelte.ts`)

Plugin WASM is not used for audio processing, but descriptor JSON (parameter names, ranges, defaults) is still needed to render the processor UI. Descriptors are bundled as static JSON files under `apps/frontend/src/lib/plugin-descriptors/` (one file per plugin, e.g. `reverb.json`). These are hand-authored alongside the plugin crates and imported statically at build time — no Tauri command or WASM loading required at runtime.

---

## CLI (`apps/cli`)

A standalone `musicum` binary that links `musicum-core` directly. Works without the desktop app running. SQLite WAL mode (enabled by `musicum-core`'s `connect()`) allows safe concurrent access if the desktop app is open at the same time.

### `Cargo.toml`

```toml
[package]
name = "musicum-cli"
version = "0.1.0"
edition = "2021"
default-run = "musicum"

[[bin]]
name = "musicum"
path = "src/main.rs"

[dependencies]
musicum-core = { path = "../../libs/musicum-core" }
clap         = { version = "4", features = ["derive"] }
tokio.workspace = true
serde_json.workspace = true
anyhow.workspace = true
```

### Command surface

```
musicum sync                          # walk library dir, update DB + sidecars
musicum files list                    # list all files (table output)
musicum files show <slug>             # show file detail + clips
musicum clips list <file-slug>        # list clips for a file
musicum clips create <file-slug> --title "Name"
musicum clips cache <clip-slug>       # run caching pipeline (requires ffmpeg)
musicum collections list
musicum collections show <slug>
musicum presets list
musicum presets apply <preset-slug> <clip-slug>
```

### `main.rs` skeleton

```rust
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(name = "musicum", about = "Musicum audio library CLI")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    Sync,
    Files(commands::files::FilesArgs),
    Clips(commands::clips::ClipsArgs),
    Collections(commands::collections::CollectionsArgs),
    Presets(commands::presets::PresetsArgs),
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();
    let settings = load_settings()?;   // reads same settings.json as desktop app
    let db = musicum_core::db::connect(&settings.library_dir).await?;

    match cli.command {
        Commands::Sync => commands::sync::run(&db, &settings).await?,
        Commands::Files(args) => commands::files::run(&db, args).await?,
        Commands::Clips(args) => commands::clips::run(&db, args).await?,
        Commands::Collections(args) => commands::collections::run(&db, args).await?,
        Commands::Presets(args) => commands::presets::run(&db, args).await?,
    }
    Ok(())
}
```

`load_settings()` reads the same `settings.json` from the Tauri app config dir (`{home}/.config/com.musicum.app/settings.json` on Linux/Mac) so the CLI and desktop app share one config.

### Output format

- Default: human-readable table (via simple `println!` / `format!`)
- `--json` flag on all list/show commands: pretty-printed JSON (useful for scripting)

---

## Audio Engine Design

### Trait hierarchy

```rust
// source.rs
pub trait AudioSource: Send {
    fn fill_buffer(&mut self, buffer: &mut [f32]) -> usize;
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u16;
    fn is_exhausted(&self) -> bool;
    fn duration_secs(&self) -> f64;
    fn position_secs(&self) -> f64;
}

// engine.rs
pub trait AudioEngine: Send {
    fn load_with_processors(&mut self, path: &Path, chain: ProcessorChain) -> anyhow::Result<()>;
    fn load(&mut self, path: &Path);
    fn play(&mut self);
    fn pause(&mut self);
    fn seek(&mut self, secs: f64);
    fn position_secs(&self) -> f64;
    fn seekhead_secs(&self) -> f64;
    fn duration_secs(&self) -> f64;
    fn sample_rate(&self) -> u32;
    fn channels(&self) -> u16;
    fn is_playing(&self) -> bool;
    fn is_exhausted(&self) -> bool;
    fn processor_chain(&self) -> &ProcessorChain;
}
```

`CpalEngine` is the concrete `AudioEngine` implementation (backed by cpal + rtrb ring buffer).

### `chain.rs` — `ProcessorChain`

Built from a `&[ProcessorEdit]` + `&ProcessorRegistry`. Folds enabled `StreamProcessor` entries into a chain of `StreamProcessorNode` wrappers via a fold pattern:

```rust
let source = entries.iter().fold(root_source, |upstream, edit| {
    Box::new(StreamProcessorNode::new(upstream, processor))
});
```

Disabled entries, `StructuralProcessor` / `Analyzer` kinds, and entries whose `processor_id` is not found in the registry are skipped silently at build time (graceful degradation when a processor is missing).

### `node.rs` — `StreamProcessorNode`

```rust
pub struct StreamProcessorNode {
    upstream: Box<dyn AudioSource>,
    processor: Arc<Mutex<Box<dyn StreamProcessor>>>,
    context: ProcessorContext,
}
```

`fill_buffer()` calls `upstream.fill_buffer()`, then applies the processor in-place on the same buffer. Delegates `position_secs()` to the upstream source.

### `player.rs` — `AudioPlayer`

Holds a `ProcessorRegistry` (loaded from the processors directory) and manages the playback queue. `load_current()` builds a `ProcessorChain` from the current clip's `ProcessorEdit` list via `ProcessorChain::from_edits()`, then calls `engine.load_with_processors()`.

### `processor_loader.rs` — `ProcessorRegistry`

Scans a directory for `.dylib` / `.so` / `.dll` files, loads each with `libloading`, resolves the `export_processor` symbol, and registers the resulting processor under its declared `processor_id`. `create(id)` instantiates a fresh processor instance by ID.

### Caching pipeline (planned)

The export/caching pipeline will reuse the same `ProcessorChain` logic as playback:
1. Decode source file with symphonia → `SymphoniaSource`
2. Build `ProcessorChain` from clip's `ProcessorEdit` list
3. Drain the chain offline (non-realtime) → `Vec<f32>`
4. Encode to MP3 via `ffmpeg` subprocess
5. Generate waveform JSON (downsample to ~1000 points per channel)
6. Write both files to `generated_dir`
7. Update `clip.cached`, `cached_path`, `duration` in DB

---

## Build Tooling

### Development

```bash
# Install Tauri CLI
cargo install tauri-cli

# Run desktop app (starts frontend dev server + Tauri window)
cargo tauri dev

# Run frontend dev server alone
cd apps/frontend && npm run dev

# Build WASM plugins (for descriptor JSON generation)
npx nx build audio-plugins

# Run musicum-core tests
cargo test -p musicum-core
```

### Production build

```bash
cargo tauri build   # produces platform-specific installer in target/release/bundle/
```

### `tauri.conf.json` (key settings)

```json
{
  "build": {
    "beforeDevCommand": "cd apps/frontend && npm run dev",
    "beforeBuildCommand": "cd apps/frontend && npm run build",
    "devUrl": "http://localhost:5173",
    "frontendDist": "../../frontend/build"
  },
  "app": {
    "windows": [{ "title": "Musicum", "width": 1280, "height": 800 }]
  },
  "bundle": {
    "identifier": "com.musicum.app",
    "targets": "all"
  }
}
```

---

## Key Rust Dependencies Summary

| Crate | Version | Purpose |
|-------|---------|---------|
| `tauri` | 2 | Desktop shell, IPC, events (planned) |
| `sea-orm` | 1 | ORM + SQLite |
| `symphonia` | 0.5 | Audio decoding (WAV, MP3, FLAC, OGG, AIFF) |
| `cpal` | 0.17 | Cross-platform audio output |
| `rtrb` | 0.3 | Lock-free ring buffer (audio thread ↔ main thread) |
| `musicum-processor-sdk` | local | Unified processor trait + FFI layer |
| `tokio` | 1 | Async runtime |
| `serde` / `serde_json` | 1 | JSON (sidecars, processors, IPC) |
| `uuid` | 1 | ID generation |
| `slug` | 0.1 | Slug generation |
| `walkdir` | 2 | Library directory traversal |
| `chrono` | 0.4 | Timestamps |
| `thiserror` | 1 | Error types |
| `tracing` | 1 | Structured logging |

`ffmpeg` is a system dependency (subprocess) used only for MP3 encoding in the caching pipeline. All other audio I/O uses pure-Rust crates.

---

## Implementation Order

Suggested order to get to a working app incrementally:

1. ✅ **Cargo workspace** — workspace, processor SDK crate, processor crates with dual `crate-type`
2. ✅ **`musicum-core` skeleton** — `lib.rs`, `error.rs`, empty module stubs
3. ✅ **DB layer** — SeaORM entities, `connect()`, `create_all()`, schema version
4. ✅ **Services** — `file_service`, `clip_service`, `sync_service` (file walk + sidecar read/write)
5. ✅ **Audio engine** — `decoder.rs`, `source.rs`, `chain.rs`, `node.rs`, `engine.rs` (cpal), `player.rs`
6. ✅ **Processor loader** — `processor_loader.rs` (dynamic `.dylib` loading), `edit_registry.rs`
7. **CLI** — `apps/cli`, clap commands wrapping the same services, `--json` flag
8. **Tauri shell** — `main.rs`, `state.rs`, wire up `sync_library` command, settings commands
9. **SvelteKit skeleton** — fresh app, `client.ts`, file browser page talking to `get_files`
10. **Clip editor UI** — `ProcessorRack`, `ProcessorItem`, `PlaybackBar`, playback store
11. **Caching pipeline** — offline drain of `ProcessorChain`, ffmpeg encode, waveform generation
12. **Collections + Presets** — service + commands + UI
13. **HTTP adapter** — Axum routes (thin wrappers over same services)
