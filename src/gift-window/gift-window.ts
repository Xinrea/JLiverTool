import * as moment from 'moment'
import 'moment/locale/zh-cn'
import { createConfirmBox } from '../common/confirmbox'
import Alpine from 'alpinejs'
import { JLiverAPI } from '../preload'
import JEvent from '../lib/events'

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

    window.jliverAPI.register(JEvent.EVENT_NEW_GIFT, (arg) => {
      console.log(arg)
      if (this.giftsCheck.has(arg.id)) {
        for (let i = 0; i < this.gifts.length; i++) {
          if (this.gifts[i].id === arg.id) {
            this.gifts[i].msg.data.num += arg.msg.data.num
            break
          }
        }
        return
      }
      this.giftsCheck.set(arg.id, true)
      this.gifts.push(arg)
      // Wait for view render
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
      document.documentElement.classList.add(
        'theme-' + (newValue || 'light')
      )
      this.base.theme = newValue
    })
  },
  base: {
    $panel: null,
    get filterFree(): boolean {
      return window.jliverAPI.get('config.passFreeGift', true)
    },
    set filterFree(value: boolean) {
      window.jliverAPI.set('config.passFreeGift', value)
    },
    opacity: 1,
    font: 'system-ui',
    font_size: 14,
    theme: 'light',
    lastSelected: null,
    lastPosition: 0,
    autoScroll: true,
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
  giftRemove(id: string) {
    window.jliverAPI.send('remove', {
      type: 'gifts',
      id: id,
    })
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
        this.gifts = new Map()
        window.jliverAPI.send('clear-gifts')
        window.jliverAPI.send('clear-guards')
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
}))

Alpine.start()
