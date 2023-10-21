import { contextBridge, ipcRenderer } from 'electron'
import { WindowType } from './lib/window_manager'
import JEvent from './lib/events'

export type JLiverAPI = {
  get: (key: string, d: any) => any
  set: (key: string, value: any) => void
  onDidChange: (key: string, callback: Function) => void
  invoke: (channel: string, ...args: any[]) => Promise<any>
  hideWindow: (wtype: WindowType) => Promise<void>
  showWindow: (wtype: WindowType) => Promise<void>
  send: (channel: string, ...args: any[]) => void
  register: (channel: JEvent, callback: Function) => void
}

// listeners keeps all registered callback in renderer process
let listeners: Map<string, Function[]> = new Map()
function registerListener(name: string) {
  console.log('registering listener', name)
  ipcRenderer.on(name, (_, arg) => {
    if (listeners[name]) {
      listeners[name].forEach((callback: Function) => {
        callback(arg)
      })
    }
  })
}

registerListener('blur')
registerListener('update-room')
registerListener('update-heat')
registerListener(JEvent[JEvent.EVENT_UPDATE_ONLINE])
registerListener('danmu')
registerListener('gift')
registerListener('guard')
registerListener('superchat')
registerListener('interact')
registerListener('entry-effect')
registerListener('reset')
registerListener('updateOpacity')
registerListener('updateWindowStatus')

// watcher keeps all registered onDidChange callback in renderer process
// and will be called when ipcMain send store-watch event
let watcher: Map<string, Function[]> = new Map()
ipcRenderer.invoke('store-register')
ipcRenderer.on('store-watch', (_, key, newValue) => {
  if (watcher[key]) {
    watcher[key].forEach((callback: Function) => {
      callback(newValue)
    })
  }
})

contextBridge.exposeInMainWorld('jliverAPI', {
  get: (key: string, d: any = null) => {
    return ipcRenderer.invoke('store-get', key, d)
  },
  set: (key: string, value: any) => {
    return ipcRenderer.invoke('store-set', key, value)
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
  hideWindow: (wtype: WindowType) => {
    return ipcRenderer.invoke('hideWindow', wtype)
  },
  showWindow: (wtype: WindowType) => {
    return ipcRenderer.invoke('showWindow', wtype)
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
})
