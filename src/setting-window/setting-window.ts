import Alpine from 'alpinejs'

Alpine.data('tab', (): any => ({
    init() {
        this.roomID = window.electron.get('config.room', '')
    },
    active: 0,
    items: [
        {
            id: 0,
            text: '基础设置'
        },
        {
            id: 1,
            text: '外观设置'
        }
    ],
    get fontSize() {
        return parseInt(window.electron.get('config.fontSize', 14))
    },
    set fontSize(v: number) {
        if (v < 14) {
            v = 14
        }
        if (v > 40) {
            v = 40
        }
        window.electron.set('config.fontSize', v)
    },
    get opacity() {
        return parseFloat(window.electron.get('config.opacity', 1))
    },
    set opacity(v: number) {
        if (v < 0) {
            v = 0
        }
        if (v > 1) {
            v = 1
        }
        window.electron.set('config.opacity', v)
    },
    roomID: '',
    confirmRoom() {
        window.electron.send('setRoom', this.roomID)
    },
    get darkTheme() {
        const themeSetting = window.electron.get('cache.theme', 'light') as string
        if (themeSetting.includes('dark')) return true
        return false
    },
    set darkTheme(v: boolean) {
        window.electron.send('theme:switch', v ? 'dark' : 'light')
        window.electron.set('cache.theme', v ? 'dark' : 'light')
    }
}))
Alpine.start()