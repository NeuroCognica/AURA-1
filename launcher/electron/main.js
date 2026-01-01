const { app, BrowserWindow, ipcMain } = require('electron')
const path = require('path')
const http = require('http')
const fs = require('fs')
const { spawn } = require('child_process')
const ServiceManager = require('./services')

// CONFIGURATION
const CONFIG_PATH = path.resolve(__dirname, '..', 'launcher.config.json')
// Launcher loads its OWN Vite dev server (launcher React UI on port 5170)
const LAUNCHER_URL = process.env.VITE_URL || 'http://127.0.0.1:5170/'

let mainWindow
let svcMgr = null
// Electron should not spawn or manage the dev server; the external launcher owns it.

// --- UTILITIES ---

// Note: Dev-server spawning and probing removed. The Python launcher is the supervisor.

// --- WINDOW MANAGEMENT ---

function createWindow(url) {
  console.log('[Main] Creating window with URL:', url)
  mainWindow = new BrowserWindow({
    width: 1000,
    height: 720,
    backgroundColor: '#1e1e1e',
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
      contextIsolation: true,
      nodeIntegration: false
    }
  })

  if (url) {
      mainWindow.loadURL(url)
  } else {
      mainWindow.loadURL('data:text/html;charset=utf-8,<html><body style="background:black;color:red"><h1>ERROR: COULD NOT LOAD VITE</h1></body></html>')
  }
}

function setupServiceManager() {
  try {
    console.log('[Main] Initializing ServiceManager...')
    svcMgr = new ServiceManager(CONFIG_PATH)
    svcMgr.onLog = (name, msg) => {
      if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.webContents.send('svc-log', { name, msg })
      }
    }
    svcMgr.onStatus = (name, status) => {
      if (mainWindow && !mainWindow.isDestroyed()) {
        mainWindow.webContents.send('svc-status', { name, status })
      }
    }
    console.log('[Main] ServiceManager initialized.')
  } catch (e) {
    console.error('[Main] Failed to init ServiceManager', e)
  }
}

// --- IPC HANDLERS ---

ipcMain.handle('svc-list', () => {
  return svcMgr ? svcMgr.list() : []
})
ipcMain.handle('svc-start', (ev, name) => {
  if (!svcMgr) return { error: 'service-manager-not-ready' }
  try { return svcMgr.start(name) } catch (e) { return { error: String(e) } }
})
ipcMain.handle('svc-stop', (ev, name) => {
  if (!svcMgr) return { error: 'service-manager-not-ready' }
  try { return svcMgr.stop(name) } catch (e) { return { error: String(e) } }
})

// --- LIFECYCLE ---

app.whenReady().then(async () => {
  console.log('[Main] Electron starting...')

  const url = LAUNCHER_URL
  console.log('[Main] Loading launcher UI from:', url)
  createWindow(url);
  setupServiceManager();
})

app.on('window-all-closed', () => {
  if (process.platform !== 'darwin') app.quit()
})

app.on('will-quit', () => {
  // No dev process to kill — launcher owns service lifecycle.
})
