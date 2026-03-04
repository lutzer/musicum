# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Musicum is a web app for organizing sound recordings and publishing collections.

## Architecture

- **Monorepo**: Nx workspace with two apps
- **Backend**: FastAPI (Python)
- **Frontend**: TBD
- **Deployment**: Docker container

### Project Structure

```
apps/
  backend/     # FastAPI application
  frontend/    # Frontend application
```

## Development Commands

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

## Testing

The backend must have thorough unit test coverage. All new backend features require corresponding unit tests.
