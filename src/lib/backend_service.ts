import { app, clipboard, ipcMain, shell, Notification } from 'electron'
import JEvent from './events'
import JLogger from './logger'
import { BiliWebSocket, PackResult } from './bilibili/biliws'
import { WindowManager } from './window_manager'
import { RoomID, WindowType, typecast, MergeUserInfo, Sender } from './types'
import BiliApi from './bilibili/biliapi'
import { GiftStore } from './gift_store'
import { Cookies } from './types'
import ConfigStore from './config_store'
import {
  DanmuMessage,
  GiftInitData,
  GiftMessage,
  GuardMessage,
  InteractMessage,
  SuperChatMessage,
} from './messages'
import { CheckQrCodeStatus, GetNewQrCode, Logout } from './bilibili/bililogin'
import { FontList, getFonts } from 'font-list'
import GithubApi from './github_api'
import { DanmuCache } from './danmu_cache'
import { v4 as uuidv4 } from 'uuid'
import { MockMessageArray } from '../common/mock'
import { GiftType } from './bilibili/api/room/gift_config'

const log = JLogger.getInstance('backend_service')

const dev = process.env.DEBUG === 'true'

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

  private _gift_list_cache: Map<number, GiftType> = new Map()
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
        () => this.updateRoomInfo(),
        30 * 1000
      )
    })

    // Everything is ready, now we start windows
    this._window_manager.Start()

    // Init font list
    this._font_list_cached = await getFonts({ disableQuoting: true })
    log.info('Get font list', { size: this._font_list_cached.length })

    // Using mock data for testing
    if (dev) {
      CreateIntervalTask(() => {
        const n = MockMessageArray.length
        const i = Math.floor(Math.random() * n)
        const msg = MockMessageArray[i]
        this.doHandler(msg)
      }, 2 * 1000)
    }
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
    // if room is in merge rooms, release it
    for (const [merge_room, conn] of this._side_conns) {
      if (merge_room.getRealID() === room.getRealID()) {
        conn.Disconnect()
        this._side_conns.delete(merge_room)
      }
    }
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

  private async releaseMergeRooms() {
    log.info('Releasing merge rooms')
    for (const [room, conn] of this._side_conns) {
      conn.Disconnect()
    }
    this._side_conns.clear()
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
      if (room.getRealID() === this._room.getRealID()) {
        log.warn('Merge room is same as primary room, ignored', { room })
        continue
      }
      if (!this._side_conns.has(room)) {
        const danmu_server_info = await BiliApi.GetDanmuInfo(
          this._config_store.Cookies,
          room
        )
        const user_info = await BiliApi.GetUserInfo(
          this._config_store.Cookies,
          room.getOwnerID()
        )
        const conn = new BiliWebSocket({
          room_id: room.getRealID(),
          uid: parseInt(this._config_store.Cookies.DedeUserID),
          server: `wss://${danmu_server_info.data.host_list[0].host}/sub`,
          token: danmu_server_info.data.token,
        })
        const merge_user_info = {
          index: rooms.indexOf(room),
          uid: user_info.data.mid,
          name: user_info.data.uname,
        }
        log.debug('Merge user info', { merge_user_info })
        conn.msg_handler =
          this.sideMsgHandlerConstructor(merge_user_info).bind(this)
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
    this._window_manager.SendTo(WindowType.WMAIN, JEvent.EVENT_UPDATE_ROOM, {
      title: room_response.data.title,
      live_status: room_response.data.live_status,
    })
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
    this._window_manager.SendTo(
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
      this._window_manager.SendTo(WindowType.WSETTING, JEvent.EVENT_LOG, msg)
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
    ipcMain.handle(
      JEvent[JEvent.INVOKE_WINDOW_DETAIL],
      async (_, uid: number) => {
        // Only login user can get detail info
        if (this._config_store.IsLogin === false) {
          return
        }
        // get user info
        const user_info = await BiliApi.GetUserInfo(
          this._config_store.Cookies,
          uid
        )
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
      }
    )
    ipcMain.handle(
      JEvent[JEvent.INVOKE_SET_CLIPBOARD],
      async (_, text: string) => {
        clipboard.writeText(text)
      }
    )
    ipcMain.handle(JEvent[JEvent.INVOKE_GET_INIT_GIFTS], async () => {
      const stored_gifts = (await this._gift_store.Get(
        'gift',
        this._room.getRealID()
      )) as GiftMessage[]
      const stored_guards = (await this._gift_store.Get(
        'guard',
        this._room.getRealID()
      )) as GuardMessage[]
      log.info('Load stored gifts', {
        gift: stored_gifts.length,
        guard: stored_guards.length,
      })
      const ret = new GiftInitData()
      ret.gifts = stored_gifts
      ret.guards = stored_guards
      return ret
    })
    ipcMain.handle(JEvent[JEvent.INVOKE_GET_INIT_SUPERCHATS], async () => {
      const stored_superchats = (await this._gift_store.Get(
        'superchat',
        this._room.getRealID()
      )) as SuperChatMessage[]
      log.info('Load stored superchats', {
        superchat: stored_superchats.length,
      })
      return stored_superchats
    })
    ipcMain.handle(
      JEvent[JEvent.INVOKE_REMOVE_GIFT_ENTRY],
      async (_, type: string, id: string) => {
        await this._gift_store.Delete(type, id)
      }
    )
    ipcMain.handle(JEvent[JEvent.INVOKE_CLEAR_GIFTS], async () => {
      await this._gift_store.Clear('gift', this._room.getRealID())
      await this._gift_store.Clear('guard', this._room.getRealID())
    })
    ipcMain.handle(JEvent[JEvent.INVOKE_CLEAR_SUPERCHATS], async () => {
      await this._gift_store.Clear('superchat', this._room.getRealID())
    })
    this._config_store.onDidChange(
      'config.merge_rooms',
      async (rooms: RoomID[]) => {
        rooms = rooms.map((room) => typecast(RoomID, room))
        await this.updateMergeRooms(rooms)
      }
    )
    this._config_store.onDidChange('config.login', async (login: boolean) => {
      if (login) {
        // reconnect primary websocket
        await this.releaseWebSocket()
        await this.setupWebSocket()
        // reconnect merge websockets
        await this.releaseMergeRooms()
        await this.initMergeRooms()
      }
    })
  }

  private async msgHandler(packet: PackResult) {
    for (const msg of packet.body) {
      this.doHandler(msg)
    }
  }

  // msg handler for primary connection
  private async doHandler(msg: any) {
    if (!msg.cmd) {
      return
    }
    // using includes to handle special messages generated during some events
    if (msg.cmd.includes('DANMU_MSG')) {
      this.danmuHandler(msg)
      return
    }
    if (msg.cmd.includes('SEND_GIFT')) {
      this.giftHandler(msg)
      return
    }
    if (msg.cmd.includes('USER_TOAST_MSG')) {
      this.guardHandler(msg)
      return
    }
    if (msg.cmd.includes('SUPER_CHAT_MESSAGE')) {
      this.superchatHandler(msg)
      return
    }
    switch (msg.cmd) {
      case 'WARNING': {
        log.warn('Received warning message', { msg })
        new Notification({
          title: '直播警告',
          body: msg.msg,
        }).show()
        break
      }
      case 'CUT_OFF': {
        log.info('Received cutoff message', { msg })
        new Notification({
          title: '直播切断',
          body: msg.msg,
        }).show()
        break
      }
      case 'ROOM_CHANGE': {
        log.info('Received room change message', { msg })
        // update room title
        this._window_manager.SendTo(
          WindowType.WMAIN,
          JEvent.EVENT_UPDATE_ROOM,
          msg.data
        )
        break
      }
      case 'ONLINE_RANK_COUNT': {
        log.debug('Received online rank count message', { msg })
        this._window_manager.SendTo(
          WindowType.WMAIN,
          JEvent.EVENT_UPDATE_ONLINE,
          msg.data
        )
        break
      }
      case 'LIVE': {
        log.info('Received live start message', { msg })
        this._window_manager.SendTo(
          WindowType.WMAIN,
          JEvent.EVENT_UPDATE_ROOM,
          {
            live_status: 1,
          }
        )
        break
      }
      case 'PREPARING': {
        log.info('Received live stop message', { msg })
        this._window_manager.SendTo(
          WindowType.WMAIN,
          JEvent.EVENT_UPDATE_ROOM,
          {
            live_status: 0,
          }
        )
        break
      }
      case 'INTERACT_WORD': {
        log.debug('Received interact word message', { msg })
        this.interactHandler(msg)
        break
      }
    }
  }

  private danmuHandler(msg: any) {
    const danmu_msg = new DanmuMessage(msg)
    if (danmu_msg.is_generated) {
      // ignore generated danmu
      return
    }
    this._window_manager.SendTo(
      WindowType.WMAIN,
      JEvent.EVENT_NEW_DANMU,
      danmu_msg
    )
    this._danmu_cache.add(danmu_msg.sender.uid, danmu_msg.content)
  }

  private async giftHandler(msg: any) {
    if (msg.data.coin_type === 'silver' && msg.data.giftName == '辣条') {
      // ignore this gift
      return
    }
    // unique id for each gift or batch of gift
    let id = msg.data.batch_combo_id
    if (id == '') {
      id = uuidv4()
    }
    let gift_animate_info = null
    if (this._gift_list_cache.has(msg.data.giftId)) {
      gift_animate_info = this._gift_list_cache.get(msg.data.giftId)
    }
    // gift animate info not found, may need update
    if (gift_animate_info === null) {
      log.warn('Gift animate info not found, may need update', {
        gift_id: msg.data.giftId,
      })
      await this.updateGiftList()
      gift_animate_info = this._gift_list_cache.get(msg.data.giftId)
      if (gift_animate_info === null) {
        log.error('Gift animate info not found after update', {
          gift_id: msg.data.giftId,
        })
        return
      }
    }
    // construct gift message from raw msg
    const gift_msg = new GiftMessage()
    gift_msg.id = id
    gift_msg.room = this._room.getRealID()
    gift_msg.sender = new Sender()
    gift_msg.sender.uid = msg.data.uid
    gift_msg.sender.uname = msg.data.uname
    gift_msg.sender.face = msg.data.face
    gift_msg.sender.medal_info = msg.data.medal_info
    gift_msg.gift_info = gift_animate_info
    gift_msg.action = msg.data.action
    gift_msg.num = msg.data.num
    gift_msg.timestamp = msg.data.timestamp
    // send to related window
    this._window_manager.SendTo(
      WindowType.WMAIN,
      JEvent.EVENT_NEW_GIFT,
      gift_msg
    )
    this._window_manager.SendTo(
      WindowType.WGIFT,
      JEvent.EVENT_NEW_GIFT,
      gift_msg
    )
    // store gift message
    this._gift_store.Push(gift_msg)
  }

  private async guardHandler(msg: any) {
    const guard_msg = new GuardMessage()
    guard_msg.id = msg.data.payflow_id
    guard_msg.room = this._room.getRealID()
    guard_msg.sender = new Sender()
    guard_msg.sender.uid = msg.data.uid
    guard_msg.sender.uname = msg.data.username
    guard_msg.num = msg.data.num
    // unit for num, should be '月'
    guard_msg.unit = msg.data.unit
    // guard level, should be 1, 2, 3
    guard_msg.guard_level = msg.data.guard_level
    guard_msg.price = msg.data.price
    // TODO need confirm
    guard_msg.timestamp = msg.data.start_time

    this._window_manager.SendTo(
      WindowType.WMAIN,
      JEvent.EVENT_NEW_GUARD,
      guard_msg
    )
    this._window_manager.SendTo(
      WindowType.WGIFT,
      JEvent.EVENT_NEW_GUARD,
      guard_msg
    )
    // store guard message
    this._gift_store.Push(guard_msg)
  }

  private superchatHandler(msg: any) {
    const superchat_msg = new SuperChatMessage()
    superchat_msg.id = msg.data.id
    superchat_msg.room = this._room.getRealID()
    superchat_msg.sender = new Sender()
    superchat_msg.sender.uid = msg.data.uid
    superchat_msg.sender.uname = msg.data.user_info.uname
    superchat_msg.sender.face = msg.data.user_info.face
    superchat_msg.sender.medal_info = msg.data.medal_info
    superchat_msg.message = msg.data.message
    superchat_msg.price = msg.data.price
    superchat_msg.timestamp = msg.data.start_time

    this._window_manager.SendTo(
      WindowType.WMAIN,
      JEvent.EVENT_NEW_SUPER_CHAT,
      superchat_msg
    )
    this._window_manager.SendTo(
      WindowType.WSUPERCHAT,
      JEvent.EVENT_NEW_SUPER_CHAT,
      superchat_msg
    )
    // store superchat message
    this._gift_store.Push(superchat_msg)
  }

  private interactHandler(msg: any) {
    const interact_msg = new InteractMessage()
    interact_msg.sender = new Sender()
    interact_msg.sender.uid = msg.data.uid
    interact_msg.sender.uname = msg.data.uname
    interact_msg.sender.medal_info = msg.data.fans_medal
    interact_msg.action = msg.data.msg_type

    this._window_manager.SendTo(
      WindowType.WMAIN,
      JEvent.EVENT_NEW_INTERACT,
      interact_msg
    )
  }

  // msg handler for side connections, only handle danmu message for side connections
  private sideMsgHandlerConstructor(owner_info: MergeUserInfo) {
    return async function (packet: PackResult) {
      for (const msg of packet.body) {
        switch (msg.cmd) {
          case 'DANMU_MSG': {
            const danmu_msg = new DanmuMessage(msg, owner_info)
            this._window_manager.SendTo(
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
