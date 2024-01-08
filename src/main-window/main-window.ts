import { createSuperchatEntry } from '../common/superchat'
import {
  createDanmuEntry,
  createGiftEntry,
  createEffectEntry,
  createGuardEntry,
  giftCache,
  createInteractEntry,
} from './danmu-entry'
import Alpine from 'alpinejs'
import JEvent from '../lib/events'
import { Languages, LanguageType } from '../i18n'
import { MedalInfo, Sender, WindowType } from '../lib/types'
import {
  DanmuMessage,
  GiftMessage,
  GuardMessage,
  InteractMessage,
  SuperChatMessage,
} from '../lib/messages'

const toggles = {
  async init() {
    console.log('Init toggles')
    const config = await window.jliverAPI.get('config', {})
    this.values['always-on-top'] = config['always-on-top'] || false
    this.values['interact-display'] = config['interact-display'] || false
    this.values['medal-display'] = config['medal-display'] || false

    // always-on-top should be set after init
    window.jliverAPI.window.alwaysOnTop(
      WindowType.WMAIN,
      this.values['always-on-top']
    )
    window.jliverAPI.window.minimizable(
      WindowType.WMAIN,
      !this.values['always-on-top']
    )
    document.documentElement.style.setProperty(
      '--medal-display',
      this.values['medal-display'] ? 'inline-block' : 'none'
    )
    document.documentElement.style.setProperty(
      '--interact-display',
      this.values['interact-display'] ? 'inline-block' : 'none'
    )
  },
  values: {
    'always-on-top': false,
    'medal-display': false,
    'interact-display': false,
  },
  toggle(name: string) {
    this.values[name] = !this.values[name]
    window.jliverAPI.set(`config.${name}`, this.values[name])
    if (name == 'always-on-top') {
      window.jliverAPI.window.alwaysOnTop(WindowType.WMAIN, this.values[name])
      window.jliverAPI.window.minimizable(WindowType.WMAIN, !this.values[name])
    }
    if (name == 'medal-display') {
      document.documentElement.style.setProperty(
        '--medal-display',
        this.values[name] ? 'inline-block' : 'none'
      )
    }
    if (name == 'interact-display') {
      document.documentElement.style.setProperty(
        '--interact-display',
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
    this.base.fontSize = initialConfig['font_size'] || 18
    this.base.opacity = initialConfig['opacity'] || 1
    this.base.font = initialConfig['font'] || 'system-ui'
    this.login = initialConfig.login || false
    this.base.theme = initialConfig.theme || 'light'

    window.jliverAPI.onDidChange('config.login', (v: boolean) => {
      this.login = v
    })
    window.jliverAPI.onDidChange('config.opacity', (newValue: number) => {
      this.base.opacity = newValue
    })
    window.jliverAPI.onDidChange('config.font_size', (newValue: number) => {
      this.base.fontSize = newValue
    })
    window.jliverAPI.onDidChange('config.font', (newValue: string) => {
      this.base.font = newValue
    })
    // Set theme class in html
    document.documentElement.classList.add(
      'theme-' + (this.base.theme || 'light')
    )
    window.jliverAPI.onDidChange('config.theme', (newValue: string) => {
      document.documentElement.classList.remove(
        'theme-' + (this.base.theme || 'light')
      )
      document.documentElement.classList.add('theme-' + (newValue || 'light'))
      this.base.theme = newValue
    })

    console.log('Init events')
    window.jliverAPI.register(JEvent.EVENT_UPDATE_ONLINE, (arg: any) => {
      // Update online number
      this.base.online = arg.count
    })
    window.jliverAPI.register(JEvent.EVENT_UPDATE_ROOM, (arg: any) => {
      // Update room title and status
      // if arg has title
      if (arg.hasOwnProperty('title')) {
        var encodedString = arg.title
        var textarea = document.createElement('textarea')
        textarea.innerHTML = encodedString
        this.base.title = textarea.value
      }
      // if arg has live_status
      if (arg.hasOwnProperty('live_status')) {
        this.base.live = arg.live_status == 1
      }
    })
    window.jliverAPI.register(JEvent.EVENT_NEW_DANMU, (arg: DanmuMessage) => {
      this.onReceiveNewDanmu(arg)
    })
    window.jliverAPI.register(JEvent.EVENT_NEW_GIFT, (arg: GiftMessage) => {
      this.onReceiveNewGift(arg)
    })
    window.jliverAPI.register(JEvent.EVENT_NEW_GUARD, (arg: GuardMessage) => {
      this.onReceiveGuard(arg)
    })
    window.jliverAPI.register(
      JEvent.EVENT_NEW_SUPER_CHAT,
      (arg: SuperChatMessage) => {
        this.onReceiveSuperchat(arg)
      }
    )
    window.jliverAPI.register(
      JEvent.EVENT_NEW_INTERACT,
      (arg: InteractMessage) => {
        this.onReceiveInteract(arg)
      }
    )

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
    font: '',
    opacity: 1,
    theme: 'light',
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
  onReceiveNewDanmu(danmu_msg: DanmuMessage) {
    this.danmuPanel.doClean()
    const $newEntry = createDanmuEntry(
      danmu_msg.side_index,
      danmu_msg.is_special,
      danmu_msg.sender.medal_info,
      danmu_msg.sender,
      danmu_msg.content,
      danmu_msg.emoji_content
    )
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveInteract(interact_msg: InteractMessage) {
    this.danmuPanel.doClean()
    const $newEntry = createInteractEntry(interact_msg)
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveEffect(content: string) {
    this.danmuPanel.doClean()
    const $newEntry = createEffectEntry(content)
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveNewGift(gift: GiftMessage) {
    // if ignore free gift
    if (window.jliverAPI.get('config.ignore_free', true)) {
      if (gift.gift_info.coin_type != 'gold') {
        return
      }
    }
    // check gift cache to merge gift in combo
    if (giftCache.has(gift.id)) {
      const old = giftCache.get(gift.id)
      const oldNum = parseInt(old.getAttribute('gift-num'))
      const newNum = oldNum + gift.num
      old.querySelector('.gift-num').innerText = `共${newNum}个`
      old.setAttribute('gift-num', String(newNum))
      return
    }
    this.danmuPanel.doClean()
    const $newEntry = createGiftEntry(gift)
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveGuard(msg: GuardMessage) {
    this.danmuPanel.doClean()
    const $newEntry = createGuardEntry(msg)
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveSuperchat(msg: SuperChatMessage) {
    console.log(msg)
    this.danmuPanel.doClean()
    // Superchat entry should not be able to remove in chat window
    const $newEntry = createSuperchatEntry(msg, false)
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
  openBiliBackend() {
    window.jliverAPI.util.openUrl(
      'https://link.bilibili.com/p/center/index#/my-room/start-live'
    )
  },
}

Alpine.data('appStatus', () => appStatus)
Alpine.data('toggles', () => toggles)
Alpine.data('menu', () => menu)
Alpine.start()

const $danmuArea = document.getElementById('danmu')
