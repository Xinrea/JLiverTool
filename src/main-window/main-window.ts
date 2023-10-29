import { createSuperchatEntry } from '../common/superchat'
import {
  createDanmuEntry,
  createGiftEntry,
  createEffectEntry,
  createEnterEntry,
  createGuardEntry,
  giftCache,
} from './danmu-entry'
import Alpine from 'alpinejs'
import { JLiverAPI } from '../preload'
import JEvent from '../lib/events'
import { Languages, LanguageType } from '../i18n'
import { WindowType } from '../lib/types'
import { MessageDanmu } from '../lib/messages'

declare global {
  interface Window {
    jliverAPI: JLiverAPI
  }
}

const toggles = {
  async init() {
    console.log('Init toggles')
    const config = await window.jliverAPI.get('config', {})
    this.values['always-on-top'] = config['always-on-top'] || false
    this.values['enter-message'] = config['enter-message'] || false
    this.values['medal-display'] = config['medal-display'] || false

    // always-on-top should be set after init
    window.jliverAPI.window.alwaysOnTop(
      WindowType.WMAIN,
      this.values['always-on-top']
    )
    document.documentElement.style.setProperty(
      '--medal-display',
      this.values['medal-display'] ? 'inline-block' : 'none'
    )
  },
  values: {
    'always-on-top': false,
    'enter-message': false,
    'medal-display': false,
  },
  toggle(name: string) {
    this.values[name] = !this.values[name]
    window.jliverAPI.set(`config.${name}`, this.values[name])
    if (name == 'always-on-top') {
      window.jliverAPI.window.alwaysOnTop(WindowType.WMAIN, this.values[name])
    }
    if (name == 'medal-display') {
      document.documentElement.style.setProperty(
        '--medal-display',
        this.values[name] ? 'inline-block' : 'none'
      )
    }
  },
}

const menu = {
  open: false,
  // get menu item id
  click(e: any) {
    const id = e.target.getAttribute('id')
    if (id) {
      switch (id) {
        case 'gift-window': {
          window.jliverAPI.window.show(WindowType.WGIFT)
          break
        }
        case 'superchat-window': {
          window.jliverAPI.window.show(WindowType.WSUPERCHAT)
          break
        }
        case 'setting-window': {
          window.jliverAPI.window.show(WindowType.WSETTING)
          break
        }
        case 'quit': {
          window.jliverAPI.app.quit()
          break
        }
        default:
      }
    }
    this.open = false
  },
}

