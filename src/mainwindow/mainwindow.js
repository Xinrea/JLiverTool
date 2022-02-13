let appStatus = {
  lastSelectedDanmu: null,
  numberDanmu: 0,
  isCoutingNew: false,
  newDanmuCount: 0,
  live: false
}

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

function showInput(e) {
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
document.getElementById('openSetting').onclick = (e) => {
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
    document.getElementById('titlebar').style.backgroundColor = '#fc3131'
  } else {
    document.getElementById('titlebar').style.backgroundColor = '#16161a'
    $onlineText.innerText = ''
  }
}

console.log('mainwindow.js loaded')
heatValue = document.getElementById('heatValue')
window.electron.onUpdate((arg) => {
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

window.electron.onDanmu((arg) => {
  // 过滤礼物弹幕
  if (arg.info[0][9] > 0) {
    return
  }
  // 判断是否为特殊身份
  let special = arg.info[2][2] > 0
  // 处理牌子
  let medalInfo = null
  if (arg.info[3].length > 0) {
    medalInfo = {
      guardLevel: arg.info[3][10],
      name: arg.info[3][1],
      level: arg.info[3][0]
    }
  }
  onReceiveNewDanmu(special, medalInfo, arg.info[2][1], arg.info[1])
})

window.electron.onInteract((arg) => {
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

window.electron.onHeat((arg) => {
  heatValue.innerText = arg
})

let $onlineText = document.getElementById('online')
window.electron.onOnline((arg) => {
  if (appStatus.live) {
    if (arg >= 9999) {
      $onlineText.innerText = '> 10000'
    } else {
      $onlineText.innerText = arg
    }
  }
})

window.electron.onEffect((arg) => {
  if (!window.electron.get('config.enableEnter', false)) return
  onReceiveEffect(arg)
})

let danmuArea = document.getElementById('danmu')
let indicator = document.getElementById('bottomIndicator')
let indicount = document.getElementById('danmuCount')
let autoScroll = true
let lastPosition = 0

window.electron.onReset(() => {
  $onlineText.innerText = ''
  danmuArea.innerHTML = ''
  autoScroll = true
  indicator.style.visibility = 'hidden'
  appStatus.newDanmuCount = 0
  appStatus.isCoutingNew = false
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
  appStatus.numberDanmu++
  if (appStatus.numberDanmu > 500) {
    appStatus.numberDanmu--
    danmuArea.removeChild(danmuArea.children[0])
  }
}

function onReceiveNewDanmu(special, medalInfo, sender, content) {
  cleanOldEntry()
  danmuArea.appendChild(createDanmuEntry(special, medalInfo, sender, content))
  scroll()
}

function onReceiveInteract(medalInfo, sender) {
  cleanOldEntry()
  danmuArea.appendChild(createEnterEntry(medalInfo, sender))
  scroll()
}

function onReceiveEffect(content) {
  cleanOldEntry()
  danmuArea.appendChild(createEffectEntry(content))
  scroll()
}

function init() {
  // Always on top
  if (window.electron.get('config.alwaysOnTop', false)) {
    window.electron.send('setAlwaysOnTop', true)
    document.getElementById('topButton').classList.toggle('enabled')
  }
  // Enter message
  if (window.electron.get('config.enableEnter', false)) {
    document.getElementById('enterButton').classList.toggle('enabled')
  }
  document.documentElement.style.setProperty(
    '--danmu-size',
    window.electron.get('config.fontSize', '14px')
  )
}

init()
