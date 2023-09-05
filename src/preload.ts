import { contextBridge, ipcRenderer } from 'electron'
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

contextBridge.exposeInMainWorld('electron', {
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
  invoke: (channel: string, ...args: any[]) => {
    return ipcRenderer.invoke(channel, ...args)
  },
  send: ipcRenderer.send,
  register: (name: string, callback: Function) => {
    listeners[name] = callback
  },
})
