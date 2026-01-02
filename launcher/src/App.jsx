import React, { useEffect, useState } from 'react'

function StatusLight({ status }) {
  const color = status === 'running' ? 'green' : status === 'starting' ? 'orange' : 'gray'
  return <span style={{ display: 'inline-block', width: 12, height: 12, borderRadius: 6, background: color, marginRight: 8 }} />
}

export default function App() {
  const [services, setServices] = useState([])
  const [logs, setLogs] = useState([])

  async function refresh() {
    if (window.launcher) {
      const list = await window.launcher.listServices()
      setServices(list)
    }
  }

  useEffect(() => { refresh() }, [])

  useEffect(() => {
    if (!window.launcher) return
    const statusCb = (d) => {
      setServices((s) => s.map(x => x.name === d.name ? { ...x, status: d.status } : x))
    }
    const logCb = (d) => {
      setLogs((l) => [{ name: d.name, text: d.msg, t: Date.now() }, ...l].slice(0, 200))
    }
    window.launcher.onStatus(statusCb)
    window.launcher.onLog(logCb)
    return () => {}
  }, [])

  const start = async (name) => {
    await window.launcher.startService(name)
    refresh()
  }
  const stop = async (name) => {
    await window.launcher.stopService(name)
    refresh()
  }

  return (
    <div className="app-root">
      <h1>AURA Launcher</h1>
      <div style={{ display: 'grid', gridTemplateColumns: '1fr 140px', gap: 12 }}>
        <div>
          {services.map(s => (
            <div key={s.name} style={{ display: 'flex', alignItems: 'center', padding: '8px 0', borderBottom: '1px solid #eee' }}>
              <StatusLight status={s.status} />
              <div style={{ flex: 1 }}>
                <div style={{ fontWeight: 600 }}>{s.name}</div>
                <div style={{ fontSize: 12, color: '#666' }}>{s.pid ? `pid: ${s.pid}` : 'not running'}</div>
              </div>
              <div>
                <button onClick={() => start(s.name)} style={{ marginRight: 8 }}>Start</button>
                <button onClick={() => stop(s.name)}>Stop</button>
              </div>
            </div>
          ))}
        </div>
        <div>
          <div style={{ fontWeight: 700, marginBottom: 8 }}>Logs</div>
          <div style={{ height: 420, overflow: 'auto', background: '#111', color: '#0f0', padding: 8, fontFamily: 'monospace', fontSize: 12 }}>
            {logs.map((l, idx) => (
              <div key={idx}>[{new Date(l.t).toLocaleTimeString()}] [{l.name}] {l.text}</div>
            ))}
          </div>
        </div>
      </div>
    </div>
  )
}
