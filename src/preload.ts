const { contextBridge, ipcRenderer } = require('electron')
const Store = require('electron-store')
const store = new Store()

let listeners = {}

function registerListener(name) {
  ipcRenderer.on(name, (event, arg) => {
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

ipcRenderer.on('store-watch', (event, key, newValue) => {
  if (watched[key]) {
    watched[key](newValue)
  }
})

contextBridge.exposeInMainWorld('electron', {
  get: (key, d) => {
    return store.get(key, d)
  },
  set: (key, value) => {
    store.set(key, value)
    ipcRenderer.send('store-watch', key, value)
  },
  onDidChange: (key, callback) => {
    watched[key] = callback
  },
  invoke: (channel, ...args) => {
    return ipcRenderer.invoke(channel, ...args)
  },
  send: ipcRenderer.send,
  register: (name, callback) => {
    listeners[name] = callback
  },
})
