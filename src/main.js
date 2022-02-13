// Modules to control application life and create native browser window
const { app, BrowserWindow, ipcMain, screen } = require('electron')
const path = require('path')
const Store = require('electron-store')
const db = require('electron-db')
const { v4: uuidv4 } = require('uuid')

let dev = process.env === 'development'

Store.initRenderer()
const store = new Store()

let room = store.get('config.room', 21484828)
let realroom = 0
let uid = 0

if (dev) {
  require('electron-reload')(__dirname, {
    electron: path.join('node_modules', '.bin', 'electron')
  })
}

let mainWindow
let giftWindow
let superchatWindow
let windowCount = 0

function createMainWindow() {
  // Create the browser window.
  let mainSize = store.get('cache.mainSize', [400, 800])
  mainWindow = new BrowserWindow({
    width: mainSize[0],
    height: mainSize[1],
    minHeight: 200,
    minWidth: 300,
    transparent: true,
    frame: false,
    show: false,
    title: '弹幕',
    icon: path.join(__dirname, 'icons/main.png'),
    webPreferences: {
      preload: path.join(__dirname, 'preload.js')
    }
  })
  if (store.has('cache.mainPos')) {
    mainWindow.setPosition(
      store.get('cache.mainPos')[0],
      store.get('cache.mainPos')[1]
    )
  }
  mainWindow.show()
  // and load the index.html of the app.
  mainWindow.loadFile('src/mainwindow/index.html')
  // Open the DevTools.
  if (dev) mainWindow.webContents.openDevTools()
  mainWindow.on('close', () => {
    store.set('cache.mainPos', mainWindow.getPosition())
    mainWindow.hide()
    // Prevent HDPI Window Size Issue
    mainWindow.setPosition(0, 0)
    store.set('cache.mainSize', mainWindow.getSize())
  })
  mainWindow.on('closed', () => {
    mainWindow = null
    stopBackendService()
    app.quit()
  })
  ipcMain.on('setAlwaysOnTop', (event, arg) => {
    mainWindow.setAlwaysOnTop(arg, 'screen-saver')
  })
  ipcMain.on('setRoom', (event, arg) => {
    if (arg) {
      room = parseInt(arg)
      store.set('config.room', room)
      stopBackendService()
      startBackendService()
    }
  })
  ipcMain.on('openBrowser', () => {
    require('child_process').exec('start https://live.bilibili.com/' + room)
  })
  ipcMain.on('quit', () => {
    stopBackendService()
    app.quit()
  })
  mainWindow.webContents.on('did-finish-load', () => {
    windowCount++
    if (windowCount === 3) {
      startBackendService()
    }
  })
}

let giftPosInit = true
function createGiftWindow() {
  let giftSize = store.get('cache.giftSize', [400, 400])
  giftWindow = new BrowserWindow({
    width: giftSize[0],
    height: giftSize[1],
    minHeight: 400,
    minWidth: 300,
    transparent: true,
    show: false,
    title: '礼物',
    frame: false,
    skipTaskbar: true,
    icon: path.join(__dirname, 'icons/gift.png'),
    webPreferences: {
      preload: path.join(__dirname, 'preload.js')
    }
  })
  giftWindow.loadFile('src/giftwindow/index.html')
  giftWindow.setAlwaysOnTop(true, 'screen-saver')
  if (dev) giftWindow.webContents.openDevTools()
  ipcMain.on('hideGiftWindow', () => {
    giftWindow.hide()
  })
  ipcMain.on('showGiftWindow', () => {
    if (!giftPosInit) {
      giftWindow.show()
      return
    }
    if (giftWindow.isVisible()) {
      return
    }
    giftPosInit = false
    let mainWinPos = mainWindow.getPosition()
    let currentDisplay = screen.getDisplayNearestPoint({
      x: mainWinPos[0],
      y: mainWinPos[1]
    })
    let relativeMainCenter = [
      mainWinPos[0] - currentDisplay.bounds.x + mainWindow.getSize()[0] / 2,
      mainWinPos[1] - currentDisplay.bounds.y + mainWindow.getSize()[1] / 2
    ]
    if (relativeMainCenter[0] > currentDisplay.size.width / 2) {
      // Prevent HDPI Window Size Issue
      giftWindow.setPosition(currentDisplay.bounds.x, currentDisplay.bounds.y)
      giftWindow.setPosition(
        mainWinPos[0] - giftWindow.getSize()[0],
        mainWinPos[1]
      )
    } else {
      giftWindow.setPosition(
        mainWinPos[0] + mainWindow.getSize()[0],
        mainWinPos[1]
      )
    }
    giftWindow.show()
  })
  giftWindow.webContents.on('did-finish-load', () => {
    windowCount++
    if (windowCount === 3) {
      startBackendService()
    }
  })
  giftWindow.on('close', () => {
    store.set('cache.giftPos', giftWindow.getPosition())
    giftWindow.hide()
    // Prevent HDPI Window Size Issue
    giftWindow.setPosition(0, 0)
    store.set('cache.giftSize', giftWindow.getSize())
  })
}

