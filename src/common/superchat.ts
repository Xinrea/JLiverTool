import * as moment from 'moment/moment'
import { renderContent } from './content-render'
import { createMedal } from './medal'
import { SuperChat } from './superchatInterface'
import { SuperChatMessage } from '../lib/messages'

// Create Superchat HTML entry for display
export function createSuperchatEntry(
  sc: SuperChatMessage,
  removable: boolean
): HTMLElement {
  const level = getSuperChatLevel(sc.price)
  const scEntry = document.createElement('div')
  scEntry.classList.add('sc-entry')
  const scEntryHeader = document.createElement('div')
  scEntryHeader.classList.add('sc-entry-header')
  scEntryHeader.style.border = `1px solid var(--sc-f-level-${level})`
  scEntryHeader.style.backgroundColor = `var(--sc-b-level-${level})`
  const scEntryHeaderLeft = document.createElement('div')
  scEntryHeaderLeft.classList.add('sc-entry-header-left')
  const scEntryHeaderLeftAvatar = document.createElement('div')
  scEntryHeaderLeftAvatar.classList.add('sc-entry-header-left-avatar')
  const scEntryHeaderLeftAvatarImg = document.createElement('img')
  scEntryHeaderLeftAvatarImg.classList.add('avatar')
  scEntryHeaderLeftAvatarImg.src = sc.sender.face
  scEntryHeaderLeftAvatar.appendChild(scEntryHeaderLeftAvatarImg)
  scEntryHeaderLeft.appendChild(scEntryHeaderLeftAvatar)
  const scEntryHeaderLeftName = document.createElement('div')
  scEntryHeaderLeftName.classList.add('sc-entry-header-left-name')
  if (sc.sender.medal_info && sc.sender.medal_info.medal_level > 0) {
    const scEntryHeaderLeftNameMedal = createMedal(sc.sender.medal_info)
    scEntryHeaderLeftName.appendChild(scEntryHeaderLeftNameMedal)
  }
  const scEntryHeaderLeftNameSender = document.createElement('div')
  scEntryHeaderLeftNameSender.classList.add('sender')
  scEntryHeaderLeftNameSender.innerText = sc.sender.uname
  scEntryHeaderLeftName.appendChild(scEntryHeaderLeftNameSender)
  scEntryHeaderLeft.appendChild(scEntryHeaderLeftName)
  scEntryHeader.appendChild(scEntryHeaderLeft)
  const scEntryHeaderRight = document.createElement('div')
  scEntryHeaderRight.classList.add('sc-entry-header-right')
  scEntryHeaderRight.innerText = '￥' + sc.price
  scEntryHeader.appendChild(scEntryHeaderRight)
  scEntry.appendChild(scEntryHeader)
  const scEntryContent = document.createElement('div')
  scEntryContent.classList.add('sc-entry-content')
  scEntryContent.style.backgroundColor = `var(--sc-f-level-${level})`
  const scEntryContentText = document.createElement('div')
  scEntryContentText.classList.add('sc-entry-content-text')
  scEntryContentText.appendChild(renderContent(sc.message))
  scEntryContent.appendChild(scEntryContentText)
  const scEntryContentTime = document.createElement('div')
  scEntryContentTime.classList.add('sc-entry-content-time')
  scEntryContentTime.innerText = moment(sc.timestamp * 1000).format(
    'YYYY/MM/DD HH:mm:ss'
  )
  scEntryContent.appendChild(scEntryContentTime)
  scEntry.appendChild(scEntryContent)
  if (removable) {
    scEntry.ondblclick = () => {
      scEntry.remove()
      window.jliverAPI.backend.removeGiftEntry('superchat', sc.id)
    }
  }
  return scEntry
}

// Different Superchat amount with different style
function getSuperChatLevel(price: number): number {
  if (price >= 2000) {
    return 5
  } else if (price >= 1000) {
    return 4
  } else if (price >= 500) {
    return 3
  } else if (price >= 100) {
    return 2
  } else if (price >= 50) {
    return 1
  }
  return 0
}
