import Alpine from 'alpinejs'
import { createConfirmBox } from '../lib/common/confirmbox'
import { createSuperchatEntry } from '../lib/common/superchat'
import JEvent from '../lib/events'
import { WindowType } from '../lib/types'
import { JLiverAPI } from '../preload'
import { SuperChatMessage } from '../lib/messages'

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
      this.font_size = newValue
    })
    window.jliverAPI.onDidChange('config.font', (newValue: string) => {
      this.font = newValue
    })
    document.documentElement.classList.add('theme-' + (this.theme || 'light'))
    window.jliverAPI.onDidChange('config.theme', (newValue: string) => {
      this.theme = newValue
    })
    await this.initSuperchats()
    window.jliverAPI.register(
      JEvent.EVENT_NEW_SUPER_CHAT,
      (sc: SuperChatMessage) => {
        this.superchatHandler(sc)
      }
    )
    window.jliverAPI.onDidChange('config.room', () => {
      // clear superchats when room changed
      $panel.innerHTML = ''
      giftMap = new Map()
      this.initSuperchats()
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
    document.documentElement.classList.remove(
      'theme-' + (this._theme || 'light')
    )
    document.documentElement.classList.add('theme-' + (newValue || 'light'))
    this._theme = newValue
  },
  async initSuperchats() {
    let init_superchats = await window.jliverAPI.backend.getInitSuperChats()
    init_superchats.forEach((sc) => {
      this.superchatHandler(sc)
    })
  },
  notifyClear() {
    document.body.appendChild(
      createConfirmBox('确定清空所有醒目留言记录？', () => {
        $panel.innerHTML = ''
        giftMap = new Map()
        window.jliverAPI.backend.clearSuperChats()
      })
    )
  },
  superchatHandler(sc: SuperChatMessage) {
    let scEntry = createSuperchatEntry(sc, true)
    $panel.appendChild(scEntry)
    if (autoScroll) {
      $panel.scrollTop = lastPosition =
        $panel.scrollHeight - $panel.clientHeight
    }
  },
  hide() {
    window.jliverAPI.window.hide(WindowType.WSUPERCHAT)
  },
}

Alpine.data('app', () => app)
Alpine.start()
