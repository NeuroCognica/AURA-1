const { app, BrowserWindow } = require('electron')
const path = require('path')

function createWindow() {
  const win = new BrowserWindow({
    width: 800,
    height: 600,
    backgroundColor: '#000000',
    webPreferences: {
      nodeIntegration: true,
      contextIsolation: false
    }
  })

  console.log('Creating window...')
  const loadPath = path.join(__dirname, 'sanity.html')
  console.log('Loading file:', loadPath)
  win.loadFile(loadPath)
}

app.whenReady().then(createWindow)

app.on('window-all-closed', () => {
  app.quit()
})
