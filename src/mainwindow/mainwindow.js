let appStatus = {
  lastSelectedDanmu: null,
  isCoutingNew: false,
  newDanmuCount: 0,
  live: false
}

let windowStatus = {
  gift: false,
  superchat: false
}

window.electron.register('updateWindowStatus', (arg) => {
  windowStatus = arg
  console.log(windowStatus)
})

function showMenu(e) {
  if (document.getElementById('dropdown-content').style.display == 'block') {
    return
  }
  document.getElementById('dropdown-content').style.display = 'block'
  e.stopPropagation()
}

function hideMenu() {
  document.getElementById('dropdown-content').style.display = 'none'
}

function hideCover() {
  document.getElementById('cover').style.visibility = 'hidden'
}

function showCover() {
  document.getElementById('cover').style.visibility = 'visible'
}

function showInput() {
  document.getElementById('roomInputPanel').style.visibility = 'visible'
}

function hideInput() {
  document.getElementById('roomInputPanel').style.visibility = 'hidden'
}

function toggleAlwaysOnTop() {
  let alwaysOnTop = window.electron.get('config.alwaysOnTop', false)
  window.electron.set('config.alwaysOnTop', !alwaysOnTop)
  window.electron.send('setAlwaysOnTop', !alwaysOnTop)
  document.getElementById('topButton').classList.toggle('enabled')
}

function toggleEnterMessage() {
  let enableEnter = window.electron.get('config.enableEnter', false)
  window.electron.set('config.enableEnter', !enableEnter)
  document.getElementById('enterButton').classList.toggle('enabled')
}

document.getElementById('menubutton').onclick = (e) => {
  showMenu(e)
  showCover()
}

document.getElementById('switchTheme').onclick = ()=>{
  window.electron.send('theme:switch')
}

let $opcaitySetting = document.getElementById('opacity-setting')
$opcaitySetting.onchange = function () {
  window.electron.set('config.opacity', this.value)
  document.documentElement.style.setProperty('--global-opacity', this.value)
  window.electron.send('updateOpacity')
}

function updateOpacity() {
  let opacity = window.electron.get('config.opacity', 1)
  document.documentElement.style.setProperty('--global-opacity', opacity)
  $opcaitySetting.value = opacity
}

document.getElementById('smallF').onclick = () => {
  document.documentElement.style.setProperty('--danmu-size', '14px')
  window.electron.set('config.fontSize', '14px')
}
document.getElementById('middleF').onclick = () => {
  document.documentElement.style.setProperty('--danmu-size', '18px')
  window.electron.set('config.fontSize', '18px')
}
document.getElementById('largeF').onclick = () => {
  document.documentElement.style.setProperty('--danmu-size', '22px')
  window.electron.set('config.fontSize', '22px')
}
document.getElementById('exportGift').onclick = () => {
  window.electron.send('exportGift')
  hideCover()
  hideMenu()
}

let replaceIndex = 0
document.getElementById('fullModeButton').onclick = () => {
  let fullMode = window.electron.get('config.fullMode', false)
  window.electron.set('config.fullMode', !fullMode)
  document.getElementById('fullModeButton').classList.toggle('enabled')
  if (!fullMode) {
    // Enabled Full Mode
    replaceIndex = 0
  }
}

function updateMedalStatus() {
  if (window.electron.get('config.medalDisplay', false)) {
    document.documentElement.style.setProperty(
      '--medal-display',
      'inline-block'
    )
  } else {
    document.documentElement.style.setProperty('--medal-display', 'none')
  }
}
document.getElementById('medalButton').onclick = () => {
  let medalDisplay = window.electron.get('config.medalDisplay', false)
  window.electron.set('config.medalDisplay', !medalDisplay)
  document.getElementById('medalButton').classList.toggle('enabled')
  updateMedalStatus()
}
document.getElementById('topButton').onclick = toggleAlwaysOnTop
document.getElementById('enterButton').onclick = toggleEnterMessage
document.getElementById('openGiftItem').onclick = function () {
  window.electron.send('showGiftWindow')
  hideCover()
  hideMenu()
}
document.getElementById('openSuperChatItem').onclick = function () {
  window.electron.send('showSuperchatWindow')
  hideCover()
  hideMenu()
}
document.getElementById('quitItem').onclick = function () {
  window.electron.send('quit')
}
document.getElementById('openBrowser').onclick = () => {
  window.electron.send('openBrowser')
  hideCover()
  hideMenu()
}
document.getElementById('openSetting').onclick = () => {
  showCover()
  showInput()
  hideMenu()
}
document.getElementById('cover').onclick = () => {
  hideMenu()
  hideInput()
  hideCover()
}
document.getElementById('roomInputPanel').onclick = (e) => {
  e.stopPropagation()
}
document.getElementById('enterRoom').onclick = function () {
  window.electron.send('setRoom', document.getElementById('roomInput').value)
  hideInput()
  hideCover()
}

function updateLiveStatus(isLive) {
  appStatus.live = isLive
  if (isLive) {
    document.getElementById('titlebar').style.backgroundColor = '#fc3131be'
  } else {
    document.getElementById('titlebar').style.backgroundColor = 'var(--titlebar-color)'
    $onlineText.innerText = ''
  }
}

