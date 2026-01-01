const { contextBridge, ipcRenderer } = require('electron')

contextBridge.exposeInMainWorld('launcher', {
  listServices: () => ipcRenderer.invoke('svc-list'),
  startService: (name) => ipcRenderer.invoke('svc-start', name),
  stopService: (name) => ipcRenderer.invoke('svc-stop', name),
  onStatus: (cb) => ipcRenderer.on('svc-status', (e, d) => cb(d)),
  onLog: (cb) => ipcRenderer.on('svc-log', (e, d) => cb(d)),
})
