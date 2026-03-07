#  Tasks

## Doing

* simplify frontend logic:
    * frontpage shows collections + tracks ( with pagination)
    * /user shows profile + collections + tracks and options to add tracks and collections (with pagination)
    * /create_collection creates new collection
    * /create_track creates new track
    * /collection/*slug* shows collection details
    * /track/*slug* shows track details
    * /collection/*slug*/edit and /track/*slug*/edit lets you edit

* create component from collection list to be reused in frontpage and user page with pagination
* create component from track list to be reusded in frontpage user page and collection page with pagination

## Backlog

* customizeable track views
* sound visualizations, 
    * wave file visualizations
    * spectrogram with frequencies and note slider
* postprocessing:
    * normalization with ffmpeg
    * 3band eq with wasm
    * compressor with wasm
