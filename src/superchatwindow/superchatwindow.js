$hideButton = document.getElementById('hideButton')
$panel = document.getElementById('panel')

document.getElementById('clearButton').onclick = () => {
  document.body.appendChild(
    createConfirmBox('确定清空所有醒目留言记录？', () => {
      panel.innerHTML = ''
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
  if (
    Math.ceil($panel.scrollTop) ==
    $panel.scrollHeight - $panel.clientHeight
  ) {
    autoScroll = true
  } else {
    autoScroll = false
  }
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
  window.electron.send('reseted')
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
