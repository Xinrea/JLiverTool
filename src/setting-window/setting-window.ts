import Alpine from 'alpinejs'

enum QrPrompt {
    NeedScan = "请使用 b 站 App 扫码登录",
    NeedConfirm = "请确认登录",
    Success = "登录成功"
}

Alpine.data('tab', (): any => ({
    init() {
        this.roomID = window.electron.get('config.room', '')
        this.loggined = window.electron.get('config.loggined', false)
        window.electron.onDidChange('config.loggined', (v: boolean) => {
            this.loggined = v
        })
        if (!this.loggined) {
            this.updateQrCode()
        } else {
            this.updateUserInfo()
        }
        window.electron.invoke('getVersion').then(ver => {
            this.currentVersion = ver
        })
    },
    active: 0,
    loggined: false,
    qrImage: '',
    qrStatusChecker: null,
    qrPrompt: QrPrompt.NeedScan,
    userInfo: null,
    currentVersion: '',
    items: [
        {
            id: 0,
            text: '基础设置'
        },
        {
            id: 1,
            text: '外观设置'
        },
        {
            id: 2,
            text: '关于'
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
    },
    async updateQrCode() {
        let qrdata = await window.electron.invoke('getQrCode')
        let qrcode = require('qrcode')
        qrcode.toDataURL(qrdata.url, (err: any, url: any) => {
            this.qrImage = url
            if (this.qrStatusChecker) {
                clearInterval(this.qrStatusChecker)
            }
            this.qrStatusChecker = setInterval(async () => {
                let qrStatus = await window.electron.invoke('checkQrCode', qrdata.oauthKey)
                switch (qrStatus.status) {
                    case 2:
                        window.electron.set('config.cookies', qrStatus.cookies)
                        window.electron.set('config.loggined', true)
                        this.loggined = true
                        this.qrPrompt = QrPrompt.Success
                        clearInterval(this.qrStatusChecker)
                        await this.updateUserInfo()
                        break;
                    case 1:
                        this.qrPrompt = QrPrompt.NeedConfirm
                        break
                    case 0:
                        this.qrPrompt = QrPrompt.NeedScan
                        break
                    default:
                        break;
                }
            }, 3000)
        })
    },
    async updateUserInfo() {
        // Update userInfo
        let mid = window.electron.get('config.cookies', {}).DedeUserID
        let userData = await window.electron.invoke('getUserInfo', mid)
        this.userInfo = userData
    },
    async accountLogout() {
        await window.electron.invoke('logout')
        window.electron.set('config.loggined', false)
        window.electron.set('config.cookies', '')
        this.userInfo = null
        this.loggined = false
        await this.updateQrCode()
    },
    openURL(url) {
        window.electron.send('openURL', url)
    }
}))
Alpine.start()