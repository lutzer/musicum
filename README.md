# Musicum

Musicum is a web app that lets you organize your sound recordings and publish collections of them.

### Features

*  create track collections
*  basic post processing on tracks: normalizations, eq, compression
*  customize track views / ai generate track views
*  sound visualizations 
*  save patch notes
*  import/export tracks
*  automated data backup


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