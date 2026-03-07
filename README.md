# Musicum

Musicum is a web app that lets you organize your sound recordings and publish collections of them.

### Features

*  create track collections
*  basic post processing on tracks: normalizations, eq, compression
*  save patch notes
*  import/export tracks
*  automated data backup

#### TODO

##### Doing

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


##### Backlog

* customizeable track views
* sound visualizations 


## Getting started

TODO: Installation Instructions

```bash
docker compose up
```

## Development

### Requirements

* poetry, install with `brew install poetry`
* python 3.11, install via pyenv
* install cargo with `brew install cargo`

```bash
# Install Dependencies
(cd apps/frontend && npm install)
(cd apps/backend && poetry install)

# Backend
npx nx serve backend          # Run backend dev server
npx nx test backend           # Run backend unit tests
npx nx test backend --watch   # Run tests in watch mode

# Frontend
npx nx serve frontend         # Run frontend dev server
npx nx test frontend          # Run frontend tests
```

Build by @lutzer and Claude Code in 2026