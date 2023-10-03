import { createConfirmBox } from '../common/confirmbox'
import { createSuperchatEntry } from '../common/superchat'
import JEvent from '../lib/events'

let $hideButton = document.getElementById('hide-button')
let $panel = document.getElementById('panel')
let giftMap = new Map()

window.jliverAPI.onDidChange('config.opacity', (newValue) => {
  document.documentElement.style.setProperty('--global-opacity', newValue)
})

document.getElementById('clear-button').onclick = () => {
  document.body.appendChild(
    createConfirmBox('确定清空所有醒目留言记录？', () => {
      $panel.innerHTML = ''
      giftMap = new Map()
      window.jliverAPI.send('clear-superchats')
    })
  )
}

$hideButton.onclick = () => {
  window.jliverAPI.send('hideSuperchatWindow')
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
  autoScroll =
    Math.ceil($panel.scrollTop) == $panel.scrollHeight - $panel.clientHeight
})

window.jliverAPI.register(JEvent.EVENT_NEW_SUPER_CHAT, (g) => {
  console.log(g)
  let scEntry = createSuperchatEntry({ id: g.id, g: g.msg, removable: true })
  $panel.appendChild(scEntry)
  if (autoScroll) {
    $panel.scrollTop = lastPosition = $panel.scrollHeight - $panel.clientHeight
  }
})
