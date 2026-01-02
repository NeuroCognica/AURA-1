const { spawn } = require('child_process')
const fs = require('fs')
const path = require('path')
const http = require('http')
const net = require('net')

class ServiceManager {
  constructor(configPath) {
    this.configPath = configPath
    this.services = new Map()
    this.healthTimers = new Map()
    this.loadConfig()
  }

  loadConfig() {
    const raw = fs.readFileSync(this.configPath, 'utf8')
    const cfg = JSON.parse(raw)
    this.config = cfg
    for (const s of cfg.services || []) {
      this.services.set(s.name, { config: s, process: null, logs: [] , status: 'stopped'})
    }
  }

  list() {
    const arr = []
    for (const [name, v] of this.services.entries()) {
      arr.push({ name, status: v.status, pid: v.process ? v.process.pid : null, logs: v.logs })
    }
    return arr
  }

  start(name) {
    const svc = this.services.get(name)
    if (!svc) throw new Error('Unknown service ' + name)
    if (svc.process) return svc.process.pid

    const cfg = svc.config
    const cwd = path.resolve(path.dirname(this.configPath), svc.config.cwd || '.')
    let child
    // Support two forms: { cmd: "npm", args: ["run","start"] } or legacy string command
    if (typeof cfg.cmd === 'string' && Array.isArray(cfg.args)) {
      child = spawn(cfg.cmd, cfg.args, { cwd, shell: true })
    } else if (typeof cfg.cmd === 'string' && !Array.isArray(cfg.args)) {
      // legacy single-string command
      child = spawn(cfg.cmd, { cwd, shell: true })
    } else if (typeof cfg === 'object' && cfg.command) {
      // alternate shape
      child = spawn(cfg.command, { cwd, shell: true })
    } else {
      // fallback: stringify
      child = spawn(String(cfg.cmd), { cwd, shell: true })
    }
    svc.process = child
    svc.status = 'starting'
    child.stdout.on('data', (d) => { this._pushLog(name, d.toString()) })
    child.stderr.on('data', (d) => { this._pushLog(name, d.toString()) })
    child.on('exit', (code, sig) => {
      svc.status = 'stopped'
      svc.process = null
      this._pushLog(name, `Process exited code=${code} signal=${sig}\n`)
    })
    const startedCmd = (typeof cfg.cmd === 'string' ? cfg.cmd : JSON.stringify(cfg.cmd)) + (Array.isArray(cfg.args) ? ' ' + cfg.args.join(' ') : '')
    this._pushLog(name, `Started: ${startedCmd}\n`)
    // start health polling
    this._startHealthPoll(name)
    return child.pid
  }

  stop(name) {
    const svc = this.services.get(name)
    if (!svc || !svc.process) return false
    try {
      svc.process.kill()
    } catch (e) {
      this._pushLog(name, `Error killing process: ${String(e)}\n`)
    }
    svc.status = 'stopping'
    return true
  }

  _pushLog(name, msg) {
    const svc = this.services.get(name)
    if (!svc) return
    svc.logs.push({ t: Date.now(), text: msg })
    if (svc.logs.length > 1000) svc.logs.shift()
    if (this.onLog) this.onLog(name, msg)
  }

  _startHealthPoll(name) {
    const svc = this.services.get(name)
    if (!svc) return
    const interval = (this.config.healthIntervalSeconds || 3) * 1000
    const poll = async () => {
      try {
        const hc = svc.config.health
        let healthy = false
        if (!hc) {
          healthy = !!svc.process
        } else if (hc.type === 'http') {
          healthy = await this._httpOk(hc.url)
        } else if (hc.type === 'tcp') {
          healthy = await this._tcpOk(hc.host, hc.port)
        }
        svc.status = healthy ? 'running' : (svc.process ? 'starting' : 'stopped')
      } catch (e) {
        svc.status = 'stopped'
      }
      if (this.onStatus) this.onStatus(name, svc.status)
    }
    this._stopHealthPoll(name)
    poll()
    const id = setInterval(poll, interval)
    this.healthTimers.set(name, id)
  }

  _stopHealthPoll(name) {
    const id = this.healthTimers.get(name)
    if (id) { clearInterval(id); this.healthTimers.delete(name) }
  }

  async _httpOk(url) {
    return new Promise((res) => {
      const req = http.get(url, (r) => {
        res(r.statusCode >= 200 && r.statusCode < 300)
      })
      req.on('error', () => res(false))
      req.setTimeout(2000, () => { req.destroy(); res(false) })
    })
  }

  async _tcpOk(host, port) {
    return new Promise((res) => {
      const s = new net.Socket()
      let done = false
      s.setTimeout(1500)
      s.connect(port, host, () => { done = true; s.destroy(); res(true) })
      s.on('error', () => { if (!done) { done = true; res(false) } })
      s.on('timeout', () => { if (!done) { done = true; s.destroy(); res(false) } })
    })
  }
}

module.exports = ServiceManager
