# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Musicum is a web app for organizing sound recordings and publishing collections.

## Architecture

- **Monorepo**: Nx workspace
- **Backend**: FastAPI (Python) with Poetry for dependency management
- **Frontend**: SvelteKit (TypeScript)
- **Audio Processing**: Rust compiled to WebAssembly
- **Deployment**: Docker container

### Project Structure

```
apps/
  backend/          # FastAPI application (Python/Poetry)
  frontend/         # SvelteKit application (TypeScript)
  audio-processor/  # Audio DSP library (Rust → WebAssembly)
```

## Development Commands

```bash
# Run the app in Docker
docker compose up

# Backend (FastAPI/Python)
nx serve backend          # Run backend dev server (port 8000)
nx test backend           # Run backend unit tests
nx install backend        # Install Poetry dependencies

# Frontend (SvelteKit)
nx serve frontend         # Run frontend dev server
nx test frontend          # Run frontend tests
nx build frontend         # Build for production

# Audio Processor (Rust/WASM)
nx build audio-processor  # Build WASM package (requires wasm-pack)
nx test audio-processor   # Run Rust tests
```

## Testing

The backend must have thorough unit test coverage. All new backend features require corresponding unit tests.

## Prerequisites

- Python 3.11+ with Poetry
- Node.js 18+
- Rust with wasm-pack (`cargo install wasm-pack`)
