import { BrowserWindow, ipcMain } from 'electron'
import ElectronStore = require('electron-store')
import path = require('path')
import Languages from '../i18n'
import Config from './global_config'
import JLogger from './logger'

const L = Languages[Config.language]
const log = JLogger.getInstance()

export enum WindowType {
  WINVALID = 'invalid',
  WMAIN = 'main',
  WGIFT = 'gift',
  WSUPERCHAT = 'superchat',
  WSETTING = 'setting',
}

const DEFAULT_WINDOWSIZE = new Map()
DEFAULT_WINDOWSIZE[WindowType.WMAIN] = [400, 800]

function WindowTypeTitle(wtype: WindowType): string {
  switch (wtype) {
    case WindowType.WMAIN:
      return L.TITLE_MAIN
    case WindowType.WGIFT:
      return L.TITLE_GIFT
    case WindowType.WSUPERCHAT:
      return L.TITLE_SUPERCHAT
    case WindowType.WSETTING:
      return L.TITLE_SETTING
    default:
  }
  throw new Error('Invalid WindowType')
}

class Window {
  private _window: BrowserWindow
  private _store: ElectronStore

  private _closed_callback: Function
  private _load_callback: Function

  public wtype: WindowType
  public loaded: boolean = false

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

  public constructor(
    parent: Window,
    wtype: WindowType,
    store: ElectronStore,
    loadCallback: Function
  ) {
    this.wtype = wtype
    this._store = store
    this._load_callback = loadCallback
    const size = store.get(
      `cache.${this.wtype}Size`,
      DEFAULT_WINDOWSIZE[wtype]
    ) as [number, number]
    // if not set position, electron will put window in the middle, that's what we need
    // so we firt initialize window and set position later
    // window created with show=false, cuz we need to adjust its position later
    this._window = new BrowserWindow({
      parent: parent._window,
      width: size[0],
      height: size[1],
      minHeight: 200,
      minWidth: 320,
      transparent: true,
      frame: false,
      show: false,
      title: WindowTypeTitle(wtype),
      icon: path.join(__dirname, `icons/${this.wtype}.png`),
      webPreferences: {
        preload: path.join(__dirname, 'preload.js'),
      },
    })
    this._window.loadFile(`src/${this.wtype}-window/index.html`)
    if (store.has(`cache.${this.wtype}Pos`)) {
      const pos = store.get(`cache.${this.wtype}Pos`) as [number, number]
      this._window.setPosition(pos[0], pos[1])
    }
    this.top = store.get('config.alwaysOnTop', false) as boolean
    // main window should always show at starting
    if (wtype == WindowType.WMAIN) {
      this.show = true
    }

    this.registerEvents()
  }

  public setClosedCallback(f: Function) {
    this._closed_callback = f
  }

  public send(channel: string, ...args: any[]) {
    if (this._window) {
      this._window.webContents.send(channel, args)
    }
  }

  // registerEvents handles window related events
  private registerEvents() {
    this._window.on('close', function () {
      this._window.hide()
      this._store.set(`cache.${this.wtype}Pos`, this._window.getPosition())
      this._store.set(`cache.${this.wtype}Size`, this._window.getSize())
    })
    this._window.on('closed', function () {
      if (this._close_callback) {
        this._close_callback()
      }
    })
    this._window.webContents.on('did-finish-load', function () {
      log.debug('Window content loaded', this.wtype)
      this.loaded = true
      this._load_callback()
    })
  }
}

export class WindowManager {
  private _main_window: Window
  private _gift_window: Window
  private _superchat_window: Window
  private _setting_window: Window
  private _load_cnt: number = 0
  private _all_load_callback: Function

  public constructor(
    store: ElectronStore,
    allLoadedCallback: Function,
    mainClosedCallback: Function
  ) {
    this._all_load_callback = allLoadedCallback
    this._main_window = new Window(
      null,
      WindowType.WMAIN,
      store,
      this.loadCallback
    )
    this._main_window.setClosedCallback(mainClosedCallback)
    // window should be created and hide at start, cuz gift data stream need to be processed in window render process
    this._gift_window = new Window(
      this._main_window,
      WindowType.WGIFT,
      store,
      this.loadCallback
    )
    this._superchat_window = new Window(
      this._main_window,
      WindowType.WSUPERCHAT,
      store,
      this.loadCallback
    )
    this._setting_window = new Window(
      this._main_window,
      WindowType.WSETTING,
      store,
      this.loadCallback
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

  public sendTo(wtype: WindowType, channel: string, ...args: any[]) {
    let target_window: Window = null
    switch (wtype) {
      case WindowType.WMAIN: {
        target_window = this._main_window
      }
      case WindowType.WGIFT: {
        target_window = this._gift_window
      }
      case WindowType.WSUPERCHAT: {
        target_window = this._superchat_window
      }
      case WindowType.WSETTING: {
        target_window = this._setting_window
      }
    }
    if (target_window) {
      target_window.send(channel, args)
    }
  }

  private loadCallback() {
    this._load_cnt += 1
    if (this._load_cnt == 4) {
      this._all_load_callback()
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
      }
      case WindowType.WGIFT: {
        this._gift_window.show = show
      }
      case WindowType.WSUPERCHAT: {
        this._superchat_window.show = show
      }
      case WindowType.WSETTING: {
        this._setting_window.show = show
      }
    }
  }
}
