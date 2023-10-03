import Alpine from 'alpinejs'
import { JLiverAPI } from '../preload'

declare global {
  interface Window {
    jliverAPI: JLiverAPI
  }
}

enum QrPrompt {
  NeedScan = '请使用 b 站 App 扫码登录',
  NeedConfirm = '请确认登录',
  Success = '登录成功',
}

Alpine.data('tab', (): any => ({
  init() {
    this.roomID = window.jliverAPI.get('config.room', '')
    this.loggined = window.jliverAPI.get('config.loggined', false)
    window.jliverAPI.onDidChange('config.loggined', (v: boolean) => {
      this.loggined = v
    })
    if (!this.loggined) {
      this.updateQrCode()
    } else {
      this.updateUserInfo()
    }
    window.jliverAPI.invoke('getVersion').then((ver) => {
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
      text: '基础设置',
    },
    {
      id: 1,
      text: '外观设置',
    },
    {
      id: 2,
      text: '关于',
    },
  ],
  get fontSize() {
    return parseInt(window.jliverAPI.get('config.fontSize', 14))
  },
  set fontSize(v: number) {
    if (v < 14) {
      v = 14
    }
    if (v > 40) {
      v = 40
    }
    window.jliverAPI.set('config.fontSize', v)
  },
  get opacity() {
    return parseFloat(window.jliverAPI.get('config.opacity', 1))
  },
  set opacity(v: number) {
    if (v < 0) {
      v = 0
    }
    if (v > 1) {
      v = 1
    }
    window.jliverAPI.set('config.opacity', v)
  },
  roomID: '',
  confirmRoom() {
    window.jliverAPI.send('setRoom', this.roomID)
  },
  get darkTheme() {
    const themeSetting = window.jliverAPI.get('cache.theme', 'light') as string
    if (themeSetting.includes('dark')) return true
    return false
  },
  set darkTheme(v: boolean) {
    window.jliverAPI.send('theme:switch', v ? 'dark' : 'light')
    window.jliverAPI.set('cache.theme', v ? 'dark' : 'light')
  },
  async updateQrCode() {
    let qrdata = await window.jliverAPI.invoke('getQrCode')
    let qrcode = require('qrcode')
    qrcode.toDataURL(qrdata.url, (err: any, url: any) => {
      this.qrImage = url
      if (this.qrStatusChecker) {
        clearInterval(this.qrStatusChecker)
      }
      this.qrStatusChecker = setInterval(async () => {
        let qrStatus = await window.jliverAPI.invoke(
          'checkQrCode',
          qrdata.oauthKey
        )
        switch (qrStatus.status) {
          case 2:
            window.jliverAPI.set('config.cookies', qrStatus.cookies)
            window.jliverAPI.set('config.loggined', true)
            this.loggined = true
            this.qrPrompt = QrPrompt.Success
            clearInterval(this.qrStatusChecker)
            await this.updateUserInfo()
            break
          case 1:
            this.qrPrompt = QrPrompt.NeedConfirm
            break
          case 0:
            this.qrPrompt = QrPrompt.NeedScan
            break
          default:
            break
        }
      }, 3000)
    })
  },
  async updateUserInfo() {
    // Update userInfo
    let mid = window.jliverAPI.get('config.cookies', {}).DedeUserID
    let userData = await window.jliverAPI.invoke('getUserInfo', mid)
    this.userInfo = userData
  },
  async accountLogout() {
    await window.jliverAPI.invoke('logout')
    window.jliverAPI.set('config.loggined', false)
    window.jliverAPI.set('config.cookies', '')
    this.userInfo = null
    this.loggined = false
    await this.updateQrCode()
  },
  openURL(url) {
    window.jliverAPI.send('openURL', url)
  },
}))
Alpine.start()
