import { contextBridge, ipcRenderer } from 'electron'
import { DefaultRoomID, RoomID, WindowType, typecast } from './lib/types'
import JEvent from './lib/events'
import RoomInitResponse from './lib/bilibili/api/room/room_init'
import GetInfoResponse from './lib/bilibili/api/room/get_info'
import UserInfoResponse from './lib/bilibili/api/user/user_info'
import { GiftInitData, GiftMessage, SuperChatMessage } from './lib/messages'
import { GetGoalsResponse } from './lib/afdian/afdianapi'
import StartLiveResponse from './lib/bilibili/api/room/start_live'
import StopLiveResponse from './lib/bilibili/api/room/stop_live'
import GetOnlineGoldRankResponse from './lib/bilibili/api/room/get_online_gold_rank'

export type JLiverAPI = {
  get: (key: string, d: any) => any
  set: (key: string, value: any) => void
  onDidChange: (key: string, callback: Function) => void
  invoke: (channel: string, ...args: any[]) => Promise<any>
  send: (channel: string, ...args: any[]) => void
  register: (channel: JEvent, callback: Function) => void
  window: {
    hide: (window_type: WindowType) => void
    show: (window_type: WindowType) => void
    minimize: (window_type: WindowType) => void
    alwaysOnTop: (window_type: WindowType, value: boolean) => void
    minimizable: (window_type: WindowType, value: boolean) => void
    windowDetail: (uid: number) => void
  }
  app: {
    quit: () => void
  }
  qr: {
    get: () => Promise<any>
    update: (key: string) => Promise<any>
  }
  user: {
    logout: () => Promise<any>
    info: (user_id: number) => Promise<UserInfoResponse>
  }
  room: {
    info: (room_id: number) => Promise<GetInfoResponse>
  }
  config: {
    room: () => Promise<RoomID>
  }
  backend: {
    updateRoom: (room: RoomID) => Promise<void>
    setRoomTitle: (title: string) => Promise<void>
    getInitGifts: () => Promise<GiftInitData>
    getInitSuperChats: () => Promise<SuperChatMessage[]>
    removeGiftEntry: (type: string, id: string) => Promise<void>
    clearGifts: () => Promise<void>
    clearSuperChats: () => Promise<void>
    callCommand: (command: string) => Promise<void>
    startLive: (area_v2: string) => Promise<StartLiveResponse>
    stopLive: () => Promise<StopLiveResponse>
    getRankList: (
      page: number,
      page_size: number
    ) => Promise<GetOnlineGoldRankResponse>
  },
  plugin: {
    invokePluginWindow: (plugin_id: string) => void,
    removePlugin: (plugin_id: string) => void,
    addPlugin: (plugin_id: string) => void,
    togglePlugin: (plugin_id: string) => void,
  },
  util: {
    openUrl: (url: string) => Promise<any>
    openLogDir: () => Promise<any>
    fonts: () => Promise<any>
    version: () => Promise<string>
    latestRelease: () => Promise<any>
    setClipboard: (text: string) => Promise<any>
    getGoals: () => Promise<GetGoalsResponse>
  }
}

// listeners keeps all registered callback in renderer process
let listeners: Map<string, Function[]> = new Map()

function registerListener(event: JEvent) {
  const eventName = JEvent[event]
  console.log('registering listener', eventName)
  ipcRenderer.on(eventName, (_, arg) => {
    if (listeners[eventName]) {
      listeners[eventName].forEach((callback: Function) => {
        callback(arg)
      })
    }
  })
}

registerListener(JEvent.EVENT_UPDATE_ROOM)
registerListener(JEvent.EVENT_UPDATE_ONLINE)
registerListener(JEvent.EVENT_NEW_DANMU)
registerListener(JEvent.EVENT_NEW_GIFT)
registerListener(JEvent.EVENT_NEW_GUARD)
registerListener(JEvent.EVENT_NEW_SUPER_CHAT)
registerListener(JEvent.EVENT_NEW_INTERACT)
registerListener(JEvent.EVENT_NEW_ENTRY_EFFECT)
registerListener(JEvent.EVENT_WINDOW_BLUR)
registerListener(JEvent.EVENT_WINDOW_FOCUS)
registerListener(JEvent.EVENT_STORE_WATCH)
registerListener(JEvent.EVENT_LOG)
registerListener(JEvent.EVENT_DETAIL_UPDATE)

// watcher keeps all registered onDidChange callback in renderer process
// and will be called when ipcMain send store-watch event
let watcher: Map<string, Function[]> = new Map()
void ipcRenderer.invoke(JEvent[JEvent.INVOKE_STORE_REGISTER])
ipcRenderer.on(JEvent[JEvent.EVENT_STORE_WATCH], (_, key, newValue) => {
  if (watcher[key]) {
    watcher[key].forEach((callback: Function) => {
      callback(newValue)
    })
  }
})

