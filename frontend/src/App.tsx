import { useEffect, useState } from 'react'
import './App.css'

type SystemSnapshot = {
  cpu_usage_percent: number
  memory: {
    total_bytes: number
    used_bytes: number
    usage_percent: number
  }
  filesystem: {
    total_bytes: number
    used_bytes: number
    available_bytes: number
    usage_percent: number
  }
  disk: {
    read_bytes_per_second: number
    write_bytes_per_second: number
  }
  network: {
    name: string
    rx_bytes_per_second: number
    tx_bytes_per_second: number
  }[]
  temperatures: {
    name: string
    temperature_celsius: number
  }[]
  gpu: {
    utilization_percent: number
    vram_used: number
    vram_total: number
  } | null
  services: {
    name: string
    active: boolean
    failed: boolean
    status: string
  }[]
  processes: {
    cpu_percent: number
    process: {
      pid: number
      name: string
      state: string
      memory_bytes: number
      command: string
    }
  }[]
}

const formatBytes = (bytes: number) => {
  if (bytes === 0) return '0 B'

  const units = ['B', 'KB', 'MB', 'GB', 'TB']
  const index = Math.min(
    Math.floor(Math.log(bytes) / Math.log(1024)),
    units.length - 1,
  )

  return `${(bytes / 1024 ** index).toFixed(1)} ${units[index]}`
}

function App() {
  const [snapshot, setSnapshot] = useState<SystemSnapshot | null>(null)
  const [connected, setConnected] = useState(false)

  useEffect(() => {
    const socket = new WebSocket('ws://127.0.0.1:3000/api/ws')

    socket.onopen = () => setConnected(true)

    socket.onmessage = (event) => {
      try {
        const data = JSON.parse(event.data) as SystemSnapshot
        setSnapshot(data)
      } catch {
        // Ignore malformed messages.
      }
    }

    socket.onclose = () => setConnected(false)
    socket.onerror = () => setConnected(false)

    return () => socket.close()
  }, [])

  if (!snapshot) {
    return (
      <main className="app-shell">
        <header className="topbar">
          <div>
            <div className="eyebrow">LINUXPULSE</div>
            <h1>System Intelligence Dashboard</h1>
            <p className="subtitle">Connecting to live Linux telemetry...</p>
          </div>

          <div className="connection offline">
            <span />
            CONNECTING
          </div>
        </header>
      </main>
    )
  }

  return (
    <main className="app-shell">
      <header className="topbar">
        <div>
          <div className="eyebrow">LINUXPULSE</div>
          <h1>System Intelligence Dashboard</h1>
          <p className="subtitle">
            Real-time visibility into your Linux machine.
          </p>
        </div>

        <div className={`connection ${connected ? 'online' : 'offline'}`}>
          <span />
          {connected ? 'LIVE' : 'DISCONNECTED'}
        </div>
      </header>

      <section className="metric-grid">
        <article className="metric-card">
          <span className="card-label">CPU</span>
          <strong>{snapshot.cpu_usage_percent.toFixed(1)}%</strong>
          <div className="progress">
            <div
              className="progress-fill"
              style={{ width: `${Math.min(snapshot.cpu_usage_percent, 100)}%` }}
            />
          </div>
        </article>

        <article className="metric-card">
          <span className="card-label">MEMORY</span>
          <strong>{snapshot.memory.usage_percent.toFixed(1)}%</strong>
          <small>
            {formatBytes(snapshot.memory.used_bytes)} /{' '}
            {formatBytes(snapshot.memory.total_bytes)}
          </small>
        </article>

        <article className="metric-card">
          <span className="card-label">DISK</span>
          <strong>{snapshot.filesystem.usage_percent.toFixed(1)}%</strong>
          <small>{formatBytes(snapshot.filesystem.used_bytes)} used</small>
        </article>

        <article className="metric-card">
          <span className="card-label">GPU</span>
          <strong>
            {snapshot.gpu
              ? `${snapshot.gpu.utilization_percent.toFixed(0)}%`
              : 'N/A'}
          </strong>
          <small>
            {snapshot.gpu
              ? `${formatBytes(snapshot.gpu.vram_used)} / ${formatBytes(snapshot.gpu.vram_total)}`
              : 'Telemetry unavailable'}
          </small>
        </article>
      </section>

      <section className="dashboard-grid">
        <article className="panel">
          <div className="panel-title">
            <span className="eyebrow">DISK I/O</span>
            <h2>Throughput</h2>
          </div>

          <div className="stats-row">
            <div>
              <span>READ</span>
              <strong>{formatBytes(snapshot.disk.read_bytes_per_second)}/s</strong>
            </div>
            <div>
              <span>WRITE</span>
              <strong>{formatBytes(snapshot.disk.write_bytes_per_second)}/s</strong>
            </div>
          </div>
        </article>

        <article className="panel">
          <div className="panel-title">
            <span className="eyebrow">THERMALS</span>
            <h2>Temperature</h2>
          </div>

          {snapshot.temperatures.map((zone) => (
            <div className="list-row" key={zone.name}>
              <span>{zone.name}</span>
              <strong>{zone.temperature_celsius.toFixed(1)}°C</strong>
            </div>
          ))}
        </article>

        <article className="panel wide">
          <div className="panel-title">
            <span className="eyebrow">NETWORK</span>
            <h2>Interface activity</h2>
          </div>

          <div className="table">
            <div className="table-row heading">
              <span>INTERFACE</span>
              <span>DOWNLOAD</span>
              <span>UPLOAD</span>
            </div>

            {snapshot.network.map((network) => (
              <div className="table-row" key={network.name}>
                <span>{network.name}</span>
                <span>{formatBytes(network.rx_bytes_per_second)}/s</span>
                <span>{formatBytes(network.tx_bytes_per_second)}/s</span>
              </div>
            ))}
          </div>
        </article>

        <article className="panel wide">
          <div className="panel-title">
            <span className="eyebrow">SYSTEM HEALTH</span>
            <h2>Services</h2>
          </div>

          <div className="table">
            {snapshot.services.map((service) => (
              <div className="table-row" key={service.name}>
                <span>{service.name}</span>
                <span
                  className={
                    service.failed
                      ? 'status failed'
                      : service.active
                        ? 'status healthy'
                        : 'status inactive'
                  }
                >
                  {service.failed
                    ? 'FAILED'
                    : service.active
                      ? 'HEALTHY'
                      : 'INACTIVE'}
                </span>
                <span>{service.status}</span>
              </div>
            ))}
          </div>
        </article>

        <article className="panel full">
          <div className="panel-title">
            <span className="eyebrow">PROCESSES</span>
            <h2>Top CPU consumers</h2>
          </div>

          <div className="table">
            <div className="table-row heading">
              <span>PROCESS</span>
              <span>CPU</span>
              <span>MEMORY</span>
            </div>

            {snapshot.processes.map(({ cpu_percent, process }) => (
              <div className="table-row" key={process.pid}>
                <span>
                  <strong>{process.name}</strong>
                  <small>PID {process.pid}</small>
                </span>
                <span>{cpu_percent.toFixed(2)}%</span>
                <span>{formatBytes(process.memory_bytes)}</span>
              </div>
            ))}
          </div>
        </article>
      </section>
    </main>
  )
}

export default App
