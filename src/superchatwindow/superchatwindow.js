$hideButton = document.getElementById('hideButton')
$panel = document.getElementById('panel')

$hideButton.onclick = () => {
  window.electron.send('hideSuperchatWindow')
}

let autoScroll = true
let lastPosition = 0
$panel.addEventListener('scroll', () => {
  if (Math.ceil($panel.scrollTop) == lastPosition) {
    // Auto scroll
    autoScroll = true
    return
  }
  // User scroll
  if (
    Math.ceil($panel.scrollTop) ==
    $panel.scrollHeight - $panel.clientHeight
  ) {
    autoScroll = true
  } else {
    autoScroll = false
  }
})

window.electron.onSuperchat((g) => {
  let scEntry = createSuperchatEntry(g.id, g.msg)
  $panel.appendChild(scEntry)
  if (autoScroll) {
    $panel.scrollTop = lastPosition = $panel.scrollHeight - $panel.clientHeight
  }
})

window.electron.onReset(() => {
  $panel.innerHTML = ''
  window.electron.send('reseted')
})

function createSuperchatEntry(id, g) {
  let level = getSuperChatLevel(g.data.price)
  let scEntry = document.createElement('div')
  scEntry.classList.add('sc-entry')
  let scEntryHeader = document.createElement('div')
  scEntryHeader.classList.add('sc-entry-header')
  scEntryHeader.style.border = `1px solid var(--sc-f-level-${level})`
  scEntryHeader.style.backgroundColor = `var(--sc-b-level-${level})`
  let scEntryHeaderLeft = document.createElement('div')
  scEntryHeaderLeft.classList.add('sc-entry-header-left')
  let scEntryHeaderLeftAvatar = document.createElement('div')
  scEntryHeaderLeftAvatar.classList.add('sc-entry-header-left-avatar')
  let scEntryHeaderLeftAvatarImg = document.createElement('img')
  scEntryHeaderLeftAvatarImg.classList.add('avatar')
  scEntryHeaderLeftAvatarImg.src = g.data.user_info.face
  scEntryHeaderLeftAvatar.appendChild(scEntryHeaderLeftAvatarImg)
  if (g.data.user_info.face_frame != '') {
    let scEntryHeaderLeftAvatarFrameImg = document.createElement('img')
    scEntryHeaderLeftAvatarFrameImg.classList.add('avatar-frame')
    scEntryHeaderLeftAvatarFrameImg.src = g.data.user_info.face_frame
    scEntryHeaderLeftAvatar.appendChild(scEntryHeaderLeftAvatarFrameImg)
  }
  scEntryHeaderLeft.appendChild(scEntryHeaderLeftAvatar)
  let scEntryHeaderLeftName = document.createElement('div')
  scEntryHeaderLeftName.classList.add('sc-entry-header-left-name')
  if (g.data.medal_info) {
    let scEntryHeaderLeftNameMedal = createMedal(
      g.data.medal_info.guard_level,
      g.data.medal_info.medal_name,
      g.data.medal_info.medal_level
    )
    scEntryHeaderLeftName.appendChild(scEntryHeaderLeftNameMedal)
  }
  let scEntryHeaderLeftNameSender = document.createElement('div')
  scEntryHeaderLeftNameSender.classList.add('sender')
  scEntryHeaderLeftNameSender.innerText = g.data.user_info.uname
  scEntryHeaderLeftName.appendChild(scEntryHeaderLeftNameSender)
  scEntryHeaderLeft.appendChild(scEntryHeaderLeftName)
  scEntryHeader.appendChild(scEntryHeaderLeft)
  let scEntryHeaderRight = document.createElement('div')
  scEntryHeaderRight.classList.add('sc-entry-header-right')
  scEntryHeaderRight.innerText = '￥' + g.data.price
  scEntryHeader.appendChild(scEntryHeaderRight)
  scEntry.appendChild(scEntryHeader)
  let scEntryContent = document.createElement('div')
  scEntryContent.classList.add('sc-entry-content')
  scEntryContent.style.backgroundColor = `var(--sc-f-level-${level})`
  let scEntryContentText = document.createElement('div')
  scEntryContentText.classList.add('sc-entry-content-text')
  scEntryContentText.innerText = g.data.message
  scEntryContent.appendChild(scEntryContentText)
  scEntry.appendChild(scEntryContent)
  scEntry.ondblclick = () => {
    scEntry.remove()
    window.electron.send('remove', {
      type: 'superchats',
      id: id
    })
  }
  return scEntry
}

function getSuperChatLevel(price) {
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
