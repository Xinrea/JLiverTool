import { renderContent } from '../common/content-render'
import { createMedal } from '../common/medal'

export function createDanmuEntry(special, medal, sender, content) {
  let danmuEntry = document.createElement('span')
  if (special) {
    danmuEntry.className = 'danmu_entry special'
  } else {
    danmuEntry.className = 'danmu_entry'
  }
  if (medal) {
    danmuEntry.appendChild(
      createMedal(medal.guardLevel, medal.name, medal.level)
    )
  }
  let danmuSender = document.createElement('span')
  danmuSender.className = 'sender'
  if (content) sender = sender + '：'
  danmuSender.innerText = sender
  danmuEntry.appendChild(danmuSender)
  if (content) {
    if (content.url) {
      let emojiSize = 30
      if (content.emoticon_unique.includes('official')) emojiSize = 6
      let danmuContent = document.createElement('span')
      danmuContent.className = 'content emoji'
      danmuContent.style.backgroundImage = `url(${content.url})`
      danmuContent.style.width = content.width / 2 + 'px'
      danmuContent.style.height = content.height / 2 + 'px'
      danmuEntry.appendChild(danmuContent)
    } else {
      danmuEntry.appendChild(renderContent(content))
    }
  }
  return danmuEntry
}

export function createEnterEntry(medal, sender) {
  return createDanmuEntry(false, medal, sender + ' 进入直播间', null)
}

export function createEffectEntry(content) {
  return createDanmuEntry(false, null, content, null)
}

export let giftCache = new Map()

export function createGiftEntry(id, g) {
  let medalInfo = {
    guardLevel: g.data.medal_info.guard_level,
    name: g.data.medal_info.medal_name,
    level: g.data.medal_info.medal_level,
  }
  if (medalInfo.level == 0) {
    medalInfo = null
  }
  let entry = doCreateGiftEntry(medalInfo, g.data.uname, g)
  giftCache.set(id, entry)
  return entry
}

export function createGuardEntry(g) {
  let medalInfo = null
  if (g.medal) {
    medalInfo = {
      guardLevel: g.medal.guard_level,
      name: g.medal.medal_name,
      level: g.medal.level,
    }
  }
  return doCreateGiftEntry(medalInfo, g.name, {
    data: {
      action: '开通',
      giftName: g.gift_name,
      isGuard: true,
      guardLevel: g.guard_level,
    },
  })
}

function doCreateGiftEntry(medal, sender, g) {
  let gift = g.data
  let danmuEntry = document.createElement('span')
  danmuEntry.className = 'danmu_entry special gift'
  if (medal) {
    danmuEntry.appendChild(
      createMedal(medal.guardLevel, medal.name, medal.level)
    )
  }
  let danmuSender = document.createElement('span')
  danmuSender.className = 'sender'
  danmuSender.innerText = sender
  danmuEntry.appendChild(danmuSender)
  // Content
  let giftAction = document.createElement('span')
  giftAction.className = 'action'
  giftAction.innerText = gift.action
  danmuEntry.appendChild(giftAction)
  let giftName = document.createElement('span')
  giftName.className = 'gift-name'
  giftName.innerText = gift.giftName
  danmuEntry.appendChild(giftName)
  let giftIcon = document.createElement('span')
  giftIcon.className = 'gift-icon'
  if (gift.isGuard) {
    giftIcon.style.backgroundImage = `var(--guard-level-${gift.guardLevel})`
  } else {
    giftIcon.style.backgroundImage = `url(${g.gif.gif})`
  }
  danmuEntry.appendChild(giftIcon)
  if (gift.num) {
    let giftNum = document.createElement('span')
    giftNum.className = 'gift-num'
    giftNum.innerText = `共${gift.num}个 | ￥${(gift.price * gift.num) / 1000}`
    danmuEntry.appendChild(giftNum)
    danmuEntry.setAttribute('gift-num', gift.num)
  }
  // Event
  // danmuEntry.onclick = function () {
  //   danmuEntry.classList.toggle('selected')
  //   if (danmuEntry.classList.contains('selected')) {
  //     if (appStatus.lastSelectedDanmu) {
  //       appStatus.lastSelectedDanmu.classList.remove('selected')
  //     }
  //     appStatus.lastSelectedDanmu = danmuEntry
  //   } else {
  //     appStatus.lastSelectedDanmu = null
  //   }
  //   appStatus.autoScroll = appStatus.lastSelectedDanmu == null
  // }
  return danmuEntry
}

const MockGuard = {
  sid: '1191b581-dde0-48dd-95a5-aaccb530981f',
  msg: {
    medal: {
      uid: 2237615,
      target_id: 4390795,
      medal_id: 38622,
      level: 1,
      medal_name: '赤樱',
      medal_color: 6067854,
      intimacy: 199,
      next_intimacy: 201,
      day_limit: 1500,
      medal_color_start: 6067854,
      medal_color_end: 6067854,
      medal_color_border: 6067854,
      is_lighted: 1,
      light_status: 1,
      wearing_status: 1,
      score: 199,
    },
    face: 'http://i2.hdslb.com/bfs/face/bbda0583aa73f50006945a5662a3bf3f0a902b85.jpg',
    name: '-密密酱-',
    gift_name: '舰长',
    guard_level: 3,
    price: 198000,
    timestamp: 1645970886,
  },
}
