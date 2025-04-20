import { contextBridge, ipcRenderer } from 'electron'
import JEvent from './lib/events'
import GetInfoResponse from './lib/bilibili/api/room/get_info'
import UserInfoResponse from './lib/bilibili/api/user/user_info'

export type JLiverAPI = {
  register: (channel: JEvent, callback: Function) => void
  user: {
    info: (user_id: number) => Promise<UserInfoResponse>
  }
  room: {
    info: (room_id: number) => Promise<GetInfoResponse>
  }
  util: {
    openUrl: (url: string) => Promise<any>
    fonts: () => Promise<any>
    setClipboard: (text: string) => Promise<any>
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
  user: {
    info: (user_id: number) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_GET_USER_INFO], user_id)
    },
  },
  room: {
    info: (room_id: number) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_GET_ROOM_INFO], room_id)
    },
  },
  util: {
    openUrl: (url: string) => {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_OPEN_URL], url)
    },
    fonts() {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_GET_FONT_LIST])
    },
    setClipboard(text: string) {
      return ipcRenderer.invoke(JEvent[JEvent.INVOKE_SET_CLIPBOARD], text)
    },
  },
})
