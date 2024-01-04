import { renderContent } from '../common/content-render'
import { createMedal } from '../common/medal'
import JEvent from '../lib/events'
import { GiftMessage } from '../lib/messages'
import { EmojiContent, MedalInfo, Sender } from '../lib/types'

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
  if (medal.medal_color && medal.medal_color_border != 12632256) {
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

export function createEnterEntry(medal: MedalInfo, sender: Sender) {
  // TODO: need a better way to handle this
  sender.uname = sender.uname + ' 进入直播间'
  return createDanmuEntry(
    -1,
    false,
    medal,
    sender,
    null,
    null
  )
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

export function createGuardEntry(g) {
  let medalInfo = null
  if (g.medal) {
    medalInfo = {
      guardLevel: g.medal.guard_level,
      name: g.medal.medal_name,
      level: g.medal.level,
    }
  }
  // TODO handle guard message
  return
}

function doCreateGiftEntry(gift: GiftMessage) {
  const danmuEntry = document.createElement('span')
  danmuEntry.className = 'danmu_entry special gift'
  // check medal
  if (gift.sender.medal_info && gift.sender.medal_info.medal_level > 0) {
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
  giftNum.innerText = `共${gift.num}个 | ￥${(gift.gift_info.price * gift.num) / 1000}`
  danmuEntry.appendChild(giftNum)
  danmuEntry.setAttribute('gift-num', gift.num.toString())
  return danmuEntry
}
