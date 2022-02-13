console.log('giftWindow.js loaded')

let giftMap = new Map()

let lastSelected = null

function hideWindow() {
  console.log('hideWindow')
  window.electron.send('hideGiftWindow')
}

document.getElementById('hideButton').onclick = hideWindow

window.electron.onGift((arg) => {
  console.log(arg)
  handleGift(arg.id, arg.msg)
})

window.electron.onGuard((arg) => {
  let entry = createGuardEntry(arg.id, arg.msg)
  panel.appendChild(entry)
  if (autoScroll) {
    panel.scrollTop = lastPosition = panel.scrollHeight - panel.clientHeight
  }
})

let panel = document.getElementById('giftpanel')
window.electron.onReset(() => {
  panel.innerHTML = ''
  giftMap = new Map()
  window.electron.send('reseted')
})

let autoScroll = true
let lastPosition = 0
panel.addEventListener('scroll', () => {
  if (Math.ceil(panel.scrollTop) == lastPosition) {
    // Auto scroll
    autoScroll = true
    return
  }
  // User scroll
  if (Math.ceil(panel.scrollTop) == panel.scrollHeight - panel.clientHeight) {
    autoScroll = true
  } else {
    autoScroll = false
  }
})

function handleGift(id, g) {
  if (g.data.coin_type !== 'gold') {
    g.data.price = 0
  }
  if (giftMap.has(id)) {
    let e = giftMap.get(id)
    let num = parseInt(e.getAttribute('gift.count'))
    e.querySelector('.gift-count').innerText = 'x' + (num + g.data.num)
    if (g.data.price > 0) {
      e.querySelector('.gift-price-value').innerText =
        '￥' + (g.data.price * (g.data.num + num)) / 1000
    }
    e.setAttribute('gift.count', g.data.num + num)
    return
  }
  entry = createGiftEntry(id, g)
  giftMap.set(id, entry)
  panel.appendChild(entry)
  if (autoScroll) {
    panel.scrollTop = lastPosition = panel.scrollHeight - panel.clientHeight
  }
}

function createGiftEntry(id, g) {
  let giftEntry = document.createElement('div')
  giftEntry.classList.add('gift-entry')
  if (g.data.price == 0) {
    giftEntry.classList.add('free')
  }
  let giftTitle = document.createElement('div')
  giftTitle.classList.add('gift-title')
  let giftSender = document.createElement('div')
  giftSender.classList.add('gift-sender')
  let senderAvator = document.createElement('div')
  senderAvator.classList.add('sender-avator')
  let senderAvatorImg = document.createElement('img')
  if (g.data.face == undefined) {
    console.log(g)
  }
  senderAvatorImg.src = g.data.face
  senderAvator.appendChild(senderAvatorImg)
  let senderName = document.createElement('div')
  senderName.classList.add('sender-name')
  senderName.innerText = g.data.uname
  giftSender.appendChild(senderAvator)
  if (g.data.medal_info.medal_name !== '') {
    let medal = createMedal(
      g.data.medal_info.guard_level,
      g.data.medal_info.medal_name,
      g.data.medal_info.medal_level
    )
    giftSender.appendChild(medal)
  }
  giftSender.appendChild(senderName)
  giftTitle.appendChild(giftSender)
  let giftTime = document.createElement('div')
  giftTime.classList.add('gift-time')
  giftTime.innerText = moment(g.data.timestamp * 1000).format(
    'YYYY/MM/DD HH:mm:ss'
  )
  giftTitle.appendChild(giftTime)
  giftEntry.appendChild(giftTitle)
  let giftContent = document.createElement('div')
  giftContent.classList.add('gift-content')
  let giftInfo = document.createElement('div')
  giftInfo.classList.add('gift-info')
  let giftAction = document.createElement('div')
  giftAction.classList.add('gift-action')
  giftAction.innerText = '投喂'
  let giftName = document.createElement('div')
  giftName.classList.add('gift-name')
  giftName.innerText = g.data.giftName
  let giftCount = document.createElement('div')
  giftCount.classList.add('gift-count')
  giftCount.innerText = 'x' + g.data.num
  let giftIcon = document.createElement('div')
  giftIcon.classList.add('gift-icon')
  giftIcon.style.backgroundImage = `url(${g.gif.png})`
  giftInfo.style.setProperty('--frame-number', g.gif.frame)
  giftInfo.appendChild(giftIcon)
  giftInfo.appendChild(giftAction)
  giftInfo.appendChild(giftName)
  giftInfo.appendChild(giftCount)
  giftContent.appendChild(giftInfo)
  let giftPrice = document.createElement('div')
  giftPrice.classList.add('gift-price')
  let giftPriceValue = document.createElement('div')
  giftPriceValue.classList.add('gift-price-value')
  giftEntry.setAttribute('gift.count', g.data.num)
  if (g.data.price > 0) {
    giftPriceValue.innerText = '￥' + (g.data.price * g.data.num) / 1000
  }
  giftPrice.appendChild(giftPriceValue)
  giftContent.appendChild(giftPrice)
  giftEntry.appendChild(giftContent)
  giftEntry.ondblclick = () => {
    if (giftEntry === lastSelected) {
      lastSelected = null
    }
    giftEntry.classList.toggle('flipped')
    setTimeout(() => {
      giftEntry.remove()
    }, 0.6 * 1000)
    giftMap.delete(id)
    window.electron.send('remove', {
      type: 'gifts',
      id: id
    })
  }
  giftEntry.onclick = () => {
    if (lastSelected) {
      lastSelected.classList.remove('selected')
    }
    if (lastSelected === giftEntry) {
      lastSelected = null
      return
    }
    lastSelected = giftEntry
    giftEntry.classList.add('selected')
  }
  return giftEntry
}

