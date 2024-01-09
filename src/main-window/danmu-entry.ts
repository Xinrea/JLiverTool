import { renderContent } from '../lib/common/content-render'
import { createMedal } from '../lib/common/medal'
import JEvent from '../lib/events'
import { GiftMessage, GuardMessage, InteractMessage } from '../lib/messages'
import { EmojiContent, MedalInfo, Sender } from '../lib/types'
import { levelToIconURL, levelToName } from '../lib/utils'

export function createDanmuEntry(
  side_index: number,
  special: boolean,
  medal: MedalInfo,
  sender: Sender,
  content: string,
  emoji_content: EmojiContent
) {
  const danmuEntry = document.createElement('span')
  if (special) {
    danmuEntry.className = 'danmu_entry special'
  } else {
    danmuEntry.className = 'danmu_entry'
  }

  if (side_index >= 0) {
    danmuEntry.classList.add('side' + side_index)
  }

  // might be gray medal, need to check color
  if (medal && medal.is_lighted) {
    danmuEntry.appendChild(createMedal(medal))
  }
  const danmuSender = document.createElement('span')
  danmuSender.className = 'sender'
  if (content) sender.uname = sender.uname + '：'
  danmuSender.innerText = sender.uname
  danmuEntry.appendChild(danmuSender)
  if (content) {
    if (emoji_content) {
      const danmuContent = document.createElement('span')
      const ratio = emoji_content.width / emoji_content.height
      danmuContent.className = 'content emoji'
      danmuContent.style.backgroundImage = `url(${emoji_content.url})`
      danmuContent.style.width = `calc((var(--danmu-size) + 32px) * ${ratio})`
      danmuContent.style.height = 'calc(var(--danmu-size) + 32px)'
      danmuEntry.appendChild(danmuContent)
    } else {
      danmuEntry.appendChild(renderContent(content))
    }
  }

  // add dbclick event
  danmuEntry.addEventListener('dblclick', () => {
    window.jliverAPI.window.windowDetail(sender.uid)
  })
  return danmuEntry
}

export function createInteractEntry(msg: InteractMessage) {
  return doCreateInteractEntry(msg)
}

export function createEffectEntry(content: string) {
  // TODO: need a better way to handle this
  const fake_sender = new Sender()
  fake_sender.uname = content
  return createDanmuEntry(-1, false, null, fake_sender, null, null)
}

export const giftCache = new Map()

export function createGiftEntry(gift: GiftMessage) {
  const entry = doCreateGiftEntry(gift)
  giftCache.set(gift.id, entry)
  return entry
}

export function createGuardEntry(g: GuardMessage) {
  const entry = doCreateGuardEntry(g)
  // TODO handle guard message
  return entry
}

function doCreateInteractEntry(msg: InteractMessage) {
  const danmuEntry = document.createElement('span')
  danmuEntry.className = 'danmu_entry interact'
  // check medal
  if (msg.sender.medal_info && msg.sender.medal_info.is_lighted) {
    danmuEntry.appendChild(createMedal(msg.sender.medal_info))
  }
  const danmuSender = document.createElement('span')
  danmuSender.className = 'sender'
  danmuSender.innerText = msg.sender.uname
  danmuEntry.appendChild(danmuSender)
  // Content
  const danmuContent = document.createElement('span')
  danmuContent.className = 'content'
  if (msg.action == 2) {
    danmuContent.innerText = '关注了直播间'
  } else {
    danmuContent.innerText = '进入了直播间'
  }
  danmuContent.style.color = 'var(--uname-color)'
  danmuContent.style.marginLeft = '8px'
  danmuEntry.appendChild(danmuContent)
  danmuEntry.addEventListener('dblclick', () => {
    window.jliverAPI.window.windowDetail(msg.sender.uid)
  })
  return danmuEntry
}

function doCreateGiftEntry(gift: GiftMessage) {
  const danmuEntry = document.createElement('span')
  danmuEntry.className = 'danmu_entry special gift'
  // check medal
  if (gift.sender.medal_info && gift.sender.medal_info.is_lighted) {
    danmuEntry.appendChild(createMedal(gift.sender.medal_info))
  }
  const danmuSender = document.createElement('span')
  danmuSender.className = 'sender'
  danmuSender.innerText = gift.sender.uname
  danmuEntry.appendChild(danmuSender)
  // Content
  const giftAction = document.createElement('span')
  giftAction.className = 'action'
  giftAction.innerText = gift.action
  danmuEntry.appendChild(giftAction)
  const giftName = document.createElement('span')
  giftName.className = 'gift-name'
  giftName.innerText = gift.gift_info.name
  danmuEntry.appendChild(giftName)
  const giftIcon = document.createElement('span')
  giftIcon.className = 'gift-icon'
  giftIcon.style.backgroundImage = `url(${gift.gift_info.webp})`
  danmuEntry.appendChild(giftIcon)
  const giftNum = document.createElement('span')
  giftNum.className = 'gift-num'
  if (gift.gift_info.coin_type == 'gold') {
    giftNum.innerText = `x ${gift.num} | ￥${gift.gift_info.price / 1000}`
  } else {
    giftNum.innerText = `x ${gift.num}`
  }
  danmuEntry.appendChild(giftNum)
  danmuEntry.setAttribute('gift-num', gift.num.toString())
  danmuEntry.addEventListener('dblclick', () => {
    window.jliverAPI.window.windowDetail(gift.sender.uid)
  })
  return danmuEntry
}

function doCreateGuardEntry(g: GuardMessage) {
  const danmuEntry = document.createElement('span')
  danmuEntry.className = 'danmu_entry special gift'
  // check medal
  if (g.sender.medal_info && g.sender.medal_info.is_lighted) {
    danmuEntry.appendChild(createMedal(g.sender.medal_info))
  }
  const danmuSender = document.createElement('span')
  danmuSender.className = 'sender'
  danmuSender.innerText = g.sender.uname
  danmuEntry.appendChild(danmuSender)
  // Content
  const guardAction = document.createElement('span')
  guardAction.className = 'action'
  // TODO different action for 开通/续费
  guardAction.innerText = '开通'
  danmuEntry.appendChild(guardAction)
  const guardName = document.createElement('span')
  guardName.className = 'gift-name'
  guardName.innerText = levelToName(g.guard_level)
  danmuEntry.appendChild(guardName)
  const giftIcon = document.createElement('span')
  giftIcon.className = 'gift-icon'
  giftIcon.style.backgroundImage = `url(${levelToIconURL(g.guard_level)})`
  danmuEntry.appendChild(giftIcon)
  const guardNum = document.createElement('span')
  guardNum.className = 'gift-num'
  guardNum.innerText = `x ${g.num}${g.unit} | ￥${g.price / 1000}`
  danmuEntry.appendChild(guardNum)
  danmuEntry.addEventListener('dblclick', () => {
    window.jliverAPI.window.windowDetail(g.sender.uid)
  })
  return danmuEntry
}
