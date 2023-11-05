import Alpine from 'alpinejs'
import { JLiverAPI } from '../preload'
import { Cookies, WindowType } from '../lib/types'
import JEvent from '../lib/events'
import * as QrCode from 'qrcode'
import UserInfoResponse from '../lib/bilibili/api/user/user_info'

declare global {
  interface window {
    jliverAPI: JLiverAPI
  }
}

enum QrPrompt {
  NeedConfirm = '请确认登录',
}

Alpine.data('app', (): any => ({
  init() {
    window.jliverAPI.register(JEvent.EVENT_WINDOW_BLUR, () => {
      this.active = false
    })
    window.jliverAPI.register(JEvent.EVENT_WINDOW_FOCUS, () => {
      this.active = true
    })
  },
  active: true,
  hide() {
    window.jliverAPI.window.hide(WindowType.WSETTING)
  },
}))

Alpine.data('room_setting', (): any => ({
  async init() {
    this.settingUpdate()
    window.jliverAPI.onDidChange('config.cookies', async (v: Cookies) => {
      this.settingUpdate()
    })
    window.jliverAPI.onDidChange('config.room', async (v: string) => {
      this.settingUpdate()
    })
  },
  room_id: '',
  room_info: {},
  owned: false,
  error: false,
  async settingUpdate() {
    this.room_id = await window.jliverAPI.get('config.room', '21484828')
    this.room_info = (
      await window.jliverAPI.room.info(parseInt(this.room_id))
    ).data
    const user_id = (await window.jliverAPI.get('config.cookies', {}))
      .DedeUserID
    if (this.room_info.uid == user_id) {
      this.owned = true
    }
  },
  async confirmRoom() {
    const prev_room_id = await window.jliverAPI.get('config.room', '21484828')
    if (this.room_id == '') {
      this.error = true
      this.room_id = prev_room_id
      return
    }
    // length > 16
    if (this.room_id.length > 16) {
      this.error = true
      this.room_id = prev_room_id
      return
    }
    // contains non-number
    if (isNaN(Number(this.room_id))) {
      this.error = true
      this.room_id = prev_room_id
      return
    }
    this.error = false
    if (this.room_id == prev_room_id) {
      return
    }
    // new room id is set, check if it's valid
    const room_info = await window.jliverAPI.room.info(parseInt(this.room_id))
    if (room_info.code != 0) {
      this.error = true
      this.room_id = prev_room_id
      return
    }
    // confirm new room id
    window.jliverAPI.set('config.room', this.room_id)
  },
}))

Alpine.data('account_setting', (): any => ({
  async init() {
    this.login = await window.jliverAPI.get('config.login', false)
    window.jliverAPI.onDidChange('config.login', (v: boolean) => {
      this.login = v
      if (this.login) {
        this.updateUserInfo()
      }
    })

    if (this.login) {
      this.updateUserInfo()
    }
  },
  user_info: {
    face: 'https://i0.hdslb.com/bfs/face/member/noface.jpg',
  },
  login: false,
  qr_dialog: false,
  qr_image: '',
  qr_prompt: '',
  async updateUserInfo() {
    const cookies = await window.jliverAPI.get('config.cookies', {})
    const updated_user_info = (await window.jliverAPI.user.info(
      parseInt(cookies.DedeUserID)
    )) as UserInfoResponse
    console.log(updated_user_info)
    this.user_info = updated_user_info.data
  },
  async qrLogin() {
    const qr_info = await window.jliverAPI.qr.get()
    this.qr_image = await QrCode.toDataURL(qr_info.url)
    this.qr_dialog = true
    // Setup interval to check qr status
    const qr_status_checker = setInterval(async () => {
      const qr_status = await window.jliverAPI.qr.update(qr_info.oauthKey)
      switch (qr_status.status) {
        case 2:
          const cookies = qr_status.cookies as Cookies
          window.jliverAPI.set('config.cookies', cookies)
          window.jliverAPI.set('config.login', true)
          this.login = true
          this.qr_dialog = false
          this.qr_prompt = ''
          this.updateUserInfo()
          clearInterval(qr_status_checker)
          break
        case 1:
          this.qr_prompt = QrPrompt.NeedConfirm
          break
        case 0:
          break
        default:
          break
      }
    }, 2000)
  },
  async updateQrCode() {},
  async accountLogout() {
    await window.jliverAPI.invoke('logout')
    this.userInfo = null
    this.login = false
    await this.updateQrCode()
  },
}))

Alpine.data('merge_setting', (): any => ({
  async init() {
    this._enable = await window.jliverAPI.get('config.merge', false)
    const merge_rooms = await window.jliverAPI.get('config.merge_rooms', [])
    const current_room = await window.jliverAPI.get('config.room', '21484828')
    for (const room_id of merge_rooms) {
      if (room_id == current_room) {
        continue
      }
      const room_info = await window.jliverAPI.room.info(room_id)
      this.room_list.push({
        id: room_id,
        name: room_info.data.title,
      })
    }
    window.jliverAPI.onDidChange('config.room', (v: number) => {
      // filter out current room
      this.room_list = this.room_list.filter((room: any) => {
        return room.id != v
      })
    })
  },
  _enable: false,
  room_list: [],
  error: false,
  to_add: '',
  get enable() {
    return this._enable
  },
  set enable(v: boolean) {
    this._enable = v
    window.jliverAPI.set('config.merge', v)
  },
  async add() {
    if (this.to_add == '') {
      this.error = true
      return
    }
    if (this.room_list.length >= 5) {
      return
    }
    if (isNaN(Number(this.to_add))) {
      this.error = true
      return
    }
    // if room id is already in list
    if (
      this.room_list.find((room: any) => {
        return room.id == parseInt(this.to_add)
      })
    ) {
      this.error = true
      return
    }
    // check if room is same with main room
    const main_room = await window.jliverAPI.get('config.room', '21484828')
    if (main_room == this.to_add) {
      this.error = true
      return
    }
    // check if room id is valid
    const room_info = await window.jliverAPI.room.info(parseInt(this.to_add))
    if (room_info.code != 0) {
      this.error = true
      return
    }
    this.error = false
    this.room_list.push({
      id: parseInt(this.to_add),
      name: room_info.data.title,
    })
    this.to_add = ''
    window.jliverAPI.set(
      'config.merge_rooms',
      this.room_list.map((r: any) => r.id)
    )
  },
  remove(index: number) {
    this.room_list.splice(index, 1)
    window.jliverAPI.set(
      'config.merge_rooms',
      this.room_list.map((r: any) => r.id)
    )
  },
}))

Alpine.data('tab', (): any => ({
  active: 0,
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
}))
Alpine.start()
