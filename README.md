# LinuxPulse

Real-time Linux system intelligence dashboard built with Rust and React.

## Features

- CPU, memory, disk, network, thermal, GPU and process monitoring
- systemd service health
- deterministic rule-based insights
- live WebSocket dashboard
- REST API
- automated backend tests

## Stack

- Rust + Tokio + Axum
- Linux /proc and /sys
- React + TypeScript + Vite
- WebSocket

## Run

### Backend
cd backend && cargo run

### Frontend
cd frontend && npm install && npm run dev

Open http://localhost:5173

## Architecture

Linux `/proc` + `/sys` + systemd → Rust collectors → `SystemSnapshot` → rule-based insight engine → Axum REST/WebSocket API → React dashboard.

## API Endpoints

- `GET /api/health` — backend health check
- `GET /api/system` — latest system snapshot
- `GET /api/insights` — current insights
- `WS /api/ws` — live snapshot and insight stream

## Testing

The backend currently has **21 automated tests** covering the major collectors and the insight engine.

```bash
cd backend
cargo test
```

## Development

Backend:

```bash
cd backend
cargo run
```

Frontend:

```bash
cd frontend
npm install
npm run dev
```

Then open `http://localhost:5173`.

## Project Goal

LinuxPulse is a systems-focused portfolio project for understanding Linux telemetry, Rust backend development, real-time WebSocket communication, deterministic system insights, and React dashboard development.
