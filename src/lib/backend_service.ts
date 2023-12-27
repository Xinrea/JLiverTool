import { app, clipboard, ipcMain, shell } from 'electron'
import JEvent from './events'
import JLogger from './logger'
import { BiliWebSocket, PackResult } from './bilibili/biliws'
import { WindowManager } from './window_manager'
import { RoomID, WindowType, typecast, MergeUserInfo, Sender } from './types'
import BiliApi from './bilibili/biliapi'
import { GiftStore } from './gift_store'
import { Cookies } from './types'
import ConfigStore from './config_store'
import { MessageDanmu } from './messages'
import { CheckQrCodeStatus, GetNewQrCode, Logout } from './bilibili/bililogin'
import { FontList, getFonts } from 'font-list'
import GithubApi from './github_api'
import { DanmuCache } from './danmu_cache'

const log = JLogger.getInstance('backend_service')

function CreateIntervalTask(f: Function, interval: number) {
  f()
  return setInterval(f, interval)
}

export default class BackendService {
  private _primary_conn: BiliWebSocket
  private _side_conns: Map<RoomID, BiliWebSocket> = new Map()
  private _window_manager: WindowManager
  private _room: RoomID
  private _config_store: ConfigStore

  private _task_update_room_info: number
  private _task_update_online_num: number

  private _gift_list_cache: Map<number, any> = new Map()
  private _gift_store: GiftStore = new GiftStore()

  private _font_list_cached: FontList = []

  private _danmu_cache: DanmuCache = new DanmuCache()

  public constructor(store: ConfigStore, window_manager: WindowManager) {
    this._config_store = store
    this._window_manager = window_manager
  }

  public async Start() {
    // load room setting
    let room = this._config_store.Room
    this._room = room

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

    // Init events
    this.initEvents()

    // Setup websocket connection with reconnect enabled
    await this.setupWebSocket()

    // Init merge rooms
    await this.initMergeRooms()

    this._window_manager.setMainLoadedCallback(() => {
      // Setup task for updating infos
      this._task_update_room_info = CreateIntervalTask(
        this.updateRoomInfo.bind(this),
        10 * 1000
      )
      this._task_update_online_num = CreateIntervalTask(
        this.updateOnlineNum.bind(this),
        10 * 1000
      )
    })

    // everything is ready, now we start windows
    this._window_manager.Start()



    // Init font list
    this._font_list_cached = await getFonts({ disableQuoting: true })
    log.info('Get font list', { size: this._font_list_cached.length })
  }

  public async Stop() {
    log.info('Stopping backend service')
    ipcMain.removeAllListeners()
    this.releaseWebSocket()
    clearInterval(this._task_update_room_info)
    clearInterval(this._task_update_online_num)
    this._window_manager.Stop()
  }

  private async roomChange(room: RoomID) {
    log.info('Room changed', { room })
    this._room = room
    this._config_store.Room = room
    this.updateRoomInfo()
    this.updateOnlineNum()
    await this.updateGiftList()
    // release old connection and setup new one
    await this.releaseWebSocket()
    await this.setupWebSocket()
  }

  private async initMergeRooms() {
    log.info('Initializing merge rooms', {
      enabled: this._config_store.IsMergeEnabled,
      rooms: this._config_store.MergeRooms,
    })
    if (!this._config_store.IsMergeEnabled) {
      return
    }
    this.updateMergeRooms(this._config_store.MergeRooms)
  }

  private async updateMergeRooms(rooms: RoomID[]) {
    log.info('Updating merge rooms', { rooms })
    // release connections not in rooms
    for (const [room, conn] of this._side_conns) {
      if (!rooms.includes(room)) {
        conn.Disconnect()
        this._side_conns.delete(room)
      }
    }
    // setup new connections
    for (let room of rooms) {
      if (!this._side_conns.has(room)) {
        const danmu_server_info = await BiliApi.GetDanmuInfo(
          this._config_store.Cookies,
          room
        )
        const user_info = await BiliApi.GetUserInfo(this._config_store.Cookies, room.getOwnerID())
        const conn = new BiliWebSocket({
          room_id: room.getRealID(),
          uid: parseInt(this._config_store.Cookies.DedeUserID),
          server: `wss://${danmu_server_info.data.host_list[0].host}/sub`,
          token: danmu_server_info.data.token,
        })
        const merge_user_info = {index: rooms.indexOf(room), uid: user_info.data.mid, name: user_info.data.uname}
        log.debug('Merge user info', {merge_user_info})
        conn.msg_handler = this.sideMsgHandlerConstructor(merge_user_info).bind(this)
        conn.Connect(true)
        this._side_conns.set(room, conn)
      }
    }
  }

  private async updateRoomInfo() {
    log.debug('Updating basic room info')
    const room_response = await BiliApi.GetRoomInfo(
      this._config_store.Cookies,
      this._room
    )
    this._window_manager.sendTo(
      WindowType.WMAIN,
      JEvent.EVENT_UPDATE_ROOM,
      room_response.data
    )
  }

