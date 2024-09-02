// Modules to control application life and create native browser window
import { app, dialog, Tray, Menu, nativeImage, ipcMain } from 'electron'
import path = require('path')
import { WindowManager } from './lib/window_manager'
import JLogger from './lib/logger'
import BackendService from './lib/backend_service'
import ConfigStore from './lib/config_store'
import JEvent from './lib/events'
import { WindowType } from './lib/types'

require('source-map-support').install()

const log = JLogger.getInstance('main')

// prevent creating default menu for performance
Menu.setApplicationMenu(null)

let tray = null
app.whenReady().then(() => {
  app.on('activate', function () {})
})

app.on('window-all-closed', function () {
  app.quit()
})

// fix window blink when showing from hide
app.commandLine.appendSwitch('wm-window-animations-disabled')

async function checkUpdateFromGithubAPI() {
  const url = 'https://api.github.com/repos/xinrea/JLiverTool/releases/latest'
  const options = {
    method: 'GET',
    headers: {
      'User-Agent': 'request',
    },
  }
  try {
    const resp = await fetch(url, options)
    const json = await resp.json()
    let version = json.tag_name
    log.info(`Latest version [${version}], local version [${app.getVersion()}]`)
    if (version !== 'v' + app.getVersion()) {
      log.info('Update available')
      dialog
        .showMessageBox(null, {
          type: 'info',
          title: '更新',
          message:
            '发现不同的版本 ' + version + '，是否前往下载？\n\n' + json.body,
          buttons: ['从 GitHub 获取', '从国内源获取', '否'],
          defaultId: 0,
          cancelId: 2,
        })
        .then((result) => {
          if (result.response === 0) {
            log.info(`Update now with GitHub url: ${json.html_url}`)
            require('openurl').open(json.html_url)
            return
          }
          if (result.response === 1) {
            log.info(`Update now with selfhost url: ${json.html_url}`)
            require('openurl').open('https://tools.vjoi.cn/')
            return
          }
        })
    }
  } catch (e) {
    log.warn('Check update failed', { error: e })
  }
}

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

  // create tray
  const icon = nativeImage.createFromPath(
    path.join(__dirname, 'assets/icons/main.png')
  )
  tray = new Tray(icon.resize({ width: 16, height: 16 }))
  const contextMenu = Menu.buildFromTemplate([
    {
      label: '关于',
      type: 'normal',
      click: () => {
        dialog.showMessageBox(null, {
          type: 'info',
          title: '关于',
          message: 'JLiverTool 弹幕机 v' + app.getVersion(),
          detail: '作者：@Xinrea\n赞助：https://afdian.net/a/Xinrea',
        })
      },
    },
    {
      label: '检查更新',
      type: 'normal',
      click: () => {
        checkUpdateFromGithubAPI()
      },
    },
    {
      label: '鼠标穿透',
      submenu: [
        {
          label: '弹幕窗口',
          type: 'checkbox',
          click: (event) => {
            window_manager.setWindowClickThrough(
              WindowType.WMAIN,
              event.checked
            )
          },
        },
        {
          label: '礼物窗口',
          type: 'checkbox',
          click: (event) => {
            window_manager.setWindowClickThrough(
              WindowType.WGIFT,
              event.checked
            )
          },
        },
        {
          label: '醒目留言窗口',
          type: 'checkbox',
          click: (event) => {
            window_manager.setWindowClickThrough(
              WindowType.WSUPERCHAT,
              event.checked
            )
          },
        },
      ],
    },
    {
      label: '设置',
      type: 'normal',
      click: () => {
        window_manager.setWindowShow(WindowType.WSETTING, true)
      },
    },
    {
      label: '退出',
      type: 'normal',
      click: () => {
        quitCallback()
      },
    },
  ])

  // 设置托盘的上下文菜单
  tray.setContextMenu(contextMenu)

  // 注册一个点击事件处理函数
  tray.on('click', () => {})

  if (store.CheckUpdate) {
    checkUpdateFromGithubAPI()
  }
})

app.on('quit', () => {
  log.info('App quit')
})
