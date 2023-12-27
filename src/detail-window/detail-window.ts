import Alpine from "alpinejs"
import JEvent from "../lib/events"
import { DetailInfo, WindowType } from "../lib/types"

const appStatus = {
    async init() {
        window.jliverAPI.register(JEvent.EVENT_DETAIL_UPDATE, (detail_info: DetailInfo) => {
            console.log(detail_info)
            this.detail_info = detail_info
            this.detail_info.sender.face = 'https:' + this.detail_info.sender.face
        })

        // Set theme class in html
        this.theme = await window.jliverAPI.get('config.theme', 'light')
        document.documentElement.classList.add('theme-'+(this.theme || 'light'))
        window.jliverAPI.onDidChange('config.theme', (newValue: string) => {
            document.documentElement.classList.remove('theme-' + (this.theme || 'light'))
            document.documentElement.classList.add('theme-' + (newValue || 'light'))
            this.theme = newValue
        })

    },
    opacity: 1,
    font: 'system-ui',
    font_size: 14,
    theme: 'light',
    detail_info: null,
    hide() {
        window.jliverAPI.window.hide(WindowType.WDETAIL)
    },
    open() {
        window.jliverAPI.util.openUrl('https://space.bilibili.com/' + this.detail_info.sender.uid)
    },
    copy(text: string) {
        window.jliverAPI.util.setClipboard(text)
    },
    timestamp2date(timestamp: number) {
        return new Date(timestamp).toLocaleString()
    },
}

Alpine.data('appStatus', () => appStatus)
Alpine.start()