function createGuardEntry(id, g) {
  let giftEntry = document.createElement('div')
  giftEntry.classList.add('gift-entry')
  let giftTitle = document.createElement('div')
  giftTitle.classList.add('gift-title')
  let giftSender = document.createElement('div')
  giftSender.classList.add('gift-sender')
  let senderAvator = document.createElement('div')
  senderAvator.classList.add('sender-avator')
  let senderAvatorImg = document.createElement('img')
  senderAvatorImg.src = g.face
  senderAvator.appendChild(senderAvatorImg)
  let senderName = document.createElement('div')
  senderName.classList.add('sender-name')
  senderName.innerText = g.name
  giftSender.appendChild(senderAvator)
  if (g.medal) {
    let medal = createMedal(
      g.medal.guard_level,
      g.medal.medal_name,
      g.medal.level
    )
    giftSender.appendChild(medal)
  }
  giftSender.appendChild(senderName)
  giftTitle.appendChild(giftSender)
  let giftTime = document.createElement('div')
  giftTime.classList.add('gift-time')
  giftTime.innerText = moment(g.timestamp * 1000).format('YYYY/MM/DD HH:mm:ss')
  giftTitle.appendChild(giftTime)
  giftEntry.appendChild(giftTitle)
  let giftContent = document.createElement('div')
  giftContent.classList.add('gift-content')
  let giftInfo = document.createElement('div')
  giftInfo.classList.add('gift-info')
  let giftAction = document.createElement('div')
  giftAction.classList.add('gift-action')
  giftAction.innerText = ''
  let giftName = document.createElement('div')
  giftName.classList.add('gift-name')
  giftName.innerText = g.gift_name
  let giftCount = document.createElement('div')
  giftCount.classList.add('gift-count')
  giftCount.innerText = ''
  let giftIcon = document.createElement('div')
  giftIcon.classList.add('gift-icon')
  giftIcon.style.backgroundImage = `var(--guard-level-${g.guard_level})`
  giftInfo.style.setProperty('--frame-number', 1)
  giftInfo.appendChild(giftIcon)
  giftInfo.appendChild(giftAction)
  giftInfo.appendChild(giftName)
  giftInfo.appendChild(giftCount)
  giftContent.appendChild(giftInfo)
  let giftPrice = document.createElement('div')
  giftPrice.classList.add('gift-price')
  let giftPriceValue = document.createElement('div')
  giftPriceValue.classList.add('gift-price-value')
  giftPriceValue.innerText = '￥' + g.price / 1000
  giftPrice.appendChild(giftPriceValue)
  giftContent.appendChild(giftPrice)
  giftEntry.appendChild(giftContent)
  giftEntry.style.background = `var(--guard-gift-bg-${g.guard_level})`
  giftEntry.ondblclick = () => {
    if (giftEntry === lastSelected) {
      lastSelected = null
    }
    giftEntry.classList.toggle('flipped')
    setTimeout(() => {
      giftEntry.remove()
    }, 0.6 * 1000)
    window.electron.send('remove', {
      type: 'guards',
      id: id
    })
  }
  giftEntry.onclick = () => {
    if (lastSelected) {
      lastSelected.classList.remove('selected')
    }
    if (lastSelected === giftEntry) {
      lastSelected = null
      return
    }
    lastSelected = giftEntry
    giftEntry.classList.add('selected')
  }
  return giftEntry
}

document.getElementById('freeButton').onclick = toggleFreeGift

function toggleFreeGift(e) {
  let passFreeGift = window.electron.get('config.passFreeGift', true)
  window.electron.set('config.passFreeGift', !passFreeGift)
  if (!passFreeGift) {
    document.documentElement.style.setProperty('--free-gift', 'none')
  } else {
    document.documentElement.style.setProperty('--free-gift', 'flex')
  }
  e.target.classList.toggle('enabled')
}

function init() {
  if (window.electron.get('config.passFreeGift', true)) {
    document.getElementById('freeButton').classList.add('enabled')
    document.documentElement.style.setProperty('--free-gift', 'none')
  } else {
    document.documentElement.style.setProperty('--free-gift', 'flex')
  }
}

init()
