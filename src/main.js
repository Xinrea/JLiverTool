// Modules to control application life and create native browser window
const { app, ipcMain, screen, BrowserWindow, dialog, nativeTheme } = require('electron')
const path = require('path')
const Store = require('electron-store')
const db = require('electron-db')
const { v4: uuidv4 } = require('uuid')
const ExcelJS = require('exceljs')
const moment = require('moment')
const https = require('https')

let dev
if (process.env.NODE_ENV) {
  dev = process.env.NODE_ENV.includes('development')
}

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
let windowStatus = {
  gift: false,
  superchat: false
}

function exportGift() {
  const workbook = new ExcelJS.Workbook()
  const giftsheet = workbook.addWorksheet('礼物')
  const guardsheet = workbook.addWorksheet('舰长')
  const superchatsheet = workbook.addWorksheet('醒目留言')
  let sheetComplete = 0
  let completeExcel = () => {
    console.log('complete')
    if (sheetComplete === 3) {
      console.log('export to file')
      dialog.showSaveDialog({
        title: '导出礼物数据',
        defaultPath: 'jlivertool_export.xlsx'
      }).then(result => {
        if (!result.canceled) {
          let filename = result.filePath
          console.log('export to', filename)
          workbook.xlsx.writeFile(filename).then(() => {
            console.log('导出成功')
          })
        }
      })
    }
  }
  giftsheet.columns = [
    { header: 'UID', key: 'id', width: 10, numFmt: '0' },
    { header: '用户名', key: 'uname', width: 20 },
    { header: '礼物名', key: 'gift_name', width: 10},
    { header: '数量', key: 'gift_num', width: 10},
    { header: '总价值（电池）', key: 'gift_price', width: 18},
    { header: '时间', key: 'date', width: 20}
  ];
  guardsheet.columns = [
    { header: 'UID', key: 'id', width: 10, numFmt: '0' },
    { header: '用户名', key: 'uname', width: 20 },
    { header: '舰队类型', key: 'gift_name', width: 10},
    { header: '时间', key: 'date', width: 20}
  ];
  superchatsheet.columns = [
    { header: 'UID', key: 'id', width: 10, numFmt: '0' },
    { header: '用户名', key: 'uname', width: 20 },
    { header: '内容', key: 'message', width: 48},
    { header: '价值（RMB）', key: 'price', width: 18},
    { header: '时间', key: 'date', width: 20}
  ];
  db.getRows('gifts', {room: room}, (success,r)=>{
    if (success) {
      r.forEach((gift)=>{
        let row = [gift.data.data.uid, gift.data.data.uname, gift.data.data.giftName, gift.data.data.num, gift.data.data.num*gift.data.data.price/100, moment(gift.data.data.timestamp*1000).format('YYYY-MM-DD HH:mm:ss')]
        giftsheet.addRow(row)
      })
    }
    sheetComplete++
    completeExcel()
  })
  db.getRows('guards', {room: room}, (success,r)=>{
    if (success) {
      r.forEach((guard)=>{
        let row = [guard.data.data.uid, guard.data.data.username, guard.data.data.gift_name, moment(guard.data.data.start_time*1000).format('YYYY-MM-DD HH:mm:ss')]
        guardsheet.addRow(row)
      })
    }
    sheetComplete++
    completeExcel()
  })
  db.getRows('superchats', {room: room}, (success,r)=>{
    if (success) {
      r.forEach((superchat)=>{
        let row = [superchat.data.data.uid, superchat.data.data.user_info.uname, superchat.data.data.message, superchat.data.data.price, moment(superchat.data.data.start_time*1000).format('YYYY-MM-DD HH:mm:ss')]
        superchatsheet.addRow(row)
      })
    }
    sheetComplete++
    completeExcel()
  })
}

