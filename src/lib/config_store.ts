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
DEFAULT_WINDOWSIZE[WindowType.WSETTING] = [800, 400]
DEFAULT_WINDOWSIZE[WindowType.WDETAIL] = [400, 300]
DEFAULT_WINDOWSIZE[WindowType.WRANK] = [400, 300]

const config_path = path.join(app.getPath('userData'), 'config_v2.json')

class Store {
  private web_contents: Electron.WebContents[]
  private registered_callbacks: Map<string, Function[]>
  constructor() {
    this.web_contents = []
    this.registered_callbacks = new Map()
    this.initConfigHandlers()
    // initialize log level
    JLogger.updateLogLevel(this.get('config.log_level', 'info') as string)
    this.onDidChange('config.log_level', (level: string) => {
      JLogger.updateLogLevel(level)
    })
  }

  private initConfigHandlers() {
    ipcMain.handle(JEvent[JEvent.INVOKE_STORE_GET], (_, key, d) => {
      return this.get(key, d)
    })

    ipcMain.handle(JEvent[JEvent.INVOKE_STORE_SET], (_, key, value) => {
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
    if (!fs.existsSync(config_path)) {
      fs.writeFileSync(config_path, '{}')
    }
    const configJson = fs.readFileSync(config_path, 'utf8')
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
    if (!fs.existsSync(config_path)) {
      fs.writeFileSync(config_path, '{}')
    }
    log.debug('update config', { key, value })
    const configJson = fs.readFileSync(config_path, 'utf8')
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
    fs.writeFileSync(config_path, newConfigJson)
    this.web_contents.forEach((wc) => {
      if (wc.isDestroyed()) {
        return
      }
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
    return this._store.get('config.always-on-top', false) as boolean
  }
  public set OnTop(b: boolean) {
    this._store.set('config.always-on-top', b)
  }

  public get CheckUpdate(): boolean {
    return this._store.get('config.check_update', true) as boolean
  }
  public set CheckUpdate(b: boolean) {
    this._store.set('config.check_update', b)
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

  public get MaxDetailEntry(): number {
    return this._store.get('config.max_detail_entry', 100) as number
  }

  public set MaxDetailEntry(n: number) {
    this._store.set('config.max_detail_entry', n)
  }

  public get GuardEffect(): boolean {
    return this._store.get('config.guard-effect', true) as boolean
  }

  public get LevelEffect(): boolean {
    return this._store.get('config.level-effect', true) as boolean
  }

  public get tts_endpoint(): string {
    return this._store.get('config.tts_provider_endpoint', '') as string
  }

  public get tts_appkey(): string {
    return this._store.get('config.tts_provider_appkey', '') as string
  }

  public get tts_access_key(): string {
    return this._store.get('config.tts_provider_access_key', '') as string
  }

  public get tts_secret_key(): string {
    return this._store.get('config.tts_provider_secret_key', '') as string
  }

  public GetPluginList(): string[] {
    const plugin_list = this._store.get('config.plugin_list', [])
    if (!Array.isArray(plugin_list)) {
      log.fatal('Plugin list is not an array', { plugin_list })
    }
    return plugin_list as string[]
  }

  public AddPlugin(plugin: string) {
    const plugin_list = this.GetPluginList()
    if (plugin_list.includes(plugin)) {
      log.error('Plugin already exists', { plugin })
      return
    }
    plugin_list.push(plugin)
    this._store.set('config.plugin_list', plugin_list)
  }

  public RemovePlugin(plugin: string) {
    const plugin_list = this.GetPluginList()
    if (!plugin_list.includes(plugin)) {
      log.error('Plugin not found', { plugin })
      return
    }
    const index = plugin_list.indexOf(plugin)
    if (index > -1) {
      plugin_list.splice(index, 1)
    }
    this._store.set('config.plugin_list', plugin_list)
  }

  public SetPluginList(plugin_list: string[]) {
    if (!Array.isArray(plugin_list)) {
      log.error('Plugin list is not an array', { plugin_list })
      return
    }
    this._store.set('config.plugin_list', plugin_list)
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
