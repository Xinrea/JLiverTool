import * as moment from 'moment'
import 'moment/locale/zh-cn'
import { createConfirmBox } from '../lib/common/confirmbox'
import Alpine from 'alpinejs'
import { JLiverAPI } from '../preload'
import JEvent from '../lib/events'
import { WindowType } from '../lib/types'
import { GiftMessage, GuardMessage } from '../lib/messages'

declare global {
  interface Window {
    jliverAPI: JLiverAPI
  }
}

Alpine.data('appStatus', () => ({
  async init() {
    this.base.$panel = document.getElementById('gift-panel')
    // Opacity related
    this.base.opacity = await window.jliverAPI.get('config.opacity', 1)
    window.jliverAPI.onDidChange('config.opacity', (newValue: number) => {
      this.base.opacity = newValue
    })

    this.base.font = await window.jliverAPI.get('config.font', 'system-ui')
    window.jliverAPI.onDidChange('config.font', (newValue: string) => {
      this.base.font = newValue
    })

    this.base.font_size = await window.jliverAPI.get('config.font_size', 14)
    window.jliverAPI.onDidChange('config.font_size', (newValue: number) => {
      this.base.font_size = newValue
    })
    this.base.filterFree = await window.jliverAPI.get(
      'config.ignore_free',
      true
    )

    await this.initGifts()

    window.jliverAPI.register(JEvent.EVENT_NEW_GIFT, (gift: GiftMessage) => {
      console.log(gift)
      this.giftHandler(gift)
      setTimeout(() => {
        if (this.base.autoScroll) {
          this.base.$panel.scrollTop = this.base.lastPosition =
            this.base.$panel.scrollHeight - this.base.$panel.clientHeight
        }
      }, 10)
    })
    window.jliverAPI.register(JEvent.EVENT_NEW_GUARD, (guard: GuardMessage) => {
      console.log(guard)
      this.guardHandler(guard)
      setTimeout(() => {
        if (this.base.autoScroll) {
          this.base.$panel.scrollTop = this.base.lastPosition =
            this.base.$panel.scrollHeight - this.base.$panel.clientHeight
        }
      }, 10)
    })
    window.jliverAPI.register(JEvent.EVENT_NEW_SUPER_CHAT, (arg) => {
      this.gifts.push(arg)
      setTimeout(() => {
        if (this.base.autoScroll) {
          this.base.$panel.scrollTop = this.base.lastPosition =
            this.base.$panel.scrollHeight - this.base.$panel.clientHeight
        }
      }, 10)
    })
    this.base.theme = await window.jliverAPI.get('config.theme', 'light')
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
    window.jliverAPI.onDidChange('config.room', () => {
      // clear all gifts and reinit
      this.gifts = []
      this.giftsCheck.clear()
      this.initGifts()
    })
    this.base.lite_mode = await window.jliverAPI.get('config.lite-mode', false)
    window.jliverAPI.onDidChange('config.lite-mode', (newValue: boolean) => {
      this.base.lite_mode = newValue
    })
  },
  base: {
    $panel: null,
    get filterFree(): boolean {
      return this._filter_free
    },
    set filterFree(value: boolean) {
      this._filter_free = value
      window.jliverAPI.set('config.ignore_free', value)
    },
    opacity: 1,
    font: 'system-ui',
    font_size: 14,
    theme: 'light',
    lastSelected: null,
    lastPosition: 0,
    autoScroll: true,
    lite_mode: false,
    _filter_free: true,
    scroll() {
      if (Math.ceil(this.$panel.scrollTop) == this.lastPosition) {
        // Auto scroll
        this.autoScroll = true
        return
      }
      // User scroll
      this.autoScroll =
        Math.ceil(this.$panel.scrollTop) ==
        this.$panel.scrollHeight - this.$panel.clientHeight
    },
  },
  gifts: [],
  giftsCheck: new Map(),
  async initGifts() {
    console.log('init stored gifts')
    const gift_data = await window.jliverAPI.backend.getInitGifts()
    for (let i = 0; i < gift_data.gifts.length; i++) {
      this.giftHandler(gift_data.gifts[i])
    }
    for (let i = 0; i < gift_data.guards.length; i++) {
      this.guardHandler(gift_data.guards[i])
    }
    // sort gifts to merge normal gifts and guard gifts
    this.gifts.sort((a: any, b: any) => {
      return a.timestamp - b.timestamp
    })
    setTimeout(() => {
      if (this.base.autoScroll) {
        this.base.$panel.scrollTop = this.base.lastPosition =
          this.base.$panel.scrollHeight - this.base.$panel.clientHeight
      }
    }, 10)
  },
  giftRemove(id: string) {
    window.jliverAPI.backend.removeGiftEntry('gift', id)
    for (let i = 0; i < this.gifts.length; i++) {
      if (this.gifts[i].id == id) {
        this.gifts.splice(i, 1)
      }
    }
    this.giftsCheck.delete(id)
  },
  giftClean() {
    document.body.appendChild(
      createConfirmBox('确定清空所有礼物和上舰记录？', () => {
        this.gifts = []
        this.giftsCheck.clear()
        window.jliverAPI.backend.clearGifts()
      })
    )
  },
  timeFormat(timestamp: number) {
    return moment(timestamp * 1000).format('YYYY/MM/DD HH:mm:ss')
  },
  intToColor(value: number) {
    let hexString = value.toString(16)
    while (hexString.length < 6) {
      hexString = hexString + '0'
    }
    return '#' + hexString
  },
  hide() {
    window.jliverAPI.window.hide(WindowType.WGIFT)
  },
  typeOfGift(gift: any) {
    // if have guard_level
    if (gift.guard_level) {
      return 1
    }
    return 0
  },
  giftHandler(gift: GiftMessage) {
    if (this.giftsCheck.has(gift.id)) {
      for (let i = 0; i < this.gifts.length; i++) {
        if (this.gifts[i].id === gift.id) {
          this.gifts[i].num += gift.num
          break
        }
      }
      return
    }
    this.giftsCheck.set(gift.id, true)
    this.gifts.push(gift)
    // Wait for view render
    setTimeout(() => {
      if (this.base.autoScroll) {
        this.base.$panel.scrollTop = this.base.lastPosition =
          this.base.$panel.scrollHeight - this.base.$panel.clientHeight
      }
    }, 10)
  },
  guardHandler(guard: GuardMessage) {
    this.gifts.push(guard)
    // Wait for view render
    setTimeout(() => {
      if (this.base.autoScroll) {
        this.base.$panel.scrollTop = this.base.lastPosition =
          this.base.$panel.scrollHeight - this.base.$panel.clientHeight
      }
    }, 10)
  },
  levelToName(level: number) {
    switch (level) {
      case 1:
        return '总督'
      case 2:
        return '提督'
      case 3:
        return '舰长'
      default:
        return '舰长'
    }
  },
}))

Alpine.start()
