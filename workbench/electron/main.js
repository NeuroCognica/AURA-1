const { app, BrowserWindow, ipcMain } = require('electron')
const path = require('path')

let mainWindow = null

const serviceStatus = {
  ollama: 'stopped',
  backend: 'running',
  frontend: 'running'
}

function createWindow() {
  const fullscreen = process.env.FULLSCREEN === '0' ? false : true
  const frameless = process.env.FRAMELESS === '1'

  mainWindow = new BrowserWindow({
    width: 1280,
    height: 800,
    fullscreen,
    frame: !frameless,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false
    }
  })

  if (process.env.VITE_DEV_SERVER_URL) {
    mainWindow.loadURL(process.env.VITE_DEV_SERVER_URL)
  } else {
    const indexHtml = path.join(__dirname, '..', 'dist', 'index.html')
    mainWindow.loadFile(indexHtml)
  }
}

app.whenReady().then(() => {
  // prefer dev server url when available
  if (process.env.NODE_ENV === 'development') {
    process.env.VITE_DEV_SERVER_URL = 'http://localhost:5173'
  }
  createWindow()

  app.on('activate', function () {
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

app.on('window-all-closed', function () {
  if (process.platform !== 'darwin') app.quit()
})

// IPC handlers: lightweight in-memory stubs
ipcMain.handle('launcher:status', async () => {
  return { services: serviceStatus }
})

ipcMain.handle('launcher:start', async (_event, service) => {
  if (!serviceStatus.hasOwnProperty(service)) return { ok: false, reason: 'unknown' }
  serviceStatus[service] = 'running'
  return { ok: true, service, status: serviceStatus[service] }
})

ipcMain.handle('launcher:stop', async (_event, service) => {
  if (!serviceStatus.hasOwnProperty(service)) return { ok: false, reason: 'unknown' }
  serviceStatus[service] = 'stopped'
  return { ok: true, service, status: serviceStatus[service] }
})
