// Modules to control application life and create native browser window
import { app, dialog, Tray, Menu, nativeImage, ipcMain } from 'electron'
import path = require('path')
import { WindowManager } from './lib/window_manager'
import JLogger from './lib/logger'
import BackendService from './lib/backend_service'
import ConfigStore from './lib/config_store'
import JEvent from './lib/events'

const log = JLogger.getInstance('main')

let tray = null
app.whenReady().then(() => {
  app.on('activate', function () {})
  const icon = nativeImage.createFromPath(
    path.join(__dirname, 'assets/icons/main.png')
  )
  tray = new Tray(icon.resize({ width: 16, height: 16 }))
  const contextMenu = Menu.buildFromTemplate([
    {
      label: '关于',
      type: 'normal',
      click: () => {
        dialog
          .showMessageBox(null, {
            type: 'info',
            title: '关于',
            message: 'JLiverTool 弹幕机 v' + app.getVersion(),
            detail: '作者：@Xinrea',
          })
          .then((r) => {})
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

let quitCallback = async () => {}
app.on('ready', () => {
  const store = new ConfigStore()
  const window_manager = new WindowManager(store, quitCallback)
  const backend_service = new BackendService(store, window_manager)
  // initialize windows
  quitCallback = async () => {
    await backend_service.Stop()
    app.quit()
  }

  void backend_service.Start()

  ipcMain.handle(JEvent[JEvent.INVOKE_APP_QUIT], quitCallback)
})
