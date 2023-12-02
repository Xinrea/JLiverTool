import { app, ipcMain, shell } from 'electron'
import JEvent from './events'
import JLogger from './logger'
import { BiliWebSocket, PackResult } from './bilibili/biliws'
import { WindowManager } from './window_manager'
import { WindowType } from './types'
import BiliApi from './bilibili/biliapi'
import { GiftStore } from './gift_store'
import { Cookies } from './types'
import ConfigStore from './config_store'
import { MessageDanmu } from './messages'
import { CheckQrCodeStatus, GetNewQrCode, Logout } from './bilibili/bililogin'
import { getFonts } from 'font-list'
import GithubApi from './github_api'

const log = JLogger.getInstance('backend_service')

function CreateIntervalTask(f: Function, interval: number) {
  f()
  return setInterval(f, interval)
}

export default class BackendService {
  private _conn: BiliWebSocket
  private _window_manager: WindowManager
  private _owner_uid: number
  private _real_room: number
  private _config_store: ConfigStore

  private _task_update_room_info: number
  private _task_update_online_num: number

  private _gift_list_cache: Map<number, any> = new Map()
  private _gift_store: GiftStore = new GiftStore()

  public constructor(store: ConfigStore, window_manager: WindowManager) {
    this._config_store = store
    this._window_manager = window_manager
  }

  public async Start() {
    let room = this._config_store.Room
    if (room === 0) {
      room = 21484828
      log.warn('Room not set, will use default', { room })
    }
    log.info('Starting backend service', { room })

    log.info('Loading cookies', { uid: this._config_store.Cookies.DedeUserID })

    // Check cookies status
    const nav_response = await BiliApi.Nav(this._config_store.Cookies)
    if (nav_response.code !== 0 || !nav_response.data.isLogin) {
      log.warn('Cookies is invalid, take as logout')
      this._config_store.IsLogin = false
      this._config_store.Cookies = new Cookies({})
    } else {
      this._config_store.IsLogin = true
    }

    // Init room info
    await this.initRoomInfo(room)

    // Must update gift list before receiving any gift message
    await this.updateGiftList()

    // Setup task for updating infos
    this._task_update_room_info = CreateIntervalTask(
      this.updateRoomInfo.bind(this),
      10 * 1000
    )
    this._task_update_online_num = CreateIntervalTask(
      this.updateOnlineNum.bind(this),
      10 * 1000
    )

    // Init events
    this.initEvents()

    // Setup websocket connection with reconnect enabled
    await this.setupWebSocket()

    // everything is ready, now we start windows
    this._window_manager.Start()

    // handle room change
    this._config_store.onDidChange('config.room', async (room) => {
      log.info('Room changed', { room })
      await this.initRoomInfo(room)
      this.updateRoomInfo()
      this.updateOnlineNum()
      this.updateGiftList()
      // release old connection and setup new one
      await this.releaseWebSocket()
      await this.setupWebSocket()
    })
  }

  public async Stop() {
    log.info('Stopping backend service')
    ipcMain.removeAllListeners()
    this.releaseWebSocket()
    clearInterval(this._task_update_room_info)
    clearInterval(this._task_update_online_num)
  }

  private async updateRoomInfo() {
    log.debug('Updating basic room info')
    const room_response = await BiliApi.GetRoomInfo(
      this._config_store.Cookies,
      this._real_room
    )
    this._window_manager.sendTo(
      WindowType.WMAIN,
      JEvent.EVENT_UPDATE_ROOM,
      room_response.data
    )
  }

  private async initRoomInfo(room: number) {
    const status_response = await BiliApi.RoomInit(
      this._config_store.Cookies,
      room
    )
    this._owner_uid = status_response.data.uid
    this._real_room = status_response.data.room_id
  }

  private async setupWebSocket() {
    const danmu_server_info = await BiliApi.GetDanmuInfo(
      this._config_store.Cookies,
      this._real_room
    )
    this._conn = new BiliWebSocket({
      room_id: this._real_room,
      uid: parseInt(this._config_store.Cookies.DedeUserID),
      server: `wss://${danmu_server_info.data.host_list[0].host}/sub`,
      token: danmu_server_info.data.token,
    })
    this._conn.msg_handler = this.msgHandler.bind(this)
    this._conn.Connect(true)
    log.debug('Websocket connected', { room: this._real_room })
  }

