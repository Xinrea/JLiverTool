import Alpine from 'alpinejs'
import { WindowType } from '../lib/types'

const app = {
  lite_mode: false,
  theme: 'light',
  opacity: 1,
  font: 'system-ui',
  font_size: 14,
  rank_list: [],
  async init() {
    // Set theme class in html
    this.theme = await window.jliverAPI.get('config.theme', 'light')
    document.documentElement.classList.add('theme-' + (this.theme || 'light'))
    window.jliverAPI.onDidChange('config.theme', (newValue: string) => {
      document.documentElement.classList.remove(
        'theme-' + (this.theme || 'light')
      )
      document.documentElement.classList.add('theme-' + (newValue || 'light'))
      this.theme = newValue
    })

    this.opacity = await window.jliverAPI.get('config.opacity', 1)
    window.jliverAPI.onDidChange('config.opacity', (newValue: number) => {
      this.opacity = newValue
    })
    this.font = await window.jliverAPI.get('config.font', 'system-ui')
    window.jliverAPI.onDidChange('config.font', (newValue: string) => {
      this.font = newValue
    })
    this.font_size = await window.jliverAPI.get('config.font_size', 14)
    window.jliverAPI.onDidChange('config.font_size', (newValue: number) => {
      this.font_size = newValue
    })
    this.lite_mode = await window.jliverAPI.get('config.lite-mode', false)
    window.jliverAPI.onDidChange('config.lite-mode', (newValue: boolean) => {
      this.lite_mode = newValue
    })
    this.rank_list = await (
      await window.jliverAPI.backend.getRankList(1, 50)
    ).data.item
    setInterval(async () => {
      this.rank_list = (
        await window.jliverAPI.backend.getRankList(1, 50)
      ).data.item
    }, 10 * 1000)
    console.log(this.rank_list)
  },
  hide() {
    window.jliverAPI.window.hide(WindowType.WRANK)
  },
  detail(uid: number) {
    window.jliverAPI.window.windowDetail(uid)
  },
}

Alpine.data('app', () => app)
Alpine.start()