let superchatPosInit = true
function createSuperchatWindow() {
  let superchatSize = store.get('cache.superchatSize', [400, 400])
  superchatWindow = new BrowserWindow({
    width: superchatSize[0],
    height: superchatSize[1],
    minHeight: 400,
    minWidth: 300,
    transparent: true,
    show: false,
    title: '醒目留言',
    frame: false,
    skipTaskbar: true,
    icon: path.join(__dirname, 'icons/main.png'),
    webPreferences: {
      preload: path.join(__dirname, 'preload.js')
    }
  })
  superchatWindow.loadFile('src/superchatwindow/index.html')
  superchatWindow.setAlwaysOnTop(true, 'screen-saver')
  if (dev) superchatWindow.webContents.openDevTools()
  ipcMain.on('hideSuperchatWindow', () => {
    superchatWindow.hide()
  })
  ipcMain.on('showSuperchatWindow', () => {
    if (!superchatPosInit) {
      superchatWindow.show()
      return
    }
    if (superchatWindow.isVisible()) {
      return
    }
    superchatPosInit = false
    let mainWinPos = mainWindow.getPosition()
    let currentDisplay = screen.getDisplayNearestPoint({
      x: mainWinPos[0],
      y: mainWinPos[1]
    })
    let relativeMainCenter = [
      mainWinPos[0] - currentDisplay.bounds.x + mainWindow.getSize()[0] / 2,
      mainWinPos[1] - currentDisplay.bounds.y + mainWindow.getSize()[1] / 2
    ]
    if (relativeMainCenter[0] > currentDisplay.size.width / 2) {
      // Prevent HDPI Window Size Issue
      superchatWindow.setPosition(
        currentDisplay.bounds.x,
        currentDisplay.bounds.y
      )
      superchatWindow.setPosition(
        mainWinPos[0] - superchatWindow.getSize()[0],
        mainWinPos[1]
      )
    } else {
      superchatWindow.setPosition(
        mainWinPos[0] + mainWindow.getSize()[0],
        mainWinPos[1]
      )
    }
    superchatWindow.show()
  })
  superchatWindow.webContents.on('did-finish-load', () => {
    windowCount++
    if (windowCount === 3) {
      startBackendService()
    }
  })
  superchatWindow.on('close', () => {
    store.set('cache.superchatPos', superchatWindow.getPosition())
    superchatWindow.hide()
    // Prevent HDPI Window Size Issue
    superchatWindow.setPosition(0, 0)
    store.set('cache.superchatSize', superchatWindow.getSize())
  })
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
app.whenReady().then(() => {
  createMainWindow()
  createGiftWindow()
  createSuperchatWindow()
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
  if (process.platform !== 'darwin') {
    stopBackendService()
    app.quit()
  }
})

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and require them here.

const {
  connecting,
  checkLiveStatus,
  getRoomInfo,
  getOnlineNum,
  getGiftList,
  getUserInfo
} = require('./bilibili')

let service = {
  stopConn: null,
  updateTask: null
}
let giftList

function startBackendService() {
  giftList = new Map()
  checkLiveStatus(room).then((res) => {
    realroom = res.room
    uid = res.uid
    getRoomInfo(realroom).then((res) => {
      mainWindow?.webContents.send('updateroom', res)
    })
    getOnlineNum(uid, realroom).then((res) => {
      mainWindow?.webContents.send('updateonline', res)
    })
    service.updateTask = setInterval(() => {
      getRoomInfo(realroom).then((res) => {
        mainWindow?.webContents.send('updateroom', res)
      })
      getOnlineNum(uid, realroom).then((res) => {
        mainWindow?.webContents.send('updateonline', res)
      })
    }, 10 * 1000)
    getGiftList(realroom).then((res) => {
      res.list.forEach((e) => {
        giftList.set(e.id, {
          animation_frame_num: e.animation_frame_num,
          png: e.frame_animation
        })
      })
      Promise.all([loadPreGifts(), loadPreGuards(), loadPreSuperchats()]).then(
        () => {
          // All Preload Data Loaded
          service.stopConn = connecting(realroom, function (type, msg) {
            switch (type) {
              case 3:
                mainWindow?.webContents.send('updateheat', msg)
                break
              case 5: {
                if (msg.cmd.includes('DANMU_MSG')) {
                  mainWindow?.webContents.send('danmu', msg)
                  break
                }
                if (msg.cmd.includes('SEND_GIFT')) {
                  if (!giftList.has(msg.data.giftId)) {
                    break
                  }
                  if (
                    msg.data.coin_type === 'silver' &&
                    msg.data.giftName == '辣条'
                  ) {
                    break
                  }
                  let id = msg.data.batch_combo_id
                  if (id === '') {
                    id = uuidv4()
                  }
                  if (msg.data.coin_type === 'gold') {
                    db.insertTableContent(
                      'gifts',
                      {
                        room: room,
                        sid: id,
                        data: msg
                      },
                      () => {}
                    )
                  }
                  giftData = {
                    gif: {
                      frame: giftList.get(msg.data.giftId).animation_frame_num,
                      png: giftList.get(msg.data.giftId).png
                    },
                    data: msg.data
                  }
                  giftWindow?.webContents.send('gift', {
                    id: id,
                    msg: giftData
                  })
                  break
                }
                if (msg.cmd.includes('INTERACT_WORD')) {
                  mainWindow?.webContents.send('interact', msg)
                  break
                }
                if (msg.cmd === 'GUARD_BUY') {
                  let id = uuidv4()
                  db.insertTableContent(
                    'guards',
                    {
                      room: room,
                      sid: id,
                      data: msg
                    },
                    () => {}
                  )
                  getUserInfo(msg.data.uid).then((userinfo) => {
                    guardBuy = {
                      medal: userinfo.fans_medal.medal,
                      face: userinfo.face,
                      name: userinfo.name,
                      gift_name: msg.data.gift_name,
                      guard_level: msg.data.guard_level,
                      price: msg.data.price,
                      timestamp: msg.data.start_time
                    }
                    giftWindow?.webContents.send('guard', {
                      sid: id,
                      msg: guardBuy
                    })
                  })
                  break
                }
                if (msg.cmd === 'SUPER_CHAT_MESSAGE') {
                  let id = uuidv4()
                  db.insertTableContent(
                    'superchats',
                    {
                      room: room,
                      sid: id,
                      data: msg
                    },
                    (s, m) => {}
                  )
                  superchatWindow?.webContents.send('superchat', {
                    id: id,
                    msg: msg
                  })
                  break
                }
                // 在线舰长数
                // { cmd: 'ONLINE_RANK_COUNT', data: { count: 22 } }
                if (msg.cmd.includes('ONLINE_RANK_COUNT')) {
                  mainWindow?.webContents.send('online_guard', msg.data.count)
                  break
                }
                // STOP_LIVE_ROOM_LIST 没啥用
                if (msg.cmd === 'STOP_LIVE_ROOM_LIST') {
                  break
                }
                // ENTRY_EFFECT 舰长入场
                if (msg.cmd.includes('ENTRY_EFFECT')) {
                  mainWindow?.webContents.send(
                    'entry_effect',
                    msg.data.copy_writing.replace(/(<%)|(%>)/g, '')
                  )
                  break
                }
              }
            }
          })
        }
      )
    })
  })
}

function stopBackendService() {
  if (service.updateTask) {
    clearInterval(service.updateTask)
    service.updateTask = null
  }
  if (service.stopConn) {
    service.stopConn()
    service.conn = null
  }
  mainWindow?.webContents.send('reset')
  giftWindow?.webContents.send('reset')
  superchatWindow?.webContents.send('reset')
}

// DB related
initDB()
function initDB() {
  // {
  //   room: 21484828,
  //   id: uuid(),
  //   data: 'raw'
  // }
  db.createTable('gifts', (success, msg) => {
    console.log(success)
    console.log(msg)
  })
  db.createTable('guards', (success, msg) => {
    console.log(success)
    console.log(msg)
  })
  db.createTable('superchats', (success, msg) => {
    console.log(success)
    console.log(msg)
  })
}

ipcMain.on('remove', (event, info) => {
  deleteAllRows(info.type, info.id)
})

function deleteAllRows(type, id) {
  db.deleteRow(
    type,
    {
      sid: id
    },
    (success, msg) => {
      if (success) {
        deleteAllRows(type, id)
      }
    }
  )
}

function loadPreGifts() {
  return new Promise((resolve, reject) => {
    db.getRows('gifts', { room: room }, (s, r) => {
      if (s) {
        for (let i = 0; i < r.length; i++) {
          let id = r[i].sid
          let msg = r[i].data
          let giftData = {
            gif: {
              frame: giftList.get(msg.data.giftId).animation_frame_num,
              png: giftList.get(msg.data.giftId).png
            },
            data: msg.data
          }
          giftWindow?.webContents.send('gift', {
            id: id,
            msg: giftData
          })
        }
      }
      resolve()
    })
  })
}

function loadPreGuards() {
  return new Promise((resolve, reject) => {
    db.getRows('guards', { room: room }, (s, r) => {
      if (s) {
        for (let i = 0; i < r.length; i++) {
          let id = r[i].sid
          let msg = r[i].data
          getUserInfo(msg.data.uid).then((userinfo) => {
            guardBuy = {
              medal: userinfo.fans_medal.medal,
              face: userinfo.face,
              name: userinfo.name,
              gift_name: msg.data.gift_name,
              guard_level: msg.data.guard_level,
              price: msg.data.price,
              timestamp: msg.data.start_time
            }
            giftWindow?.webContents.send('guard', {
              id: id,
              msg: guardBuy
            })
          })
        }
      }
      resolve()
    })
  })
}

function loadPreSuperchats() {
  return new Promise((resolve, reject) => {
    db.getRows('superchats', { room: room }, (s, r) => {
      if (s) {
        for (let i = 0; i < r.length; i++) {
          let id = r[i].sid
          let msg = r[i].data
          superchatWindow?.webContents.send('superchat', {
            id: id,
            msg: msg
          })
        }
      }
      resolve()
    })
  })
}