  private async initRoomInfo(room: RoomID) {
    const status_response = await BiliApi.RoomInit(
      this._config_store.Cookies,
      room
    )
    if (this._room.getRealID() !== status_response.data.room_id) {
      log.warn("Real room id doesn't match room id, updated", {
        room: this._room,
      })
      this._room = new RoomID(
        status_response.data.short_id,
        status_response.data.room_id,
        status_response.data.uid
      )
      this._config_store.Room = this._room
    }
  }

  private async getRoomBasicInfo(room: RoomID) {
    const status_response = await BiliApi.RoomInit(
      this._config_store.Cookies,
      room
    )
    return {
      uid: status_response.data.uid,
      real_room_id: status_response.data.room_id,
    }
  }

  private async setupWebSocket() {
    const danmu_server_info = await BiliApi.GetDanmuInfo(
      this._config_store.Cookies,
      this._room
    )
    this._primary_conn = new BiliWebSocket({
      room_id: this._room.getRealID(),
      uid: parseInt(this._config_store.Cookies.DedeUserID),
      server: `wss://${danmu_server_info.data.host_list[0].host}/sub`,
      token: danmu_server_info.data.token,
    })
    this._primary_conn.msg_handler = this.msgHandler.bind(this)
    this._primary_conn.Connect(true)
    log.debug('Websocket connected', { room: this._room })
  }

  private async releaseWebSocket() {
    this._primary_conn.Disconnect()
    log.debug('Websocket released', { room: this._room })
  }

  private async updateOnlineNum() {
    const online_response = await BiliApi.GetOnlineGoldRank(
      this._config_store.Cookies,
      this._room
    )
    log.debug('Updating online number', {
      online: online_response.data.onlineNum,
    })
    this._window_manager.sendTo(
      WindowType.WMAIN,
      JEvent.EVENT_UPDATE_ONLINE,
      online_response.data
    )
  }

  private async updateGiftList() {
    const gift_response = await BiliApi.GetGiftConfig(
      this._config_store.Cookies,
      this._room
    )
    for (const gift of gift_response.data.list) {
      this._gift_list_cache.set(gift.id, gift)
    }
    log.info('Updating gift list', { length: this._gift_list_cache.size })
  }

  private initEvents() {
    // Window request previous gift data
    ipcMain.handle(JEvent[JEvent.INVOKE_REQUEST_GIFT_DATA], (_, ...args) => {
      return this._gift_store.Get(args[0], this._room.getRealID())
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
    ipcMain.handle(
      JEvent[JEvent.INVOKE_GET_ROOM_INFO],
      async (_, room: number) => {
        const resp = await BiliApi.GetRoomInfo(
          this._config_store.Cookies,
          new RoomID(0, room, 0)
        )
        log.debug('Get room info', { room: room })
        return resp
      }
    )
    ipcMain.handle(JEvent[JEvent.INVOKE_GET_FONT_LIST], async () => {
      if (this._font_list_cached.length > 0) {
        return this._font_list_cached
      }
      this._font_list_cached = await getFonts({ disableQuoting: true })
      log.info('Get font list', { size: this._font_list_cached.length })
      return this._font_list_cached
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
      log.info('Get latest release')
      return resp
    })
    ipcMain.handle(JEvent[JEvent.INVOKE_UPDATE_ROOM], (_, new_room: RoomID) => {
      this.roomChange(typecast(RoomID, new_room))
    })
    ipcMain.handle(JEvent[JEvent.INVOKE_WINDOW_DETAIL], async (_, uid: number) => {
      // get user info
      const user_info = await BiliApi.GetUserInfo(this._config_store.Cookies, uid)
      const sender = new Sender()
      sender.uid = uid
      sender.uname = user_info.data.uname
      sender.face = user_info.data.face
      const danmus = this._danmu_cache.get(uid)
      const detail_info = {
        sender: sender,
        danmus: danmus,
      }
      this._window_manager.updateDetailWindow(detail_info)
    })
    ipcMain.handle(JEvent[JEvent.INVOKE_SET_CLIPBOARD], async (_, text: string) => {
      clipboard.writeText(text)
    })
    this._config_store.onDidChange(
      'config.merge_rooms',
      async (rooms: RoomID[]) => {
        rooms = rooms.map((room) => typecast(RoomID, room))
        await this.updateMergeRooms(rooms)
      }
    )
  }

  // msg handler for primary connection
  private msgHandler(packet: PackResult) {
    for (const msg of packet.body) {
      switch (msg.cmd) {
        case 'DANMU_MSG': {
          const danmu_msg = new MessageDanmu(msg)
          this._window_manager.sendTo(
            WindowType.WMAIN,
            JEvent.EVENT_NEW_DANMU,
            danmu_msg
          )
          this._danmu_cache.add(danmu_msg.sender.uid, danmu_msg.content)
        }
      }
    }
  }

  // msg handler for side connections
  private sideMsgHandlerConstructor(owner_info: MergeUserInfo) {
    return function(packet: PackResult) {
      for (const msg of packet.body) {
        switch (msg.cmd) {
          case 'DANMU_MSG': {
            const danmu_msg = new MessageDanmu(msg, owner_info)
            this._window_manager.sendTo(
              WindowType.WMAIN,
              JEvent.EVENT_NEW_DANMU,
              danmu_msg
            )
            this._danmu_cache.add(danmu_msg.sender.uid, danmu_msg.content)
          }
        }
      }
    }
  }
}