console.log('mainwindow.js loaded')
heatValue = document.getElementById('heatValue')
window.electron.register('updateroom', (arg) => {
  if (!arg) {
    return
  }
  if (arg.live_status == 1) {
    updateLiveStatus(true)
  } else {
    updateLiveStatus(false)
  }
  document.getElementById('livetitle').innerText = arg.title
  if (heatValue.innerText == '') {
    heatValue.innerText = arg.online
  }
})

window.electron.register('danmu', (arg) => {
  // ??????????????????
  if (arg.info[0][9] > 0) {
    return
  }
  // ???????????????????????????
  let special = arg.info[2][2] > 0
  // ????????????
  let medalInfo = null
  if (arg.info[3].length > 0) {
    medalInfo = {
      guardLevel: arg.info[3][10],
      name: arg.info[3][1],
      level: arg.info[3][0]
    }
  }
  if (arg.info[0][12] > 0) {
    // Emoji Danmu
    onReceiveNewDanmu(special, medalInfo, arg.info[2][1], arg.info[0][13])
  } else {
    onReceiveNewDanmu(special, medalInfo, arg.info[2][1], arg.info[1])
  }
})

window.electron.register('interact', (arg) => {
  if (!window.electron.get('config.enableEnter', false)) return
  let medalInfo = null
  if (arg.data.fans_medal.level > 0) {
    medalInfo = {
      guardLevel: arg.data.fans_medal.guard_level,
      name: arg.data.fans_medal.medal_name,
      level: arg.data.fans_medal.medal_level
    }
  }
  onReceiveInteract(medalInfo, arg.data.uname)
})

window.electron.register('updateheat', (arg) => {
  heatValue.innerText = arg
})

let $onlineText = document.getElementById('online')
window.electron.register('updateonline', (arg) => {
  if (appStatus.live) {
    if (arg >= 9999) {
      $onlineText.innerText = '> 10000'
    } else {
      $onlineText.innerText = arg
    }
  }
})

window.electron.register('entry_effect', (arg) => {
  if (!window.electron.get('config.enableEnter', false)) return
  onReceiveEffect(arg)
})

window.electron.register('gift', (arg) => {
  if (!windowStatus.gift) {
    onReceiveGift(arg.id, arg.msg)
  }
})

window.electron.register('guard', (arg) => {
  if (!windowStatus.gift) {
    console.log(arg)
    onReceiveGuard(arg.id, arg.msg)
  }
})

window.electron.register('superchat', (arg) => {
  if (!windowStatus.superchat) {
    onReceiveSuperchat(arg.id, arg.msg)
  }
})

let danmuArea = document.getElementById('danmu')
let indicator = document.getElementById('bottomIndicator')
let indicount = document.getElementById('danmuCount')
let autoScroll = true
let lastPosition = 0

window.electron.register('reset', () => {
  $onlineText.innerText = ''
  danmuArea.innerHTML = ''
  autoScroll = true
  indicator.style.visibility = 'hidden'
  appStatus.newDanmuCount = 0
  appStatus.isCoutingNew = false
  // Reset Full Mode Related
  replaceIndex = 0
  // Make Sure All Data Reseted
  window.electron.send('reseted')
})

danmuArea.addEventListener('scroll', () => {
  if (Math.ceil(danmuArea.scrollTop) == lastPosition) {
    // Auto scroll
    autoScroll = true
    return
  }
  // User scroll
  if (
    Math.ceil(danmuArea.scrollTop) ==
    danmuArea.scrollHeight - danmuArea.clientHeight
  ) {
    autoScroll = true
    indicator.style.visibility = 'hidden'
    appStatus.newDanmuCount = 0
    appStatus.isCoutingNew = false
  } else {
    autoScroll = false
  }
})

function scroll() {
  if (autoScroll) {
    danmuArea.scrollTop = lastPosition =
      danmuArea.scrollHeight - danmuArea.clientHeight
  }
  if (!autoScroll) {
    appStatus.isCoutingNew = true
    if (appStatus.newDanmuCount > 0) {
      indicator.style.visibility = 'visible'
      indicount.innerText = appStatus.newDanmuCount
    }
  } else {
    indicator.style.visibility = 'hidden'
    appStatus.newDanmuCount = 0
    appStatus.isCoutingNew = false
  }
}

indicator.onclick = () => {
  danmuArea.scrollTop = lastPosition =
    danmuArea.scrollHeight - danmuArea.clientHeight
  indicator.style.visibility = 'hidden'
  appStatus.newDanmuCount = 0
  appStatus.isCoutingNew = false
}

function cleanOldEntry() {
  if (appStatus.isCoutingNew) appStatus.newDanmuCount++
  if (danmuArea.children.length > 1000) {
    danmuArea.removeChild(danmuArea.children[0])
  }
}

