import { ipcMain } from 'electron'
import JEvent from './events'
import JLogger from './logger'
import { BiliWebSocket, PackResult } from './bilibili/biliws'
import { WindowManager, WindowType } from './window_manager'
import BiliApi from './bilibili/biliapi'
import { GiftStore } from './gift_store'
import { Cookies } from './types'
import ConfigStore from './config_store'
import { MessageDanmu } from './messages'

const log = JLogger.getInstance('backend_service')

function CreateIntervalTask(f: Function, interval: number) {
  f()
  return setInterval(f, interval)
}

export default class BackendService {
  private _conn: BiliWebSocket
  private _cookies: Cookies
  private _window_manager: WindowManager
  private _owner_uid: number
  private _real_room: number

  private _task_update_room_info: number
  private _task_update_online_num: number

  private _gift_list_cache: Map<number, any> = new Map()
  private _gift_store: GiftStore = new GiftStore()

  private _window_ready: boolean[] = [false, false, false]

  public constructor() {}

  public async Start(
    room: number,
    store: ConfigStore,
    window_manager: WindowManager
  ) {
    log.info('Starting backend service', { room })
    this._window_manager = window_manager
    this._cookies = store.Cookies
    log.info('Loading cookies', { uid: this._cookies.DedeUserID })

    const status_response = await BiliApi.RoomInit(this._cookies, room)
    this._owner_uid = status_response.data.uid
    this._real_room = status_response.data.room_id

    // Must update gift list before recieving any gift message
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
    const danmu_server_info = await BiliApi.GetDanmuInfo(
      this._cookies,
      this._real_room
    )
    this._conn = new BiliWebSocket({
      roomid: this._real_room,
      uid: parseInt(this._cookies.DedeUserID),
      server: `wss://${danmu_server_info.data.host_list[0].host}/sub`,
      token: danmu_server_info.data.token,
    })
    this._conn.msg_handler = this.msgHandler.bind(this)
    this._conn.Connect(true)
  }

  public async Stop() {
    log.info('Stopping backend service')
    clearInterval(this._task_update_room_info)
    clearInterval(this._task_update_online_num)
    this._conn.Disconnect()
  }

  private async updateRoomInfo() {
    log.debug('Updating basic room info')
    const room_response = await BiliApi.GetRoomInfo(
      this._cookies,
      this._real_room
    )
    this._window_manager.sendTo(
      WindowType.WMAIN,
      JEvent.EVENT_UPDATE_ROOM,
      room_response.data
    )
  }

  private async updateOnlineNum() {
    const online_response = await BiliApi.GetOnlineGoldRank(
      this._cookies,
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
      this._cookies,
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
    ipcMain.on(JEvent[JEvent.EVENT_WINDOW_READY], (_, wtype: WindowType) => {
      this._window_ready[wtype] = true
    })
    // only used in main window
    ipcMain.handle(JEvent[JEvent.INVOKE_WINDOW_MINIMIZE], (_) => {
      this._window_manager.minimize(WindowType.WMAIN)
    })
  }

  private msgHandler(packet: PackResult) {
    for (const msg of packet.body) {
      log.debug('Recieved message', { msg })
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
