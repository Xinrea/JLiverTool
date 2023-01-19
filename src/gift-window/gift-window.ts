import * as moment from 'moment'
import 'moment/locale/zh-cn'
import { createMedal } from '../common/medal'
import { createConfirmBox } from '../common/confirmbox'
import Alpine from 'alpinejs'

Alpine.data('appStatus', () => ({
  init() {
    this.base.$panel = document.getElementById('gift-panel')
    // Opacity related
    window.electron.onDidChange('config.opacity', (newValue) => {
      this.base.opacity = newValue
    })
    this.base.opacity = window.electron.get('config.opacity', 1)
    window.electron.register('gift', (arg) => {
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
    window.electron.register('reset', () => {
      this.base.lastSelected = null
      this.base.autoScroll = true
      this.base.lastPosition = 0
      this.gifts = []
      window.electron.send('reset')
    })
    window.electron.register('guard', (arg) => {
      this.gifts.push(arg)
      setTimeout(() => {
        if (this.base.autoScroll) {
          this.base.$panel.scrollTop = this.base.lastPosition =
            this.base.$panel.scrollHeight - this.base.$panel.clientHeight
        }
      }, 10)
    })
  },
  base: {
    $panel: null,
    get filterFree(): boolean {
      return window.electron.get('config.passFreeGift', true)
    },
    set filterFree(value: boolean) {
      window.electron.set('config.passFreeGift', value)
    },
    opacity: 1,
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
    window.electron.send('remove', {
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
        window.electron.send('clear-gifts')
        window.electron.send('clear-guards')
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
