const { contextBridge, ipcRenderer } = require('electron')

contextBridge.exposeInMainWorld('launcher', {
  status: async () => ipcRenderer.invoke('launcher:status'),
  // convenience shorthands
  startBackend: async () => ipcRenderer.invoke('backend:start'),
  stopBackend: async () => ipcRenderer.invoke('backend:stop'),
  startFrontend: async () => ipcRenderer.invoke('frontend:start'),
  stopFrontend: async () => ipcRenderer.invoke('frontend:stop'),
  killAll: async () => ipcRenderer.invoke('kill:all'),
  // generic start/stop kept for compatibility
  start: async (service) => ipcRenderer.invoke('launcher:start', service),
  stop: async (service) => ipcRenderer.invoke('launcher:stop', service),
  // event subscriptions
  on: (channel, cb) => {
    const valid = ['backend:log','backend:error','backend:status','backend:stopped','frontend:log','frontend:error','frontend:status','frontend:stopped']
    if (!valid.includes(channel)) return
    const listener = (_e, ...args) => cb(...args)
    ipcRenderer.on(channel, listener)
    return () => ipcRenderer.removeListener(channel, listener)
  }
})
