function createSuperchatEntry(id, g, removable) {
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
  if (removable) {
    scEntry.ondblclick = () => {
      scEntry.remove()
      window.electron.send('remove', {
        type: 'superchats',
        id: id
      })
    }
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

const MockSuperchat = {
  id: '28b18dd5-8b6f-44fe-9fe0-e181a4d146fa',
  msg: {
    cmd: 'SUPER_CHAT_MESSAGE',
    data: {
      background_bottom_color: '#2A60B2',
      background_color: '#EDF5FF',
      background_color_end: '#405D85',
      background_color_start: '#3171D2',
      background_icon: '',
      background_image:
        'https://i0.hdslb.com/bfs/live/a712efa5c6ebc67bafbe8352d3e74b820a00c13e.png',
      background_price_color: '#7497CD',
      color_point: 0.7,
      dmscore: 120,
      end_time: 1645973356,
      gift: {
        gift_id: 12000,
        gift_name: '醒目留言',
        num: 1
      },
      id: 3420457,
      is_ranked: 0,
      is_send_audit: 0,
      medal_info: {
        anchor_roomid: 21919321,
        anchor_uname: 'HiiroVTuber',
        guard_level: 3,
        icon_id: 0,
        is_lighted: 1,
        medal_color: '#1a544b',
        medal_color_border: 6809855,
        medal_color_end: 5414290,
        medal_color_start: 1725515,
        medal_level: 21,
        medal_name: '王牛奶',
        special: '',
        target_id: 508963009
      },
      message: 'hiiro,二周年快乐！',
      message_font_color: '#A3F6FF',
      message_trans: '',
      price: 30,
      rate: 1000,
      start_time: 1645973296,
      time: 60,
      token: 'F6EA31D4',
      trans_mark: 0,
      ts: 1645973296,
      uid: 21131097,
      user_info: {
        face: 'http://i0.hdslb.com/bfs/face/dd69fba7016323edf120ef5ef8171d723d76673b.jpg',
        face_frame:
          'https://i0.hdslb.com/bfs/live/80f732943cc3367029df65e267960d56736a82ee.png',
        guard_level: 3,
        is_main_vip: 0,
        is_svip: 0,
        is_vip: 0,
        level_color: '#61c05a',
        manager: 0,
        name_color: '#00D1F1',
        title: 'title-111-1',
        uname: '慕臣来喝口王牛奶吧',
        user_level: 12
      }
    },
    roomid: 21919321
  }
}
