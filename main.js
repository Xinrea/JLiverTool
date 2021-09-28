// Modules to control application life and create native browser window
const {app, BrowserWindow} = require('electron')
const path = require('path')
const prompt = require('electron-prompt')
const scriptKeepHistory = `
console.log('Start Removing Elements')
const historyPanel = document.querySelector('.chat-history-panel')
document.body.innerHTML = "<div id='panel' style='position:absolute;'></div>"
document.querySelector('#panel').appendChild(historyPanel)
document.title = 'JDanmaku'
`

const newCSS = `
.chat-history-panel {
  background-color: #1f1f1f !important;
}
.chat-history-panel .chat-history-list .chat-item.danmaku-item {
  color: #e8e8e8 !important;
}
body {
  background-color: #1f1f1f !important;
}
`

function createWindow () {
  // Create the browser window.
  const mainWindow = new BrowserWindow({
    width: 800,
    height: 600,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js')
    }
  })
  prompt({
    title: '请输入房间号',
    label: '房间号',
    value: '21484828',
    inputAttrs: {
      type: 'text'
    },
    type: 'input'
  },mainWindow)
    .then((r) => {
      if (r === null) {
        r = '21484828'
      }
      mainWindow.loadURL('https://live.bilibili.com/' + r)
      let contents = mainWindow.webContents
      contents.setAudioMuted(true)
      contents.executeJavaScript(scriptKeepHistory).then((result) => {
        console.log(result)
      })
      contents.on('did-finish-load', () => {
        contents.insertCSS(newCSS, 'user')
      })
    })
    .catch(console.error)
  // and load the index.html of the app.
  // Open the DevTools.
  // mainWindow.webContents.openDevTools()
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.whenReady().then(() => {
  createWindow()

  app.on('activate', function () {
    // On macOS it's common to re-create a window in the app when the
    // dock icon is clicked and there are no other windows open.
    if (BrowserWindow.getAllWindows().length === 0) createWindow()
  })
})

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on('window-all-closed', function () {
  if (process.platform !== 'darwin') app.quit()
})

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and require them here.
