import { ipcMain, BrowserWindow } from 'electron'
import path = require('path')
import JLogger from './logger'
import JEvent from './events'
import ConfigStore from './config_store'
import { DetailInfo, WindowType } from './types'

const log = JLogger.getInstance('window_manager')
const dev = process.env.DEBUG === 'true'

function WindowTypeTitle(win_type: WindowType): string {
  switch (win_type) {
    case WindowType.WMAIN:
      return 'danmu'
    case WindowType.WGIFT:
      return 'gift'
    case WindowType.WSUPERCHAT:
      return 'superchat'
    case WindowType.WSETTING:
      return 'setting'
    case WindowType.WDETAIL:
      return 'detail'
    case WindowType.WRANK:
      return 'rank'
    default:
  }
  throw new Error('Invalid WindowType')
}

class Window {
  private readonly _window: BrowserWindow
  private _store: ConfigStore
  private _is_quit: boolean = false

  private _closed_callback: Function

  public win_type: WindowType
  public loaded: boolean = false

  public minimize() {
    if (this._window) {
      this._window.minimize()
    }
  }

  public minimizable(): boolean {
    if (!this._window) {
      return false
    }
    return this._window.isMinimizable()
  }

  public setMinimizable(b: boolean) {
    if (this._window) {
      this._window.setMinimizable(b)
    }
  }

