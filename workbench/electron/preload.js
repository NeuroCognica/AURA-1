const { contextBridge, ipcRenderer } = require('electron')

contextBridge.exposeInMainWorld('launcher', {
  status: async () => ipcRenderer.invoke('launcher:status'),
  start: async (service) => ipcRenderer.invoke('launcher:start', service),
  stop: async (service) => ipcRenderer.invoke('launcher:stop', service)
})
