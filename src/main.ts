// Modules to control application life and create native browser window
import { app, BrowserWindow, dialog, ipcMain, nativeTheme, screen, Tray, Menu, nativeImage, Cookie } from 'electron'
import { DanmuMockMessage, GiftMockMessage, GuardMockMessage, SuperChatMockMessage } from './common/mock'
import path = require('path')
import Store = require('electron-store')
import db = require('electron-db')
import { v4 as uuidv4 } from 'uuid'
import ExcelJS = require('exceljs')
import moment = require('moment')
import https = require('https')
import { GetNewQrCode, CheckQrCodeStatus, QrCodeStatus, Logout } from './bililogin'
import { Cookies } from './types'

let dev: boolean = false
if (process.env.NODE_ENV) {
  dev = process.env.NODE_ENV.includes('development')
}

Store.initRenderer()
const store = new Store()

let room = store.get('config.room', 21484828)
let realroom = 0
let uid = 0

let mainWindow
let giftWindow
let superchatWindow
let settingWindow
let windowCount = 0
let windowStatus = {
  gift: false,
  superchat: false,
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
      dialog
        .showSaveDialog({
          title: '导出礼物数据',
          defaultPath: 'jlivertool_export.xlsx',
        })
        .then((result) => {
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
    { header: '礼物名', key: 'gift_name', width: 10 },
    { header: '数量', key: 'gift_num', width: 10 },
    { header: '总价值（电池）', key: 'gift_price', width: 18 },
    { header: '时间', key: 'date', width: 20 },
  ]
  guardsheet.columns = [
    { header: 'UID', key: 'id', width: 10, numFmt: '0' },
    { header: '用户名', key: 'uname', width: 20 },
    { header: '舰队类型', key: 'gift_name', width: 10 },
    { header: '时间', key: 'date', width: 20 },
  ]
  superchatsheet.columns = [
    { header: 'UID', key: 'id', width: 10, numFmt: '0' },
    { header: '用户名', key: 'uname', width: 20 },
    { header: '内容', key: 'message', width: 48 },
    { header: '价值（RMB）', key: 'price', width: 18 },
    { header: '时间', key: 'date', width: 20 },
  ]
  db.getRows('gifts', { room: room }, (success: boolean, r: any[]) => {
    if (success) {
      r.forEach((gift) => {
        let row = [
          gift.data.data.uid,
          gift.data.data.uname,
          gift.data.data.giftName,
          gift.data.data.num,
          (gift.data.data.num * gift.data.data.price) / 100,
          moment(gift.data.data.timestamp * 1000).format('YYYY-MM-DD HH:mm:ss'),
        ]
        giftsheet.addRow(row)
      })
    }
    sheetComplete++
    completeExcel()
  })
  db.getRows('guards', { room: room }, (success: boolean, r: any[]) => {
    if (success) {
      r.forEach((guard) => {
        let row = [
          guard.data.data.uid,
          guard.data.data.username,
          guard.data.data.gift_name,
          moment(guard.data.data.start_time * 1000).format(
            'YYYY-MM-DD HH:mm:ss'
          ),
        ]
        guardsheet.addRow(row)
      })
    }
    sheetComplete++
    completeExcel()
  })
  db.getRows('superchats', { room: room }, (success: boolean, r: any[]) => {
    if (success) {
      r.forEach((superchat) => {
        let row = [
          superchat.data.data.uid,
          superchat.data.data.user_info.uname,
          superchat.data.data.message,
          superchat.data.data.price,
          moment(superchat.data.data.start_time * 1000).format(
            'YYYY-MM-DD HH:mm:ss'
          ),
        ]
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
    minWidth: 320,
    transparent: true,
    frame: false,
    show: false,
    title: '弹幕',
    icon: path.join(__dirname, 'icons/main.png'),
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
    },
  })
  if (store.has('cache.mainPos')) {
    mainWindow.setPosition(
      store.get('cache.mainPos')[0],
      store.get('cache.mainPos')[1]
    )
  }
  mainWindow.show()
  // and load the index.html of the app.
  mainWindow.loadFile('src/main-window/index.html')
  if (store.get('config.alwaysOnTop', false)) mainWindow.setAlwaysOnTop(true, 'screen-saver')
  mainWindow.on('close', () => {
    store.set('cache.mainPos', mainWindow.getPosition())
    mainWindow.hide()
    // Prevent HDPI Window Size Issue
    mainWindow.setPosition(0, 0)
    store.set('cache.mainSize', mainWindow.getSize())
  })
  mainWindow.on('blur', () => {
    mainWindow?.webContents.send('blur')
  })
  mainWindow.on('closed', () => {
    mainWindow = null
    stopBackendService()
    app.quit()
  })
  ipcMain.on('setAlwaysOnTop', (_, arg) => {
    mainWindow.setAlwaysOnTop(arg, 'screen-saver')
  })
  ipcMain.on('setRoom', (_, arg) => {
    if (arg) {
      room = parseInt(arg)
      console.log('Set room: ', room)
      store.set('config.room', room)
      stopBackendService()
      // Reset All Windows For New Room
      mainWindow?.webContents.send('reset')
      giftWindow?.webContents.send('reset')
      superchatWindow?.webContents.send('reset')
    }
  })
  ipcMain.on('openBrowser', () => {
    require('openurl').open('https://link.bilibili.com/p/center/index/my-room/start-live#/my-room/start-live')
  })
  ipcMain.on('openURL', (_, arg) => {
    require('openurl').open(arg)
  })
  ipcMain.on('exportGift', () => {
    exportGift()
  })
  ipcMain.on('quit', () => {
    stopBackendService()
    app.quit()
  })
  ipcMain.on('setting', () => {
    createSettingWindow()
  })
  ipcMain.on('store-watch', (_, key, newValue) => {
    mainWindow?.webContents.send('store-watch', key, newValue)
    giftWindow?.webContents.send('store-watch', key, newValue)
    superchatWindow?.webContents.send('store-watch', key, newValue)
  })
  mainWindow.webContents.on('did-finish-load', () => {
    windowCount++
    if (windowCount === 3) {
      startBackendService()
    }
  })
  if (store.has('cache.theme')) {
    const themeSetting = store.get('cache.theme', 'light') as string
    nativeTheme.themeSource = themeSetting.includes('light') ? 'light' : (themeSetting.includes('dark') ? 'dark' : 'system')
  } else {
    nativeTheme.themeSource = 'light'
    store.set('cache.theme', 'light')
  }
  ipcMain.on('theme:switch', (_, theme) => {
    nativeTheme.themeSource = theme
  })
  ipcMain.on('minimize', () => {
    mainWindow.minimize()
  })
  checkUpdate()
}

let giftPosInit = true

function createGiftWindow() {
  let giftSize = store.get('cache.giftSize', [400, 400])
  giftWindow = new BrowserWindow({
    width: giftSize[0],
    height: giftSize[1],
    minHeight: 400,
    minWidth: 320,
    transparent: true,
    show: false,
    title: '礼物',
    frame: false,
    skipTaskbar: true,
    icon: path.join(__dirname, 'icons/gift.png'),
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
    },
  })
  giftWindow.loadFile('src/gift-window/index.html')
  giftWindow.setAlwaysOnTop(true, 'screen-saver')
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
      y: mainWinPos[1],
    })
    let relativeMainCenter = [
      mainWinPos[0] - currentDisplay.bounds.x + mainWindow.getSize()[0] / 2,
      mainWinPos[1] - currentDisplay.bounds.y + mainWindow.getSize()[1] / 2,
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
    minWidth: 320,
    transparent: true,
    show: false,
    title: '醒目留言',
    frame: false,
    skipTaskbar: true,
    icon: path.join(__dirname, 'icons/main.png'),
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
    },
  })
  superchatWindow.loadFile('src/superchat-window/index.html')
  superchatWindow.setAlwaysOnTop(true, 'screen-saver')
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
      y: mainWinPos[1],
    })
    let relativeMainCenter = [
      mainWinPos[0] - currentDisplay.bounds.x + mainWindow.getSize()[0] / 2,
      mainWinPos[1] - currentDisplay.bounds.y + mainWindow.getSize()[1] / 2,
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

function createSettingWindow() {
  if (settingWindow) {
    settingWindow.focus()
    return
  }
  settingWindow = new BrowserWindow({
    width: 600,
    height: 400,
    resizable: false,
    title: '设置',
    autoHideMenuBar: true,
    webPreferences: {
      preload: path.join(__dirname, 'preload.js'),
    },
  })
  settingWindow.loadFile('src/setting-window/index.html')
  settingWindow.setAlwaysOnTop(true, 'screen-saver')
  settingWindow.on('closed', () => {
    settingWindow = null
  })
}

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
let tray = null
app.whenReady().then(() => {
  createMainWindow()
  createGiftWindow()
  createSuperchatWindow()
  app.on('activate', function () {
  })
  // 创建一个托盘实例，指定图标文件路径
  const icon = nativeImage.createFromPath(
    path.join(__dirname, 'assets/icons/main.png')
  );
  tray = new Tray(icon.resize({ width: 16, height: 16 }))

  // 创建一个菜单，包含一些项
  const contextMenu = Menu.buildFromTemplate([
    {
      label: '关于', type: 'normal', click: () => {
        dialog.showMessageBox(mainWindow, {
          type: 'info',
          title: '关于',
          message: 'JLiverTool 弹幕机 v' + app.getVersion(),
          detail: '作者：@Xinrea',
        })
      }
    },
    {
      label: '检查更新', type: 'normal', click: () => {
        checkUpdate()
      }
    },
    {
      label: '鼠标穿透', submenu: [
        { label: '弹幕窗口', type: 'checkbox', click: (e) => { mainWindow.setIgnoreMouseEvents(e.checked) } },
        { label: '礼物窗口', type: 'checkbox', click: (e) => { giftWindow.setIgnoreMouseEvents(e.checked) } },
        {
          label: '醒目留言窗口', type: 'checkbox', click: (e) => { superchatWindow.setIgnoreMouseEvents(e.checked) },
        }]
    },
    {
      label: '设置', type: 'normal', click: () => {
        createSettingWindow()
      }
    },
    { label: '退出', type: 'normal', click: () => { stopBackendService(); app.quit() } }
  ])

  // 设置托盘的上下文菜单
  tray.setContextMenu(contextMenu)

  // 注册一个点击事件处理函数
  tray.on('click', () => {
    mainWindow.restore()
  })
})

// Quit when all windows are closed, except on macOS. There, it's common
// for applications and their menu bar to stay active until the user quits
// explicitly with Cmd + Q.
app.on('window-all-closed', function () {
  stopBackendService()
  app.quit()
})

// In this file you can include the rest of your app's specific main process
// code. You can also put them in separate files and require them here.

import { connecting, checkLiveStatus, getRoomInfo, getOnlineNum, getGiftList, GetUserInfo, LiveStatus, WsInfo, getDanmuInfo, DanmuSend, UpdateRoomTitle, StopLive } from './bilibili'

let service = {
  stopConn: null,
  updateTask: null,
  conn: null
}
let giftList


async function startBackendService() {
  let giftList = new Map()
  let cookies = store.get('config.cookies', '')
  let statusRes = await checkLiveStatus(room)
  console.log('check room status')
  realroom = statusRes.room
  uid = statusRes.uid
  let roomRes = await getRoomInfo(realroom)
  mainWindow?.webContents.send('update-room', roomRes)
  let onlineRes = await getOnlineNum(uid, realroom)
  mainWindow?.webContents.send('update-online', onlineRes)
  service.updateTask = setInterval(() => {
    getRoomInfo(realroom).then((res) => {
      mainWindow?.webContents.send('update-room', res)
    })
    getOnlineNum(uid, realroom).then((res) => {
      mainWindow?.webContents.send('update-online', res)
    })
  }, 10 * 1000)
  getGiftList(realroom).then((res: any) => {
    console.log('get gift list')
    res.list.forEach((e) => {
      giftList.set(e.id, {
        animation_frame_num: e.animation_frame_num,
        png: e.frame_animation,
        gif: e.gif,
      })
    })
    Promise.all([loadPreGifts(), loadPreGuards(), loadPreSuperchats()]).then(
      async () => {
        // All Preload Data Loaded
        let msgHandler = function (type: number, msg: any) {
          switch (type) {
            case 3:
              mainWindow?.webContents.send('update-heat', msg)
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
                      data: msg,
                    },
                    () => {
                    }
                  )
                }
                let giftInfo = {
                  animation_frame_num: 1,
                  png: '',
                  gif: '',
                }
                if (giftList.has(msg.data.giftId)) {
                  giftInfo = giftList.get(msg.data.giftId)
                }
                let giftData = {
                  gif: {
                    frame: giftInfo.animation_frame_num,
                    png: giftInfo.png,
                    gif: giftInfo.gif,
                  },
                  data: msg.data,
                }
                giftWindow?.webContents.send('gift', {
                  id: id,
                  msg: giftData,
                })
                mainWindow?.webContents.send('gift', {
                  id: id,
                  msg: giftData,
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
                    data: msg,
                  },
                  () => {
                  }
                )
                let guardBuy = {
                  medal: msg.data.medal,
                  face: msg.data.face,
                  name: msg.data.username,
                  gift_name: msg.data.gift_name,
                  guard_level: msg.data.guard_level,
                  price: msg.data.price,
                  timestamp: msg.data.start_time,
                }
                giftWindow?.webContents.send('guard', {
                  id: id,
                  msg: guardBuy,
                })
                mainWindow?.webContents.send('guard', {
                  id: id,
                  msg: guardBuy,
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
                    data: msg,
                  },
                  (s, m) => {
                  }
                )
                superchatWindow?.webContents.send('superchat', {
                  id: id,
                  msg: msg,
                })
                mainWindow?.webContents.send('superchat', {
                  id: id,
                  msg: msg,
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
                  'entry-effect',
                  msg.data.copy_writing.replace(/(<%)|(%>)/g, '')
                )
                break
              }
            }
          }
        }
        let wsInfo = {} as WsInfo
        wsInfo.roomid = realroom
        if (cookies != '') {
          wsInfo.uid = Number(cookies['DedeUserID'])
        }
        let danmuInfo = await getDanmuInfo(cookies, realroom)
        if (danmuInfo['code'] != 0) {
          console.warn(danmuInfo, 'Using default setting')
          wsInfo.uid = 0
          wsInfo.server = 'wss://broadcastlv.chat.bilibili.com/sub'
        } else {
          wsInfo.token = danmuInfo['data']['token']
          wsInfo.server = 'wss://' + danmuInfo['data']['host_list'][0]['host'] + '/sub'
        }
        service.stopConn = connecting(wsInfo, msgHandler)
        // For debugging
        // if (dev) {
        //   setInterval(() => {
        //     switch (Math.floor(Math.random() * 4)) {
        //       case 0: {
        //         msgHandler(5, SuperChatMockMessage)
        //         break
        //       }
        //       case 1: {
        //         msgHandler(5, GuardMockMessage)
        //         break
        //       }
        //       case 2: {
        //         msgHandler(5, DanmuMockMessage)
        //         break
        //       }
        //       case 3: {
        //         msgHandler(5, GiftMockMessage)
        //         break
        //       }
        //       default:
        //         break
        //     }
        //   }, 10 * 1000)
        // }
      }
    )
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
ipcMain.on('reset', () => {
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

ipcMain.on('remove', (_, info) => {
  console.log('remove', info)
  deleteAllRows(info.type, {
    sid: info.id,
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
    room: room,
  })
})

ipcMain.on('clear-guards', () => {
  deleteAllRows('guards', {
    room: room,
  })
})

ipcMain.on('clear-superchats', () => {
  deleteAllRows('superchats', {
    room: room,
  })
})

ipcMain.handle('getQrCode', async (event) => {
  return await GetNewQrCode()
})

ipcMain.handle('checkQrCode', async (event, authinfo: string) => {
  return await CheckQrCodeStatus(authinfo)
})

ipcMain.handle('getUserInfo', async (event, mid) => {
  return await GetUserInfo(store.get('config.cookies', ''), mid)
})

ipcMain.handle('logout', async () => {
  let cookies = store.get('config.cookies', '')
  if (cookies == '') {
    return
  }
  return await Logout(cookies)
})

ipcMain.handle('getVersion', () => {
  return app.getVersion()
})

ipcMain.handle('sendDanmu', async (event, content) => {
  if (!store.get('config.loggined')) {
    return
  }
  let cookies = store.get('config.cookies') as Cookies
  return await DanmuSend(cookies, realroom, content)
})

ipcMain.handle('callCommand', async (e, cmd) => {
  cmd = cmd.substring(1)
  let parts = cmd.split(' ')
  if (!store.get('config.loggined') as boolean) {
    return
  }
  let cookies = store.get('config.cookies') as Cookies
  switch (parts[0]) {
    case 'title':
      // Set new title for room
      if (parts.length != 2) {
        return
      }
      const newTitle = parts[1]
      if (newTitle.length == 0) {
        return
      }
      let resp = await UpdateRoomTitle(cookies, realroom, newTitle)
      break
    case 'bye':
      resp = await StopLive(cookies, realroom)
      break
  }
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
            png: '',
          }
          if (giftList.has(msg.data.giftId)) {
            giftInfo = giftList.get(msg.data.giftId)
          }
          let giftData = {
            gif: {
              frame: giftInfo.animation_frame_num,
              png: giftInfo.png,
            },
            data: msg.data,
          }
          giftWindow?.webContents.send('gift', {
            id: id,
            msg: giftData,
          })
        }
      }
      resolve(true)
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
          let guardBuy = {
            medal: msg.data.medal,
            face: msg.data.face,
            name: msg.data.username,
            gift_name: msg.data.gift_name,
            guard_level: msg.data.guard_level,
            price: msg.data.price,
            timestamp: msg.data.start_time,
          }
          giftWindow?.webContents.send('guard', {
            id: id,
            msg: guardBuy,
          })
        }
      }
      resolve(true)
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
            msg: msg,
          })
        }
      }
      resolve(true)
    })
  })
}

// Catch Other Exception
process.on('uncaughtException', function (err) {
  console.log(err)
})

function checkUpdate() {
  let updateFile = 'latest.yml'
  // if current platform is MacOS
  if (process.platform === 'darwin') {
    updateFile = 'latest-mac.yml'
  }
  const options = {
    hostname: 'raw.vjoi.cn',
    port: 443,
    path: '/jlivertool/' + updateFile,
    method: 'GET',
    headers: {
      'User-Agent': 'request',
    },
  }
  const req = https.request(options, (res) => {
    let data = ''
    res.on('data', (d) => {
      data += d
    })
    res.on('end', () => {
      // parse yaml data
      let yaml = require('yaml')
      let latest = yaml.parse(data)
      let version = latest.version
      if (version == undefined) return
      var semver = require('semver');
      if (semver.gt(version, app.getVersion())) {
        console.log('Update available')
        dialog
          .showMessageBox(mainWindow, {
            type: 'info',
            title: '更新',
            message: '发现新版本 ' + version + '，是否前往下载？',
            buttons: ['是', '否'],
          })
          .then((result) => {
            if (result.response === 0) {
              console.log('Update now')
              require('openurl').open('https://raw.vjoi.cn/jlivertool/' + latest.path)
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