const appStatus = {
  async init() {
    console.log('Init config')
    const initialConfig = await window.jliverAPI.get('config', {})

    this.l.texts = Languages[initialConfig.language || LanguageType.zh]
    this.base.fontSize = initialConfig.fontSize || 18
    this.base.opacity = initialConfig.opacity || 1
    this.login = initialConfig.loggined || false

    window.jliverAPI.onDidChange('config.loggined', (v: boolean) => {
      this.login = v
    })
    window.jliverAPI.onDidChange('config.opacity', (newValue: number) => {
      this.base.opacity = newValue
    })
    window.jliverAPI.onDidChange('config.fontSize', (newValue: number) => {
      this.base.fontSize = newValue
    })

    console.log('Init events')
    window.jliverAPI.register(JEvent.EVENT_UPDATE_ONLINE, (arg: any) => {
      // Update online number in title
      if (this.base.live) {
        if (arg.onlineNum >= 9999) {
          this.base.online = '> 10000'
        } else {
          this.base.online = String(arg.onlineNum)
        }
      }
    })
    window.jliverAPI.register(JEvent.EVENT_UPDATE_ROOM, (arg: any) => {
      // Update room title
      this.base.title = arg.title
      this.base.live = arg.live_status == 1
    })
    window.jliverAPI.register(JEvent.EVENT_NEW_DANMU, (arg: MessageDanmu) => {
      this.onReceiveNewDanmu(arg)
    })

    console.log('Init smooth scroll')
    setInterval(() => {
      if (this.danmuPanel.autoScroll && this.danmuPanel.scrollRemain > 0) {
        const v = Math.ceil(this.danmuPanel.scrollRemain / 60)
        $danmuArea.scrollTop += v
        this.danmuPanel.scrollRemain = Math.max(
          Math.ceil(
            $danmuArea.scrollHeight -
              $danmuArea.clientHeight -
              $danmuArea.scrollTop
          ),
          this.danmuPanel.scrollRemain - v
        )
      } else {
        this.danmuPanel.scrollRemain = 0
      }
    }, 16)
  },
  // language texts
  l: {
    texts: {},
  },
  base: {
    title: 'Loading',
    online: '',
    live: false,
    fontSize: 18,
    opacity: 1,
  },
  windowStatus: {
    gift: false,
    superchat: false,
  },
  danmuPanel: {
    replaceIndex: 0,
    lastSelectedDanmu: null,
    newDanmuCount: 0,
    autoScroll: true,
    scrollRemain: 0,
    enableAutoScroll() {
      $danmuArea.scrollTop = $danmuArea.scrollHeight - $danmuArea.clientHeight
      this.scrollRemain = 0
      this.newDanmuCount = 0
      this.autoScroll = true
    },
    handleNewEntry(entry: HTMLElement) {
      $danmuArea.appendChild(entry)
      if (this.autoScroll) {
        this.scrollRemain = Math.ceil(
          $danmuArea.scrollHeight -
            $danmuArea.clientHeight -
            $danmuArea.scrollTop
        )
      }
    },
    scrollHandler() {
      // User scroll
      if (
        Math.ceil($danmuArea.scrollTop) >=
        $danmuArea.scrollHeight - $danmuArea.clientHeight - 10
      ) {
        this.autoScroll = true
        this.newDanmuCount = 0
        this.scrollRemain = Math.ceil(
          $danmuArea.scrollHeight -
            $danmuArea.clientHeight -
            $danmuArea.scrollTop
        )
      } else {
        this.autoScroll = false
      }
    },
    doClean() {
      if (!this.autoScroll) this.newDanmuCount++
      // Only display max 200 entries
      if ($danmuArea.children.length > 200) {
        $danmuArea.removeChild($danmuArea.children[0])
      }
    },
  },
  minimize() {
    window.jliverAPI.window.minimize(WindowType.WMAIN)
  },
  onReceiveNewDanmu(danmu_msg: MessageDanmu) {
    this.danmuPanel.doClean()
    const $newEntry = createDanmuEntry(
      danmu_msg.is_special,
      danmu_msg.sender.medal_info,
      danmu_msg.sender.uname,
      danmu_msg.content,
      danmu_msg.emoji_content
    )
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveInteract(medalInfo, sender) {
    this.danmuPanel.doClean()
    const $newEntry = createEnterEntry(medalInfo, sender)
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveEffect(content) {
    this.danmuPanel.doClean()
    const $newEntry = createEffectEntry(content)
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveGift(id, msg) {
    if (window.jliverAPI.get('config.passFreeGift', true)) {
      if (msg.data.coin_type !== 'gold') {
        return
      }
    }
    if (giftCache.has(id)) {
      const old = giftCache.get(id)
      const oldNum = parseInt(old.getAttribute('gift-num'))
      const newNum = oldNum + parseInt(msg.data.num)
      old.querySelector('.gift-num').innerText = `共${newNum}个`
      old.setAttribute('gift-num', String(newNum))
      return
    }
    this.danmuPanel.doClean()
    const $newEntry = createGiftEntry(id, msg)
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveGuard(id, msg) {
    this.danmuPanel.doClean()
    const $newEntry = createGuardEntry(msg)
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveSuperchat(id, msg) {
    this.danmuPanel.doClean()
    // Superchat entry should not be able to remove in chat window
    const $newEntry = createSuperchatEntry({ id, g: msg, removable: false })
    this.danmuPanel.handleNewEntry($newEntry)
  },
  login: false,
  content: '',
  async sendDanmu(e) {
    if (this.content != '') {
      e.target.innerText = ''
      this.content = this.content.slice(0, -2)
      if (this.content[0] == '/') {
        await window.jliverAPI.invoke('callCommand', this.content)
      } else {
        await window.jliverAPI.invoke('sendDanmu', this.content)
      }
    }
  },
  async handleContentEdit(e) {
    if (e.target.innerText.length <= 30) {
      this.content = e.target.innerText
    } else {
      e.target.innerText = this.content
    }
  },
}

Alpine.data('appStatus', () => appStatus)
Alpine.data('toggles', () => toggles)
Alpine.data('menu', () => menu)
Alpine.start()

const $danmuArea = document.getElementById('danmu')
