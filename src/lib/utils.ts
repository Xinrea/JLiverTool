export function levelToName(level: number) {
  switch (level) {
    case 1:
      return '总督'
    case 2:
      return '提督'
    case 3:
      return '舰长'
    default:
      return ''
  }
}

export function levelToIconURL(level: number) {
  switch (level) {
    case 1:
      return 'https://i0.hdslb.com/bfs/activity-plat/static/20211222/627754775478985e330c25a90ec7baf0/icon-guard1.png@44w_44h'
    case 2:
      return 'https://i0.hdslb.com/bfs/activity-plat/static/20211222/627754775478985e330c25a90ec7baf0/icon-guard2.png@44w_44h'
    case 3:
      return 'https://i0.hdslb.com/bfs/activity-plat/static/20211222/627754775478985e330c25a90ec7baf0/icon-guard3.png@44w_44h'
    default:
      return ''
  }
}

export function InteractActionToStr(action: number) {
  if (action === 1) {
    return '进入了'
  }
  return '关注了'
}
