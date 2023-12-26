import Alpine from 'alpinejs'
import { createConfirmBox } from '../common/confirmbox'
import { createSuperchatEntry } from '../common/superchat'
import JEvent from '../lib/events'
import { WindowType } from '../lib/types'
import { JLiverAPI } from '../preload'

let $panel = document.getElementById('panel')
let giftMap = new Map()

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

declare global {
  interface window {
    jliverAPI: JLiverAPI
  }
}

const app = {
  async init() {
    this.opacity = await window.jliverAPI.get('config.opacity', 1)
    this.theme = await window.jliverAPI.get('config.theme', 'light')
    this.font = await window.jliverAPI.get('config.font', 'system-ui')
    this.font_size = await window.jliverAPI.get('config.font_size', 14)
    window.jliverAPI.onDidChange('config.opacity', (newValue: number) => {
      this.opacity = newValue
    })
    window.jliverAPI.onDidChange('config.font_size', (newValue: number) => {
      this.font_size= newValue
    })
    window.jliverAPI.onDidChange('config.font', (newValue: string) => {
      this.font= newValue
    })
    document.documentElement.classList.add('theme-'+(this.theme || 'light'))
    window.jliverAPI.onDidChange('config.theme', (newValue: string) => {
      this.theme = newValue
    })
  },
  opacity: 1,
  font: 'system-ui',
  font_size: 14,
  _theme: 'light',
  get theme() {
    return this._theme
  },
  set theme(newValue) {
    document.documentElement.classList.remove('theme-' + (this._theme || 'light'))
    document.documentElement.classList.add('theme-' + (newValue || 'light'))
    this._theme = newValue
  },
  notifyClear() {
    document.body.appendChild(
      createConfirmBox('确定清空所有醒目留言记录？', () => {
        $panel.innerHTML = ''
        giftMap = new Map()
        window.jliverAPI.send('clear-superchats')
      })
    )
  },
  hide() {
    window.jliverAPI.window.hide(WindowType.WSUPERCHAT)
  }
}

Alpine.data('app', () => app)
Alpine.start()
