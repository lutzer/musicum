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

Build by lutzer and Claude Code

## Install

TODO: Installation Instructions

## Development

```bash
# Run the app in Docker
docker compose up

# Backend
nx serve backend          # Run backend dev server
nx test backend           # Run backend unit tests
nx test backend --watch   # Run tests in watch mode

# Frontend
nx serve frontend         # Run frontend dev server
nx test frontend          # Run frontend tests
```