function notifyWindowChange() {
  mainWindow?.webContents.send('updateWindowStatus', windowStatus)
}

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
      // Reset All Windows For New Room
      mainWindow?.webContents.send('reset')
      giftWindow?.webContents.send('reset')
      superchatWindow?.webContents.send('reset')
    }
  })
  ipcMain.on('openBrowser', () => {
    require('openurl').open('https://live.bilibili.com/' + room)
  })
  ipcMain.on('openURL', (event, arg) => {
    require('openurl').open(arg)
  })
  ipcMain.on('exportGift', ()=>{
    exportGift()
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
  if (store.has('cache.theme')) {
    nativeTheme.themeSource = store.get('cache.theme')
  } else {
    nativeTheme.themeSource = 'light'
    store.set('cache.theme', 'light')
  }
  ipcMain.on('theme:switch', ()=>{
    if (store.get('cache.theme', 'light') === 'light') {
      nativeTheme.themeSource = 'dark'
      store.set('cache.theme', 'dark')
    } else {
      nativeTheme.themeSource = 'light'
      store.set('cache.theme', 'light')
    }
  })
  checkUpdateFromGithubAPI()
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
    windowStatus.gift = false
    notifyWindowChange()
  })
  ipcMain.on('showGiftWindow', () => {
    windowStatus.gift = true
    notifyWindowChange()
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
    windowStatus.superchat = false
    notifyWindowChange()
  })
  ipcMain.on('showSuperchatWindow', () => {
    windowStatus.superchat = true
    notifyWindowChange()
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

ipcMain.on('updateOpacity', (event, arg) => {
  giftWindow?.webContents.send('updateOpacity')
  superchatWindow?.webContents.send('updateOpacity')
})

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and require them here.

const {
  connecting,
  checkLiveStatus,
  getRoomInfo,
  getOnlineNum,
  getGiftList
} = require('./bilibili')

let service = {
  stopConn: null,
  updateTask: null
}
let giftList

function startBackendService() {
  giftList = new Map()
  checkLiveStatus(room).then((res) => {
    console.log('check room status')
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
      console.log('get gift list')
      res.list.forEach((e) => {
        giftList.set(e.id, {
          animation_frame_num: e.animation_frame_num,
          png: e.frame_animation,
          gif: e.gif
        })
      })
      Promise.all([loadPreGifts(), loadPreGuards(), loadPreSuperchats()]).then(
        () => {
          // All Preload Data Loaded
          let msgHandler = function (type, msg) {
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
                  let giftInfo = {
                    animation_frame_num: 1,
                    png: '',
                    gif: ''
                  }
                  if (giftList.has(msg.data.giftId)) {
                    giftInfo = giftList.get(msg.data.giftId)
                  }
                  giftData = {
                    gif: {
                      frame: giftInfo.animation_frame_num,
                      png: giftInfo.png,
                      gif: giftInfo.gif
                    },
                    data: msg.data
                  }
                  giftWindow?.webContents.send('gift', {
                    id: id,
                    msg: giftData
                  })
                  mainWindow?.webContents.send('gift', {
                    id: id,
                    msg: giftData
                  })
                  break
                }
                if (msg.cmd.includes('INTERACT_WORD')) {
                  mainWindow?.webContents.send('interact', msg)
                  break
                }
                if (msg.cmd.includes('GUARD_BUY')) {
                  // getUserInfo(msg.data.uid).then((userinfo) => {
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
                  guardBuy = {
                    medal:msg.data.medal,
                    face: msg.data.face,
                    name: msg.data.username,
                    gift_name: msg.data.gift_name,
                    guard_level: msg.data.guard_level,
                    price: msg.data.price,
                    timestamp: msg.data.start_time
                  }
                  giftWindow?.webContents.send('guard', {
                    id: id,
                    msg: guardBuy
                  })
                  mainWindow?.webContents.send('guard', {
                    id: id,
                    msg: guardBuy
                  })
                  // })
                  break
                }
                if (msg.cmd == 'SUPER_CHAT_MESSAGE') {
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
                  mainWindow?.webContents.send('superchat', {
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
          }
          service.stopConn = connecting(realroom, msgHandler)
          const mock = {
            cmd: 'SUPER_CHAT_MESSAGE',
            data: {
              background_bottom_color: '#2A60B2',
              background_color: '#EDF5FF',
              background_color_end: '#405D85',
              background_color_start: '#3171D2',
              background_icon: '',
              background_image:
                  'https://i0.hdslb.com/bfs/live/a712efa5c6ebc67bafbe8352d3e74b820a00c13e.png',
              background_price_color: '#7497CD',
              color_point: 0.7,
              dmscore: 120,
              end_time: 1645973356,
              gift: {
                gift_id: 12000,
                gift_name: '醒目留言',
                num: 1
              },
              id: 3420457,
              is_ranked: 0,
              is_send_audit: 0,
              medal_info: {
                anchor_roomid: 21919321,
                anchor_uname: 'HiiroVTuber',
                guard_level: 3,
                icon_id: 0,
                is_lighted: 1,
                medal_color: '#1a544b',
                medal_color_border: 6809855,
                medal_color_end: 5414290,
                medal_color_start: 1725515,
                medal_level: 21,
                medal_name: '王牛奶',
                special: '',
                target_id: 508963009
              },
              message: '这里是一个BV号BV114514测试',
              message_font_color: '#A3F6FF',
              message_trans: '',
              price: 30,
              rate: 1000,
              start_time: 1645973296,
              time: 60,
              token: 'F6EA31D4',
              trans_mark: 0,
              ts: 1645973296,
              uid: 21131097,
              user_info: {
                face: 'http://i0.hdslb.com/bfs/face/dd69fba7016323edf120ef5ef8171d723d76673b.jpg',
                face_frame:
                    'https://i0.hdslb.com/bfs/live/80f732943cc3367029df65e267960d56736a82ee.png',
                guard_level: 3,
                is_main_vip: 0,
                is_svip: 0,
                is_vip: 0,
                level_color: '#61c05a',
                manager: 0,
                name_color: '#00D1F1',
                title: 'title-111-1',
                uname: '慕臣来喝口王牛奶吧',
                user_level: 12
              }
            },
            roomid: 21919321
          }
          setInterval(() => {
            msgHandler(5,mock)
          }, 5000);
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
}

let resetCount = 0
ipcMain.on('reseted', () => {
  resetCount++
  if (resetCount === 3) {
    resetCount = 0
    startBackendService()
  }
})

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
  console.log('remove', info)
  deleteAllRows(info.type, {
    sid: info.id
  })
})

function deleteAllRows(type, condition) {
  db.deleteRow(type, condition, (success, msg) => {
    if (success) {
      deleteAllRows(type, condition)
    }
  })
}

ipcMain.on('clear-gifts', () => {
  deleteAllRows('gifts', {
    room: room
  })
})

ipcMain.on('clear-guards', () => {
  deleteAllRows('guards', {
    room: room
  })
})

ipcMain.on('clear-superchats', () => {
  deleteAllRows('superchats', {
    room: room
  })
})

function loadPreGifts() {
  return new Promise((resolve, reject) => {
    db.getRows('gifts', { room: room }, (s, r) => {
      console.log('load pre gifts:', r.length)
      if (s) {
        for (let i = 0; i < r.length; i++) {
          let id = r[i].sid
          let msg = r[i].data
          let giftInfo = {
            animation_frame_num: 1,
            png: ''
          }
          if (giftList.has(msg.data.giftId)) {
            giftInfo = giftList.get(msg.data.giftId)
          }
          let giftData = {
            gif: {
              frame: giftInfo.animation_frame_num,
              png: giftInfo.png
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
      console.log('load pre guards:', r.length)
      if (s) {
        for (let i = 0; i < r.length; i++) {
          let id = r[i].sid
          let msg = r[i].data
          guardBuy = {
            medal: msg.data.medal,
            face: msg.data.face,
            name: msg.data.username,
            gift_name: msg.data.gift_name,
            guard_level: msg.data.guard_level,
            price: msg.data.price,
            timestamp: msg.data.start_time
          }
          giftWindow?.webContents.send('guard', {
            id: id,
            msg: guardBuy
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
      console.log('load pre superchats:', r.length)
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

// Catch Other Exception
process.on('uncaughtException', function (err) {
  console.log(err)
})

function checkUpdateFromGithubAPI() {
  const options = {
    hostname: 'api.github.com',
    port: 443,
    path: '/repos/xinrea/JLiverTool/releases/latest',
    method: 'GET',
    headers: {
      'User-Agent': 'request'
    }
  }
  const req = https.request(options, (res) => {
    let data = ''
    res.on('data', (d) => {
      data += d
    })
    res.on('end', () => {
      let json = JSON.parse(data)
      let version = json.tag_name
      console.log('latest version:', version)
      if (version !== 'v'+app.getVersion()) {
        console.log('Update available')
        dialog.showMessageBox(mainWindow, {
          type: 'info',
          title: '更新',
          message: '发现新版本 '+version+'，是否前往下载？\n'+json.body,
          buttons: ['是', '否']
        }).then((result) => {
          if (result.response === 0) {
            console.log("Update now")
            require('openurl').open(json.html_url)
          }
        })
      }
    })
  })
  req.on('error', (e) => {
    console.error(e)
  })
  req.end()
}