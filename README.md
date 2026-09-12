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
