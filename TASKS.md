## TODO

## core
* [x] Sync key to add potential new files from the source directory
* [x] Sync presets collections, everything
* [x] remove caching functionality and caching fields of clips
* [x] add audio plugin pipeline
* [ ] create trim tool, that trims on a specific threshold, similar to the analysis tool in the audio plugins
* [ ] create workflow to export slices, add bool to slice tool, to export all slices
* [ ] add zero crossing cuts to edits
* [ ] load plugins and structural edits dynamically not at compile time
* [ ] integrate vst plugins, and maybe replace own plugin system ? 
* [x] add option to repair filebase: if sidecar doesnt has a soundfile try to find it by its hash and rename the sidecar. if no soundfile with that hash exist ask if sidecar should be removed. also remove the db entry respectivly. have option -f to remove without confirmation
* [x] create file slug from path + filename 
* [x] add option to rebuild sidecars from database: "sync --rebuild-sidecars", it should remove all sidecars in the library folder and recreate them using the database entries
* [x] refactor config singleton
* [ ] create file analysis pipeline for plugins. for example for nomalization, but also for fft analysis
* [ ] create processors file that wraps structural processors and audio plugins + offline processors into a unified interface
    * [ ] unify parameter interface
    * [ ] create analyzer pipeline
    * [ ] adapt interface so it can wrap vst plugins as well
* [ ] create analysis metafiles that lives in .generated folder when a file is added/changed or simply when its not existing and requested by the frontend. name: <file_slug>.data.json
    * [ ] generate waveform data
    * [ ] generate peak level
    * [ ] generate transient detection for bpm detection
    * [ ] generate spectogram
    * [ ] generate duration
    * [ ] add function to clean up orphaned data files
* [ ] speed up export. it is very slow

## cli
* [x] add collection feature
* [x] display folder name and tags in list
* [ ] export/import function for collections and presets
* [x] export audio files in a certain format
* [x] now i want you to integrate the audio plugins in the cli client, they should be listed in the processors list. the list should also show the type, structural or audio-plugin. also i should be able to add them through the editor in presets and clips
* [x] remove plugin dependencies from the cli, should only be in the core library. there should be a registry that exposes the available plugins and processors and that lets you update edits and there should be a an engine function to update processors and plugins while its playing. i want to reuse this interface also with the tauri gui at a later point, so please design that interface to be reusable
* [x] list available output devices and add option for player to play on a specific one
* [ ] add option to start player with a certain preset without writing it to the database
* [x] document code completion setup
* [ ] bug: code completion doesnt seem to work reliably
* [x] musicum clip create <file_slug> should create a clip with the same slug
* [x] rename cli points files -> file, collextions -> collection, clips -> clip, presets -> preset, processors -> processor
* [x] renanme "processor list" to "list-processors"


## gui
* [ ] choose ui framework
* [ ] Filemanager like interface to manage source files, collections, clips and presets with a sidebar
* [ ] Display all items as rows or cards
* [ ] Allow selection of multiple files, collections and clips to do batch operations

### Ideas

* [ ] fast play von sample bibliotheken
* [ ] standard bearbeitung: compress, eq, normalize, distort
* [ ] bpm
* [ ] automatic slicing
* [ ] transient detection (serato sample)
* [ ] sample / lange files getrennt im player 