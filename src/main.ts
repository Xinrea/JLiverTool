// Modules to control application life and create native browser window
import {
  app,
  BrowserWindow,
  dialog,
  ipcMain,
  nativeTheme,
  screen,
  Tray,
  Menu,
  nativeImage,
} from 'electron'
import {
  DanmuMockMessage,
  GiftMockMessage,
  GuardMockMessage,
  SuperChatMockMessage,
} from './common/mock'
import path = require('path')
import Store = require('electron-store')
import https = require('https')
import {
  GetNewQrCode,
  CheckQrCodeStatus,
  Logout,
} from './lib/bilibili/bililogin'
import * as type from './lib/types'
import { WindowManager } from './lib/window_manager'
import JLogger from './lib/logger'
import BackendService from './lib/backend_service'

const log = JLogger.getInstance('main')

// initialize store
Store.initRenderer()
const store = new Store()

// This method will be called when Electron has finished
// initialization and is ready to create browser windows.
// Some APIs can only be used after this event occurs.
let tray = null
app.whenReady().then(() => {
  app.on('activate', function () {})
  // 创建一个托盘实例，指定图标文件路径
  const icon = nativeImage.createFromPath(
    path.join(__dirname, 'assets/icons/main.png')
  )
  tray = new Tray(icon.resize({ width: 16, height: 16 }))

  // 创建一个菜单，包含一些项
  const contextMenu = Menu.buildFromTemplate([
    {
      label: '关于',
      type: 'normal',
      click: () => {
        dialog.showMessageBox(null, {
          type: 'info',
          title: '关于',
          message: 'JLiverTool 弹幕机 v' + app.getVersion(),
          detail: '作者：@Xinrea',
        })
      },
    },
    {
      label: '检查更新',
      type: 'normal',
      click: () => {
        // TODO check update
      },
    },
    {
      label: '鼠标穿透',
      submenu: [
        {
          label: '弹幕窗口',
          type: 'checkbox',
          click: (e) => {},
        },
        {
          label: '礼物窗口',
          type: 'checkbox',
          click: (e) => {},
        },
        {
          label: '醒目留言窗口',
          type: 'checkbox',
          click: (e) => {},
        },
      ],
    },
    {
      label: '设置',
      type: 'normal',
      click: () => {},
    },
    {
      label: '退出',
      type: 'normal',
      click: () => {
        app.quit()
      },
    },
  ])

  // 设置托盘的上下文菜单
  tray.setContextMenu(contextMenu)

  // 注册一个点击事件处理函数
  tray.on('click', () => {})
})

app.on('window-all-closed', function () {
  app.quit()
})

app.on('ready', () => {
  // initialize windows
  const window_manager = new WindowManager(
    store,
    () => {
      log.info('All window loaded, starting backend service')
    },
    () => {
      log.info('Main window closed, stopping backend service')
      app.quit()
    }
  )

  const backend_service = new BackendService()
  backend_service.Start(21484828, store, window_manager)
})
