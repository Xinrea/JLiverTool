import * as Store from 'electron-store'
import { Cookies } from './types'
import { WindowType } from './window_manager'

export type WindowSetting = {
  pos: [number, number]
  size: [number, number] | null
}

const DEFAULT_WINDOWSIZE = new Map()
DEFAULT_WINDOWSIZE[WindowType.WMAIN] = [400, 800]
DEFAULT_WINDOWSIZE[WindowType.WGIFT] = [400, 300]
DEFAULT_WINDOWSIZE[WindowType.WSUPERCHAT] = [400, 300]
DEFAULT_WINDOWSIZE[WindowType.WSETTING] = [400, 300]

export class ConfigStore {
  private _store: Store

  constructor() {
    Store.initRenderer()
    this._store = new Store()
  }

  public get Cookies(): Cookies {
    return this._store.get('config.cookies', new Cookies()) as Cookies
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

  public GetWindowCachedSetting(wtype: WindowType): WindowSetting {
    let setting = this._store.get(`config.window.${wtype}`, null)
    if (setting === null) {
      setting = {
        pos: null,
        size: DEFAULT_WINDOWSIZE[wtype],
      }
    }
    return setting as WindowSetting
  }
  public UpdateWindowCachedSetting(wtype: WindowType, setting: WindowSetting) {
    this._store.set(`config.window.${wtype}`, setting)
  }
}

export default ConfigStore
