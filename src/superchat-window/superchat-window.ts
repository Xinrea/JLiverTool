import {createConfirmBox} from '../common/confirmbox'
import {createSuperchatEntry} from '../common/superchat'

let $hideButton = document.getElementById('hide-button')
let $panel = document.getElementById('panel')
let giftMap = new Map()

document.getElementById('clear-button').onclick = () => {
  document.body.appendChild(
    createConfirmBox('确定清空所有醒目留言记录？', () => {
      $panel.innerHTML = ''
      giftMap = new Map()
      window.electron.send('clear-superchats')
    })
  )
}

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
  autoScroll = Math.ceil($panel.scrollTop) ==
    $panel.scrollHeight - $panel.clientHeight;
})

window.electron.register('superchat', (g) => {
  let scEntry = createSuperchatEntry(g.id, g.msg, true)
  $panel.appendChild(scEntry)
  if (autoScroll) {
    $panel.scrollTop = lastPosition = $panel.scrollHeight - $panel.clientHeight
  }
})

window.electron.register('reset', () => {
  $panel.innerHTML = ''
  window.electron.send('reset')
})

window.electron.register('updateOpacity', () => {
  document.documentElement.style.setProperty(
    '--global-opacity',
    window.electron.get('config.opacity', 1)
  )
})

document.documentElement.style.setProperty(
  '--global-opacity',
  window.electron.get('config.opacity', 1)
)
