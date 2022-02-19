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

registerListener('updateroom')
registerListener('updateheat')
registerListener('updateonline')
registerListener('danmu')
registerListener('gift')
registerListener('guard')
registerListener('superchat')
registerListener('interact')
registerListener('entry_effect')
registerListener('reset')
registerListener('updateOpacity')

contextBridge.exposeInMainWorld('electron', {
  get: (key, d) => {
    return store.get(key, d)
  },
  set: (key, value) => {
    store.set(key, value)
  },
  send: ipcRenderer.send,
  register: (name, callback) => {
    listeners[name] = callback
  }
})
