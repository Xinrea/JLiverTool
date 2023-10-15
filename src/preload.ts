import { contextBridge, ipcRenderer } from 'electron'
import { WindowType } from './lib/window_manager'
import JEvent from './lib/events'
import * as Store from 'electron-store'
const store = new Store()

let listeners = {}

function registerListener(name: string) {
  ipcRenderer.on(name, (_, arg) => {
    if (listeners[name]) {
      listeners[name](arg)
    }
  })
}

registerListener('blur')
registerListener('update-room')
registerListener('update-heat')
registerListener('update-online')
registerListener('danmu')
registerListener('gift')
registerListener('guard')
registerListener('superchat')
registerListener('interact')
registerListener('entry-effect')
registerListener('reset')
registerListener('updateOpacity')
registerListener('updateWindowStatus')

let watched = {}

ipcRenderer.on('store-watch', (_, key, newValue) => {
  if (watched[key]) {
    watched[key](newValue)
  }
})

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

contextBridge.exposeInMainWorld('jliverAPI', {
  get: (key: string, d: any) => {
    return store.get(key, d)
  },
  set: (key: string, value: any) => {
    store.set(key, value)
    ipcRenderer.send('store-watch', key, value)
  },
  onDidChange: (key: string, callback: Function) => {
    watched[key] = callback
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
  send: ipcRenderer.send,
  register: (channel: JEvent, callback: Function) => {
    listeners[JEvent[channel]] = callback
  },
})