contextBridge.exposeInMainWorld('jliverAPI', {
  get: (key: string, d: any = null) => {
    return ipcRenderer.invoke(JEvent[JEvent.INVOKE_STORE_GET], key, d)
  },
  set: (key: string, value: any) => {
    return ipcRenderer.invoke(JEvent[JEvent.INVOKE_STORE_SET], key, value)
  },
  onDidChange: (key: string, callback: Function) => {
    if (!watcher[key]) {
      watcher[key] = []
    }
    watcher[key].push(callback)
  },
  //TODO this should be removed after all channel wrapped in function
  invoke: (channel: string, ...args: any[]) => {
    return ipcRenderer.invoke(channel, ...args)
  },
  window: {
    hide: (window_type: WindowType) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_WINDOW_HIDE], window_type)
    },
    show: (window_type: WindowType) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_WINDOW_SHOW], window_type)
    },
    minimize: (window_type: WindowType) => {
      return ipcRenderer.invoke(
        JEvent[JEvent.INVOKE_WINDOW_MINIMIZE],
        window_type
      )
    },
    alwaysOnTop: (window_type: WindowType, value: boolean) => {
      return ipcRenderer.invoke(
        JEvent[JEvent.INVOKE_WINDOW_ALWAYS_ON_TOP],
        window_type,
        value
      )
    },
    minimizable: (window_type: WindowType, value: boolean) => {
      return ipcRenderer.invoke(
        JEvent[JEvent.INVOKE_WINDOW_MINIMIZABLE],
        window_type,
        value
      )
    },
    windowDetail: (uid: number) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_WINDOW_DETAIL], uid)
    },
  },
  app: {
    quit: () => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_APP_QUIT])
    },
  },
  //TODO this should be removed after all channel wrapped in function
  send: ipcRenderer.send,
  register: (channel: JEvent, callback: Function) => {
    if (JEvent[channel] === undefined) {
      console.log('invalid channel', channel)
      return
    }
    if (!listeners[JEvent[channel]]) {
      listeners[JEvent[channel]] = []
    }
    listeners[JEvent[channel]].push(callback)
  },
  qr: {
    get: () => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_QR_CODE])
    },
    update: (key: string) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_QR_CODE_UPDATE], key)
    },
  },
  user: {
    logout: () => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_LOGOUT])
    },
    info: (user_id: number) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_GET_USER_INFO], user_id)
    },
  },
  room: {
    info: (room_id: number) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_GET_ROOM_INFO], room_id)
    },
  },
  backend: {
    updateRoom: (room: RoomID) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_UPDATE_ROOM], room)
    },
    setRoomTitle: (title: string) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_SET_ROOM_TITLE], title)
    },
    getInitGifts: () => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_GET_INIT_GIFTS])
    },
    getInitSuperChats: () => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_GET_INIT_SUPERCHATS])
    },
    removeGiftEntry: (type: string, id: string) => {
      return ipcRenderer.invoke(
        JEvent[JEvent.INVOKE_REMOVE_GIFT_ENTRY],
        type,
        id
      )
    },
    clearGifts: () => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_CLEAR_GIFTS])
    },
    clearSuperChats: () => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_CLEAR_SUPERCHATS])
    },
    callCommand: (command: string) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_CALL_COMMAND], command)
    },
    startLive: (area_v2: string) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_START_LIVE], area_v2)
    },
    stopLive: () => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_STOP_LIVE])
    },
    getRankList: (page: number, page_size: number) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_GET_RANK], page, page_size)
    },
  },
  config: {
    room: async () => {
      const room = await ipcRenderer.invoke(
        JEvent[JEvent.INVOKE_STORE_GET],
        'config.room',
        DefaultRoomID
      )
      if (
        !room.hasOwnProperty('short_id') ||
        !room.hasOwnProperty('room_id') ||
        !room.hasOwnProperty('owner_uid')
      ) {
        return DefaultRoomID
      }
      // using this way to keep object function
      return typecast(RoomID, room)
    },
  },
  util: {
    openUrl: (url: string) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_OPEN_URL], url)
    },
    openLogDir: () => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_OPEN_LOG_DIR])
    },
    fonts() {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_GET_FONT_LIST])
    },
    version() {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_GET_VERSION])
    },
    latestRelease() {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_GET_LATEST_RELEASE])
    },
    setClipboard(text: string) {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_SET_CLIPBOARD], text)
    },
    getGoals() {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_GET_GOALS])
    },
  },
})