function onReceiveNewDanmu(special, medalInfo, sender, content) {
  cleanOldEntry()
  let $newEntry = createDanmuEntry(special, medalInfo, sender, content)
  danmuArea.appendChild($newEntry)
  if (window.electron.get('config.fullMode', false)) {
    if (danmuArea.scrollHeight > danmuArea.clientHeight) {
      replaceIndex++
      replaceIndex = replaceIndex % (danmuArea.children.length - 1)
      danmuArea.children[replaceIndex].replaceWith($newEntry)
      // When Window Height Decrease Too Much
      while (danmuArea.scrollHeight > danmuArea.clientHeight) {
        danmuArea.firstChild.remove()
      }
    }
  }
  scroll()
}

function onReceiveInteract(medalInfo, sender) {
  cleanOldEntry()
  let $newEntry = createEnterEntry(medalInfo, sender)
  danmuArea.appendChild($newEntry)
  if (window.electron.get('config.fullMode', false)) {
    if (danmuArea.scrollHeight > danmuArea.clientHeight) {
      replaceIndex++
      replaceIndex = replaceIndex % (danmuArea.children.length - 1)
      danmuArea.children[replaceIndex].replaceWith($newEntry)
      // When Window Height Decrease Too Much
      while (danmuArea.scrollHeight > danmuArea.clientHeight) {
        danmuArea.firstChild.remove()
      }
    }
  }
  scroll()
}

function onReceiveEffect(content) {
  cleanOldEntry()
  let $newEntry = createEffectEntry(content)
  danmuArea.appendChild($newEntry)
  if (window.electron.get('config.fullMode', false)) {
    if (danmuArea.scrollHeight > danmuArea.clientHeight) {
      replaceIndex++
      replaceIndex = replaceIndex % (danmuArea.children.length - 1)
      danmuArea.children[replaceIndex].replaceWith($newEntry)
      // When Window Height Decrease Too Much
      while (danmuArea.scrollHeight > danmuArea.clientHeight) {
        danmuArea.firstChild.remove()
      }
    }
  }
  scroll()
}

function onReceiveGift(id, msg) {
  if (window.electron.get('config.passFreeGift', true)) {
    if (msg.data.coin_type !== 'gold') {
      return
    }
  }
  if (giftCache.has(id)) {
    let old = giftCache.get(id)
    let oldNum = parseInt(old.getAttribute('gift-num'))
    let newNum = oldNum + parseInt(msg.data.num)
    old.querySelector('.gift-num').innerText = `???${newNum}???`
    old.setAttribute('gift-num', newNum)
    return
  }
  cleanOldEntry()
  let $newEntry = createGiftEntry(id, msg)
  danmuArea.appendChild($newEntry)
  if (window.electron.get('config.fullMode', false)) {
    if (danmuArea.scrollHeight > danmuArea.clientHeight) {
      replaceIndex++
      replaceIndex = replaceIndex % (danmuArea.children.length - 1)
      danmuArea.children[replaceIndex].replaceWith($newEntry)
      // When Window Height Decrease Too Much
      while (danmuArea.scrollHeight > danmuArea.clientHeight) {
        danmuArea.firstChild.remove()
      }
    }
  }
  scroll()
}

function onReceiveGuard(id, msg) {
  cleanOldEntry()
  let $newEntry = createGuardEntry(msg)
  danmuArea.appendChild($newEntry)
  if (window.electron.get('config.fullMode', false)) {
    if (danmuArea.scrollHeight > danmuArea.clientHeight) {
      replaceIndex++
      replaceIndex = replaceIndex % (danmuArea.children.length - 1)
      danmuArea.children[replaceIndex].replaceWith($newEntry)
      // When Window Height Decrease Too Much
      while (danmuArea.scrollHeight > danmuArea.clientHeight) {
        danmuArea.firstChild.remove()
      }
    }
  }
  scroll()
}

function onReceiveSuperchat(id, msg) {
  cleanOldEntry()
  let $newEntry = createSuperchatEntry(id, msg, false)
  danmuArea.appendChild($newEntry)
  if (window.electron.get('config.fullMode', false)) {
    if (danmuArea.scrollHeight > danmuArea.clientHeight) {
      replaceIndex++
      replaceIndex = replaceIndex % (danmuArea.children.length - 1)
      danmuArea.children[replaceIndex].replaceWith($newEntry)
      // When Window Height Decrease Too Much
      while (danmuArea.scrollHeight > danmuArea.clientHeight) {
        danmuArea.firstChild.remove()
      }
    }
  }
  scroll()
}

function init() {
  // Full Mode
  if (window.electron.get('config.fullMode', false)) {
    document.getElementById('fullModeButton').classList.toggle('enabled')
  }
  if (window.electron.get('config.medalDisplay', false)) {
    document.getElementById('medalButton').classList.toggle('enabled')
    updateMedalStatus()
  }
  // Always on top
  if (window.electron.get('config.alwaysOnTop', false)) {
    window.electron.send('setAlwaysOnTop', true)
    document.getElementById('topButton').classList.toggle('enabled')
  }
  // Enter message
  if (window.electron.get('config.enableEnter', false)) {
    document.getElementById('enterButton').classList.toggle('enabled')
  }
  updateOpacity()

  document.documentElement.style.setProperty(
    '--danmu-size',
    window.electron.get('config.fontSize', '14px')
  )
}

init()
