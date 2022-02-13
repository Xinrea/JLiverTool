const { contextBridge, ipcRenderer } = require('electron')
const Store = require('electron-store')
const store = new Store()

let updateListener
let danmuListener
let heatListener
let giftListener
let guardListener
let interactListener
let effectListener
let resetListener
let onlineListener
let superchatListener

ipcRenderer.on('updateroom', (event, arg) => {
  if (updateListener) {
    updateListener(arg)
  }
})

ipcRenderer.on('updateheat', (event, arg) => {
  if (heatListener) {
    heatListener(arg)
  }
})

ipcRenderer.on('updateonline', (event, arg) => {
  if (onlineListener) {
    onlineListener(arg)
  }
})

ipcRenderer.on('danmu', (event, arg) => {
  if (danmuListener) {
    danmuListener(arg)
  }
})

ipcRenderer.on('gift', (event, arg) => {
  if (giftListener) {
    giftListener(arg)
  }
})

ipcRenderer.on('guard', (event, arg) => {
  if (guardListener) {
    guardListener(arg)
  }
})

ipcRenderer.on('superchat', (event, arg) => {
  if (superchatListener) {
    superchatListener(arg)
  }
})

ipcRenderer.on('interact', (event, arg) => {
  if (interactListener) {
    interactListener(arg)
  }
})

ipcRenderer.on('entry_effect', (event, arg) => {
  if (effectListener) {
    effectListener(arg)
  }
})

ipcRenderer.on('reset', (event, arg) => {
  if (resetListener) {
    resetListener(arg)
  }
})

contextBridge.exposeInMainWorld('electron', {
  get: (key, d) => {
    return store.get(key, d)
  },
  set: (key, value) => {
    store.set(key, value)
  },
  send: ipcRenderer.send,
  onUpdate: (callback) => (updateListener = callback),
  onOnline: (callback) => (onlineListener = callback),
  onDanmu: (callback) => (danmuListener = callback),
  onHeat: (callback) => (heatListener = callback),
  onGift: (callback) => (giftListener = callback),
  onGuard: (callback) => (guardListener = callback),
  onSuperchat: (callback) => (superchatListener = callback),
  onInteract: (callback) => (interactListener = callback),
  onEffect: (callback) => (effectListener = callback),
  onReset: (callback) => (resetListener = callback)
})