  private async releaseWebSocket() {
    this._conn.Disconnect()
    log.debug('Websocket released', { room: this._real_room })
  }

  private async updateOnlineNum() {
    const online_response = await BiliApi.GetOnlineGoldRank(
      this._config_store.Cookies,
      this._owner_uid,
      this._real_room
    )
    log.debug('Updating online number', { online: online_response.data })
    this._window_manager.sendTo(
      WindowType.WMAIN,
      JEvent.EVENT_UPDATE_ONLINE,
      online_response.data
    )
  }

  private async updateGiftList() {
    const gift_response = await BiliApi.GetGiftConfig(
      this._config_store.Cookies,
      this._real_room
    )
    for (const gift of gift_response.data.list) {
      this._gift_list_cache.set(gift.id, gift)
    }
    log.info('Updating gift list', { length: this._gift_list_cache.size })
  }

  private initEvents() {
    // Window request previous gift data
    ipcMain.handle(JEvent[JEvent.INVOKE_REQUEST_GIFT_DATA], (_, ...args) => {
      return this._gift_store.Get(args[0], this._real_room)
    })
    ipcMain.handle(JEvent[JEvent.INVOKE_QR_CODE], async () => {
      return await GetNewQrCode()
    })
    ipcMain.handle(JEvent[JEvent.INVOKE_QR_CODE_UPDATE], async (_, key) => {
      return await CheckQrCodeStatus(key)
    })
    ipcMain.handle(JEvent[JEvent.INVOKE_LOGOUT], async () => {
      const resp = await Logout(this._config_store.Cookies)
      log.info('Logout', { resp })
      this._config_store.Cookies = new Cookies({})
      this._config_store.IsLogin = false
    })
    ipcMain.handle(
      JEvent[JEvent.INVOKE_GET_USER_INFO],
      async (_, user_id: number) => {
        const resp = await BiliApi.GetUserInfo(
          this._config_store.Cookies,
          user_id
        )
        log.debug('Get user info', { resp })
        return resp
      }
    )
    ipcMain.handle(JEvent[JEvent.INVOKE_OPEN_URL], async (_, url: string) => {
      return shell.openExternal(url)
    })
    ipcMain.handle(JEvent[JEvent.INVOKE_GET_ROOM_INFO], async (_, room_id) => {
      const resp = await BiliApi.GetRoomInfo(
        this._config_store.Cookies,
        room_id
      )
      log.debug('Get room info', { room: room_id, resp })
      return resp
    })
    ipcMain.handle(JEvent[JEvent.INVOKE_GET_FONT_LIST], async () => {
      const font_list = await getFonts({ disableQuoting: true })
      log.debug('Get font list', { font_list })
      return font_list
    })
    ipcMain.handle(JEvent[JEvent.INVOKE_GET_VERSION], async () => {
      return app.getVersion()
    })
    ipcMain.on(JEvent[JEvent.EVENT_LOG], (msg) => {
      this._window_manager.sendTo(WindowType.WSETTING, JEvent.EVENT_LOG, msg)
    })
    ipcMain.handle(JEvent[JEvent.INVOKE_OPEN_LOG_DIR], async () => {
      return shell.openPath(JLogger.getLogPath())
    })
    ipcMain.handle(JEvent[JEvent.INVOKE_GET_LATEST_RELEASE], async () => {
      const resp = await GithubApi.GetLatestRelease()
      log.debug('Get latest release', { resp })
      return resp
    })
  }

  private msgHandler(packet: PackResult) {
    for (const msg of packet.body) {
      log.debug('Received message', { msg })
      switch (msg.cmd) {
        case 'DANMU_MSG': {
          const danmu_msg = new MessageDanmu(msg)
          this._window_manager.sendTo(
            WindowType.WMAIN,
            JEvent.EVENT_NEW_DANMU,
            danmu_msg
          )
        }
      }
    }
  }
}