  public setIgnoreMouseEvents(b: boolean) {
    if (this._window) {
      this._window.setIgnoreMouseEvents(b)
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

  public setQuit() {
    this._is_quit = true
  }

  public close() {
    this.setQuit()
    if (this._window) {
      this._window.close()
    }
  }

  public id(): number {
    if (!this._window) {
      return -1
    }
    return this._window.id
  }

  public constructor(
    parent: Window,
    win_type: WindowType,
    store: ConfigStore,
    loaded_callback?: Function,
    closed_callback?: Function
  ) {
    this.win_type = win_type
    this._store = store
    const setting = store.GetWindowCachedSetting(win_type)
    log.debug('Creating window', { window: this.win_type, setting: setting })
    let should_frame = false
    let should_transparent = true
    if (dev && process.platform === 'darwin') {
      log.debug(
        'Running in dev mode on MacOS, window transparent set to false for frame, or window drag will not work'
      )
      should_transparent = false
      should_frame = true
    }
    // if not set position, electron will put window in the middle, that's what we need,
    // so we first initialize window and set position later
    // window created with show=false, cuz we need to adjust its position later
    this._window = new BrowserWindow({
      parent: parent ? parent._window : null,
      width: setting.size[0],
      height: setting.size[1],
      minHeight: 200,
      minWidth: 200,
      transparent: should_transparent,
      frame: should_frame,
      show: false,
      title: WindowTypeTitle(win_type),
      icon: path.join(__dirname, `icons/${this.win_type}.png`),
      webPreferences: {
        preload: path.join(__dirname, 'preload.js'),
      },
    })
    void this._window.loadFile(`src/${this.win_type}-window/index.html`)
    if (setting.pos) {
      this._window.setPosition(setting.pos[0], setting.pos[1])
    }
    // main window should always show at starting
    if (win_type == WindowType.WMAIN) {
      this.show = true
    }

    this._window.on('blur', () => {
      this._window.webContents.send(JEvent[JEvent.EVENT_WINDOW_BLUR], {})
    })
    this._window.on('focus', () => {
      this._window.webContents.send(JEvent[JEvent.EVENT_WINDOW_FOCUS], {})
    })
    this._window.once('ready-to-show', () => {
      if (loaded_callback) {
        loaded_callback()
      } else {
        log.warn('Window loaded callback not set', { window: this.win_type })
      }
    })

    if (closed_callback) {
      this._closed_callback = closed_callback
    }

    if (this._store.OnTop) {
      this.top = true
    }

    if (dev) {
      this._window.webContents.openDevTools()
    }
    this.registerEvents()
  }

  public send(channel: string, args: any) {
    if (this._window) {
      this._window.webContents.send(channel, args)
    }
  }

  // registerEvents handles window related events
  private registerEvents() {
    this._window.on('close', (e) => {
      // Do not hide main window
      if (this.win_type != WindowType.WMAIN) {
        this._window.hide()
      }
      if (!this._is_quit) {
        e.preventDefault()
        return
      }
      this._store.UpdateWindowCachedSetting(this.win_type, {
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
      log.debug('Window content loaded', { window: this.win_type })
      this.loaded = true
    })
  }
}

export class WindowManager {
  private _main_window: Window
  private _gift_window: Window
  private _superchat_window: Window
  private _setting_window: Window
  private _detail_windows: Window
  private _rank_window: Window
  private _main_loaded_callback: Function
  private _gift_loaded_callback: Function
  private _superchat_loaded_callback: Function
  private readonly _main_close_callback: Function
  private readonly _config_store: ConfigStore

  public constructor(store: ConfigStore, mainClosedCallback: Function) {
    this._config_store = store
    this._main_close_callback = mainClosedCallback
  }

  public Start() {
    this._main_window = new Window(
      null,
      WindowType.WMAIN,
      this._config_store,
      this._main_loaded_callback,
      this._main_close_callback
    )
    // window should be created and hide at start, cuz gift data stream need to be processed in window render process
    this._gift_window = new Window(
      this._main_window,
      WindowType.WGIFT,
      this._config_store,
      this._gift_loaded_callback
    )

    this._superchat_window = new Window(
      this._main_window,
      WindowType.WSUPERCHAT,
      this._config_store,
      this._superchat_loaded_callback
    )

    this._setting_window = new Window(
      this._main_window,
      WindowType.WSETTING,
      this._config_store
    )
    this._detail_windows = new Window(
      this._main_window,
      WindowType.WDETAIL,
      this._config_store
    )
    this._rank_window = new Window(
      this._main_window,
      WindowType.WRANK,
      this._config_store
    )
    this.registerEvents()
  }

  public setMainLoadedCallback(f: Function) {
    this._main_loaded_callback = f
  }

  public setGiftLoadedCallback(f: Function) {
    this._gift_loaded_callback = f
  }

  public setSuperChatLoadedCallback(f: Function) {
    this._superchat_loaded_callback = f
  }

  public loaded(): boolean {
    return (
      this._main_window.loaded &&
      this._gift_window.loaded &&
      this._superchat_window.loaded &&
      this._setting_window.loaded
    )
  }

  public SendTo(win_type: WindowType, channel: JEvent, args: any) {
    let target_window: Window = null
    switch (win_type) {
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
      case WindowType.WDETAIL: {
        target_window = this._detail_windows
        break
      }
      case WindowType.WRANK: {
        target_window = this._rank_window
        break
      }
      default: {
        log.error('Invalid window type', { win_type: win_type })
      }
    }
    if (target_window) {
      target_window.send(JEvent[channel], args)
      return true
    } else {
      return false
    }
  }

  // registerEvents initialize all ipcMain related events
  private registerEvents() {
    ipcMain.handle(
      JEvent[JEvent.INVOKE_WINDOW_HIDE],
      (e, win_type: WindowType) => {
        log.debug('[EVENT] INVOKE_WINDOW_HIDE', { context: e })
        this.setWindowShow(win_type, false)
      }
    )
    ipcMain.handle(
      JEvent[JEvent.INVOKE_WINDOW_SHOW],
      (e, win_type: WindowType) => {
        log.debug('[EVENT] INVOKE_WINDOW_SHOW', { context: e })
        this.setWindowShow(win_type, true)
      }
    )
    ipcMain.handle(
      JEvent[JEvent.INVOKE_WINDOW_MINIMIZE],
      (_, win_type: WindowType) => {
        this.minimize(win_type)
      }
    )
    ipcMain.handle(
      JEvent[JEvent.INVOKE_WINDOW_ALWAYS_ON_TOP],
      (_, win_type: WindowType, value: boolean) => {
        switch (win_type) {
          case WindowType.WMAIN: {
            this._main_window.top = value
            return
          }
          case WindowType.WGIFT: {
            this._gift_window.top = value
            return
          }
          case WindowType.WSUPERCHAT: {
            this._superchat_window.top = value
            return
          }
          case WindowType.WSETTING: {
            this._setting_window.top = value
            return
          }
        }
      }
    )
    ipcMain.handle(
      JEvent[JEvent.INVOKE_WINDOW_MINIMIZABLE],
      (_, win_type: WindowType, value: boolean) => {
        switch (win_type) {
          case WindowType.WMAIN: {
            this._main_window.setMinimizable(value)
            return
          }
          case WindowType.WGIFT: {
            this._gift_window.setMinimizable(value)
            return
          }
          case WindowType.WSUPERCHAT: {
            this._superchat_window.setMinimizable(value)
            return
          }
          case WindowType.WSETTING: {
            this._setting_window.setMinimizable(value)
            return
          }
        }
      }
    )
  }

  public setWindowShow(win_type: WindowType, show: boolean) {
    log.debug('Set window show', { type: win_type, show: show })
    switch (win_type) {
      case WindowType.WMAIN: {
        // main window should always show
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
      case WindowType.WDETAIL: {
        this._detail_windows.show = show
        return
      }
      case WindowType.WRANK: {
        this._rank_window.show = show
        return
      }
    }
  }

  public setWindowClickThrough(win_type: WindowType, click_through: boolean) {
    switch (win_type) {
      case WindowType.WMAIN: {
        this._main_window.setIgnoreMouseEvents(click_through)
        return
      }
      case WindowType.WGIFT: {
        this._gift_window.setIgnoreMouseEvents(click_through)
        return
      }
      case WindowType.WSUPERCHAT: {
        this._superchat_window.setIgnoreMouseEvents(click_through)
        return
      }
      case WindowType.WSETTING: {
        this._setting_window.setIgnoreMouseEvents(click_through)
        return
      }
      case WindowType.WDETAIL: {
        this._detail_windows.setIgnoreMouseEvents(click_through)
        return
      }
    }
  }

  public minimize(win_type: WindowType) {
    switch (win_type) {
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
      case WindowType.WDETAIL: {
        this._detail_windows.minimize()
        return
      }
      case WindowType.WRANK: {
        this._rank_window.minimize()
        return
      }
    }
  }

  public updateDetailWindow(detail_info: DetailInfo) {
    log.debug('Create detail window', { uid: detail_info.sender.uid })
    this._detail_windows.send(JEvent[JEvent.EVENT_DETAIL_UPDATE], detail_info)
    this._detail_windows.show = true
  }

  public Stop() {
    this._main_window.close()
    this._gift_window.close()
    this._superchat_window.close()
    this._setting_window.close()
    this._detail_windows.close()
    this._rank_window.close()
  }
}
