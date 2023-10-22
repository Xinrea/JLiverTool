import { BrowserWindow, ipcMain } from 'electron'
import path = require('path')
import JLogger from './logger'
import JEvent from './events'
import ConfigStore from './config_store'

const log = JLogger.getInstance('window_manager')

export enum WindowType {
  WINVALID = 'invalid',
  WMAIN = 'main',
  WGIFT = 'gift',
  WSUPERCHAT = 'superchat',
  WSETTING = 'setting',
}

function WindowTypeTitle(wtype: WindowType): string {
  switch (wtype) {
    case WindowType.WMAIN:
      return 'danmu'
    case WindowType.WGIFT:
      return 'gift'
    case WindowType.WSUPERCHAT:
      return 'superchat'
    case WindowType.WSETTING:
      return 'setting'
    default:
  }
  throw new Error('Invalid WindowType')
}

class Window {
  private _window: BrowserWindow
  private _store: ConfigStore

  private _closed_callback: Function

  public wtype: WindowType
  public loaded: boolean = false

  public minimize() {
    if (this._window) {
      this._window.minimize()
    }
  }

  public get top(): boolean {
    if (!this._window) {
      return false
    }
    return this._window.isAlwaysOnTop()
  }
  public set top(b: boolean) {
    if (this._window) {
      this._window.setAlwaysOnTop(b, 'screen-saver')
    }
    this._store.OnTop = b
  }

  public _show: boolean = false
  public get show(): boolean {
    return this._show
  }
  public set show(b: boolean) {
    if (!this._window) {
      return
    }
    if (b) {
      this._window.show()
    } else {
      this._window.hide()
    }
  }

  public constructor(parent: Window, wtype: WindowType, store: ConfigStore) {
    this.wtype = wtype
    this._store = store
    const setting = store.GetWindowCachedSetting(wtype)
    log.debug('Creating window', { window: this.wtype, setting: setting })
    // if not set position, electron will put window in the middle, that's what we need
    // so we firt initialize window and set position later
    // window created with show=false, cuz we need to adjust its position later
    this._window = new BrowserWindow({
      parent: parent ? parent._window : null,
      width: setting.size[0],
      height: setting.size[1],
      minHeight: 200,
      minWidth: 320,
      transparent: true,
      //      frame: false,
      show: false,
      title: WindowTypeTitle(wtype),
      icon: path.join(__dirname, `icons/${this.wtype}.png`),
      webPreferences: {
        preload: path.join(__dirname, 'preload.js'),
      },
    })
    this._window.loadFile(`src/${this.wtype}-window/index.html`)
    if (setting.pos) {
      this._window.setPosition(setting.pos[0], setting.pos[1])
    }
    // main window should always show at starting
    if (wtype == WindowType.WMAIN) {
      this.show = true
    }
    this._window.webContents.openDevTools()
    this.registerEvents()
  }

  public setClosedCallback(f: Function) {
    this._closed_callback = f
  }

  public send(channel: string, args: any) {
    if (this._window) {
      this._window.webContents.send(channel, args)
    }
  }

  // registerEvents handles window related events
  private registerEvents() {
    this._window.on('close', () => {
      this._window.hide()
      this._store.UpdateWindowCachedSetting(this.wtype, {
        pos: [this._window.getPosition()[0], this._window.getPosition()[1]],
        size: [this._window.getSize()[0], this._window.getSize()[1]],
      })
    })
    this._window.on('closed', () => {
      if (this._closed_callback) {
        this._closed_callback()
      }
    })
    this._window.webContents.on('did-finish-load', () => {
      log.debug('Window content loaded', { window: this.wtype })
      this.loaded = true
    })
  }
}

export class WindowManager {
  private _main_window: Window
  private _gift_window: Window
  private _superchat_window: Window
  private _setting_window: Window

  public constructor(store: ConfigStore, mainClosedCallback: Function) {
    this._main_window = new Window(null, WindowType.WMAIN, store)
    this._main_window.setClosedCallback(mainClosedCallback)
    // window should be created and hide at start, cuz gift data stream need to be processed in window render process
    this._gift_window = new Window(this._main_window, WindowType.WGIFT, store)
    this._superchat_window = new Window(
      this._main_window,
      WindowType.WSUPERCHAT,
      store
    )
    this._setting_window = new Window(
      this._main_window,
      WindowType.WSETTING,
      store
    )

    this.registerEvents()
  }

  public loaded(): boolean {
    return (
      this._main_window.loaded &&
      this._gift_window.loaded &&
      this._superchat_window.loaded &&
      this._setting_window.loaded
    )
  }

  public sendTo(wtype: WindowType, channel: JEvent, args: any) {
    let target_window: Window = null
    switch (wtype) {
      case WindowType.WMAIN: {
        target_window = this._main_window
        break
      }
      case WindowType.WGIFT: {
        target_window = this._gift_window
        break
      }
      case WindowType.WSUPERCHAT: {
        target_window = this._superchat_window
        break
      }
      case WindowType.WSETTING: {
        target_window = this._setting_window
        break
      }
      default: {
        log.error('Invalid window type', { wtype })
      }
    }
    if (target_window) {
      target_window.send(JEvent[channel], args)
    }
  }

  // registerEvents initialize all ipcMain related events
  private registerEvents() {
    ipcMain.handle('hideWindow', (_, wtype: WindowType) => {
      this.toggleWindowShow(wtype, false)
    })
    ipcMain.handle('showWindow', (_, wtype: WindowType) => {
      this.toggleWindowShow(wtype, true)
    })
  }

  private toggleWindowShow(wtype: WindowType, show: boolean) {
    switch (wtype) {
      case WindowType.WMAIN: {
        this._main_window.show = show
        return
      }
      case WindowType.WGIFT: {
        this._gift_window.show = show
        return
      }
      case WindowType.WSUPERCHAT: {
        this._superchat_window.show = show
        return
      }
      case WindowType.WSETTING: {
        this._setting_window.show = show
        return
      }
    }
  }

  public minimize(wtype: WindowType) {
    switch (wtype) {
      case WindowType.WMAIN: {
        this._main_window.minimize()
        return
      }
      case WindowType.WGIFT: {
        this._gift_window.minimize()
        return
      }
      case WindowType.WSUPERCHAT: {
        this._superchat_window.minimize()
        return
      }
      case WindowType.WSETTING: {
        this._setting_window.minimize()
        return
      }
    }
  }
}
