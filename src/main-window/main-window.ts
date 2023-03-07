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

let appStatus = {
  init() {
    configLoad()
    this.base.fontSize = parseInt(window.electron.get('config.fontSize', 18))
    this.base.opacity = window.electron.get('config.opacity', 1)
    window.electron.onDidChange('config.opacity', (newValue) => {
      this.base.opacity = newValue
    })
    window.electron.onDidChange('config.fontSize', (newValue) => {
      this.base.fontSize = newValue
    })
    window.electron.register('update-heat', (arg) => {
      this.base.heat = arg
    })
    window.electron.register('update-room', (arg) => {
      if (!arg) {
        return
      }
      this.base.live = arg.live_status === 1
      this.base.title = arg.title
    })
    window.electron.register('update-online', (arg) => {
      if (this.base.live) {
        if (arg >= 9999) {
          this.base.online = '> 10000'
        } else {
          this.base.online = String(arg)
        }
      }
    })
    window.electron.register('danmu', (arg) => {
      // 过滤礼物弹幕
      if (arg.info[0][9] > 0) {
        return
      }
      // 判断是否为特殊身份
      let special = arg.info[2][2] > 0
      // 处理牌子
      let medalInfo = null
      if (arg.info[3].length > 0) {
        medalInfo = {
          guardLevel: arg.info[3][10],
          name: arg.info[3][1],
          level: arg.info[3][0],
        }
      }
      if (arg.info[0][12] > 0) {
        // Emoji Danmu
        this.onReceiveNewDanmu(
          special,
          medalInfo,
          arg.info[2][1],
          arg.info[0][13]
        )
      } else {
        this.onReceiveNewDanmu(special, medalInfo, arg.info[2][1], arg.info[1])
      }
    })
    // Update window status for determine whether to superchat or gift in main window
    window.electron.register('updateWindowStatus', (arg) => {
      this.windowStatus = arg
    })
    window.electron.register('blur', (arg) => {
      console.log(
        'close menu'
      )
      this.menuOpen = false
    })
    // Register all kinds of message to handle
    window.electron.register('gift', (arg) => {
      if (!this.windowStatus.gift) {
        this.onReceiveGift(arg.id, arg.msg)
      }
    })
    window.electron.register('guard', (arg) => {
      if (!this.windowStatus.gift) {
        this.onReceiveGuard(arg.id, arg.msg)
      }
    })
    window.electron.register('superchat', (arg) => {
      if (!this.windowStatus.superchat) {
        this.onReceiveSuperchat(arg.id, arg.msg)
      }
    })
    window.electron.register('entry-effect', (arg) => {
      if (!this.enterMessage) return
      this.onReceiveEffect(arg)
    })
    window.electron.register('interact', (arg) => {
      if (!this.enterMessage) return
      let medalInfo = null
      if (arg.data.fans_medal.level > 0) {
        medalInfo = {
          guardLevel: arg.data.fans_medal.guard_level,
          name: arg.data.fans_medal.medal_name,
          level: arg.data.fans_medal.medal_level,
        }
      }
      this.onReceiveInteract(medalInfo, arg.data.uname)
    })
    window.electron.register('reset', () => {
      $danmuArea.innerHTML = ''
      this.danmuPanel.autoScroll = true
      this.danmuPanel.newDanmuCount = 0
      this.danmuPanel.isCountingNew = false
      // Reset Full Mode Related
      this.danmuPanel.replaceIndex = 0
      // Make Sure All Data Reset
      window.electron.send('reset')
    })
    // AutoScroll thread, 16 ms for 60 fps
    setInterval(() => {
      if (this.danmuPanel.autoScroll && this.danmuPanel.scrollRemain > 0) {
        let v = Math.ceil(this.danmuPanel.scrollRemain / 60)
        $danmuArea.scrollTop += v
        this.danmuPanel.scrollRemain = Math.max(Math.ceil($danmuArea.scrollHeight - $danmuArea.clientHeight - $danmuArea.scrollTop)
, this.danmuPanel.scrollRemain - v)
      } else {
        this.danmuPanel.scrollRemain = 0
      }
    }, 16)
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
        this.scrollRemain = Math.ceil($danmuArea.scrollHeight - $danmuArea.clientHeight - $danmuArea.scrollTop)
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
        this.scrollRemain = Math.ceil($danmuArea.scrollHeight - $danmuArea.clientHeight - $danmuArea.scrollTop)
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
  get onTop(): boolean {
    return window.electron.get('config.alwaysOnTop', true)
  },
  set onTop(value) {
    window.electron.set('config.alwaysOnTop', value)
    window.electron.send('setAlwaysOnTop', value)
  },
  get enterMessage(): boolean {
    return window.electron.get('config.enableEnter', false)
  },
  set enterMessage(value) {
    window.electron.set('config.enableEnter', value)
  },
  get medalDisplay(): boolean {
    return window.electron.get('config.medalDisplay', false)
  },
  set medalDisplay(value) {
    window.electron.set('config.medalDisplay', value)
    document.documentElement.style.setProperty(
      '--medal-display',
      value ? 'inline-block' : 'none'
    )
  },
  menuOpen: false,
  electronSend: (channel, ...args) => {
    window.electron.send(channel, ...args)
  },
  minimize() {
    window.electron.send('minimize')
  },
  onReceiveNewDanmu(special, medalInfo, sender, content) {
    this.danmuPanel.doClean()
    let $newEntry = createDanmuEntry(special, medalInfo, sender, content)
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveInteract(medalInfo, sender) {
    this.danmuPanel.doClean()
    let $newEntry = createEnterEntry(medalInfo, sender)
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveEffect(content) {
    this.danmuPanel.doClean()
    let $newEntry = createEffectEntry(content)
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveGift(id, msg) {
    if (window.electron.get('config.passFreeGift', true)) {
      if (msg.data.coin_type !== 'gold') {
        return
      }
    }
    if (giftCache.has(id)) {
      let old = giftCache.get(id)
      let oldNum = parseInt(old.getAttribute('gift-num'))
      let newNum = oldNum + parseInt(msg.data.num)
      old.querySelector('.gift-num').innerText = `共${newNum}个`
      old.setAttribute('gift-num', String(newNum))
      return
    }
    this.danmuPanel.doClean()
    let $newEntry = createGiftEntry(id, msg)
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveGuard(id, msg) {
    this.danmuPanel.doClean()
    let $newEntry = createGuardEntry(msg)
    this.danmuPanel.handleNewEntry($newEntry)
  },
  onReceiveSuperchat(id, msg) {
    this.danmuPanel.doClean()
    // Superchat entry should not be able to remove in chat window
    let $newEntry = createSuperchatEntry({ id, g: msg, removable: false })
    this.danmuPanel.handleNewEntry($newEntry)
  },
}

Alpine.data('appStatus', () => appStatus)
Alpine.start()

let $danmuArea = document.getElementById('danmu')

function configLoad() {
  // Load initial medal style in setter
  appStatus.medalDisplay = appStatus.medalDisplay
  // Init font size
}
