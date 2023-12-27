import { app, ipcMain } from 'electron'
import path = require('path')
import fs = require('fs')
import { Cookies, DefaultRoomID, RoomID, WindowType, typecast } from './types'
import JLogger from './logger'
import JEvent from './events'

const log = JLogger.getInstance('config_store')

export type WindowSetting = {
  pos: [number, number]
  size: [number, number] | null
}

const DEFAULT_WINDOWSIZE = new Map()
DEFAULT_WINDOWSIZE[WindowType.WMAIN] = [400, 800]
DEFAULT_WINDOWSIZE[WindowType.WGIFT] = [400, 300]
DEFAULT_WINDOWSIZE[WindowType.WSUPERCHAT] = [400, 300]
DEFAULT_WINDOWSIZE[WindowType.WSETTING] = [400, 300]
DEFAULT_WINDOWSIZE[WindowType.WDETAIL] = [400, 300]

class Store {
  private web_contents: Electron.WebContents[]
  private config_path: string
  private registered_callbacks: Map<string, Function[]>
  constructor() {
    this.config_path = path.join(app.getPath('userData'), 'config.json')
    this.web_contents = []
    this.registered_callbacks = new Map()
    this.initConfigHandlers()
    log.debug('Config store initialized', { path: this.config_path })
  }

  private initConfigHandlers() {
    ipcMain.handle(JEvent[JEvent.INVOKE_STORE_GET], (_, key, d) => {
      log.debug('store-get called', { key, d })
      return this.get(key, d)
    })

    ipcMain.handle(JEvent[JEvent.INVOKE_STORE_SET], (_, key, value) => {
      log.debug('store-set called', { key, value })
      return this.set(key, value)
    })

    // this helps us to send value change event to all renderers
    // so store-register must be called in preload.ts
    ipcMain.handle(JEvent[JEvent.INVOKE_STORE_REGISTER], (event) => {
      log.debug('store-register called', { sender: event.sender.getTitle() })
      this.web_contents.push(event.sender)
    })
  }

  get(key: string, default_value: any = null) {
    if (!fs.existsSync(this.config_path)) {
      fs.writeFileSync(this.config_path, '{}')
    }
    const configJson = fs.readFileSync(this.config_path, 'utf8')
    let configJs = JSON.parse(configJson)
    const keys = key.split('.')
    let cur = configJs
    for (let i = 0; i < keys.length; i++) {
      const k = keys[i]
      if (!(k in cur)) {
        return default_value
      }
      cur = cur[k]
    }
    return cur
  }

  set(key: string, value: any) {
    const configPath = path.join(app.getPath('userData'), 'config.json')
    if (!fs.existsSync(configPath)) {
      fs.writeFileSync(configPath, '{}')
    }
    const configJson = fs.readFileSync(configPath, 'utf8')
    const configJs = JSON.parse(configJson)
    const keys = key.split('.')
    let cur = configJs
    for (let i = 0; i < keys.length - 1; i++) {
      const k = keys[i]
      if (!(k in cur)) {
        cur[k] = {}
      }
      cur = cur[k]
    }
    cur[keys[keys.length - 1]] = value
    const newConfigJson = JSON.stringify(configJs)
    fs.writeFileSync(configPath, newConfigJson)
    this.web_contents.forEach((wc) => {
      wc.send(JEvent[JEvent.EVENT_STORE_WATCH], key, value)
    })
    if (this.registered_callbacks.has(key)) {
      this.registered_callbacks.get(key).forEach((callback) => {
        callback(value)
      })
    }
  }

  onDidChange(key: string, callback: Function) {
    if (!this.registered_callbacks.has(key)) {
      this.registered_callbacks.set(key, [])
    }
    this.registered_callbacks.get(key).push(callback)
  }
}

export class ConfigStore {
  private _store: Store

  constructor() {
    log.debug('Initializing config store')
    this._store = new Store()
  }

  public get Cookies(): Cookies {
    const cookiesData = this._store.get('config.cookies', {})
    return new Cookies(cookiesData)
  }
  public set Cookies(cookies: Cookies) {
    this._store.set('config.cookies', cookies)
  }

  public get OnTop(): boolean {
    return this._store.get('config.alwaysOnTop', false) as boolean
  }
  public set OnTop(b: boolean) {
    this._store.set('config.alwaysOnTop', b)
  }

  public get Room(): RoomID {
    const room = this._store.get('config.room', DefaultRoomID) as RoomID
    // it might be another type in old version, need to check it here
    if (!room.hasOwnProperty('short_id') || !room.hasOwnProperty('room_id')) {
      return DefaultRoomID
    }
    return typecast(RoomID, room)
  }
  public set Room(room: RoomID) {
    this._store.set('config.room', room)
  }

  public get IsLogin(): boolean {
    return this._store.get('config.login', false) as boolean
  }

  public set IsLogin(b: boolean) {
    this._store.set('config.login', b)
  }

  public set IsMergeEnabled(b: boolean) {
    this._store.set('config.merge', b)
  }

  public get IsMergeEnabled(): boolean {
    return this._store.get('config.merge', false) as boolean
  }

  public get MergeRooms(): RoomID[] {
    const rooms = this._store.get('config.merge_rooms', []) as RoomID[]
    return rooms.map((room) => typecast(RoomID, room))
  }

  public GetWindowCachedSetting(wtype: WindowType): WindowSetting {
    let setting = this._store.get(`config.window.${wtype}`, {
      pos: null,
      size: DEFAULT_WINDOWSIZE[wtype],
    })
    if (setting.size === null) {
      log.fatal('Window size is null', { wtype })
    }
    return setting as WindowSetting
  }
  public UpdateWindowCachedSetting(wtype: WindowType, setting: WindowSetting) {
    this._store.set(`config.window.${wtype}`, setting)
  }

  public onDidChange(key: string, callback: Function) {
    this._store.onDidChange(key, callback)
  }
}

export default ConfigStore
