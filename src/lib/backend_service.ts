import ElectronStore = require('electron-store')
import ReconnectingWebSocket from 'reconnecting-websocket'
import {
  connecting,
  checkLiveStatus,
  getRoomInfo,
  getOnlineNum,
  getGiftList,
  GetUserInfo,
  WsInfo,
  getDanmuInfo,
  DanmuSend,
  UpdateRoomTitle,
  StopLive,
} from './bilibili/message'
import JEvent from './events'
import JLogger from './logger'
import { WindowManager, WindowType } from './window_manager'

const log = JLogger.getInstance()

export default class BackendService {
  private _conn: ReconnectingWebSocket
  private _cookies: string
  private _window_manager: WindowManager

  public constructor() {}
  public async start(
    room: number,
    store: ElectronStore,
    window_manager: WindowManager
  ) {
    this._window_manager = window_manager
    log.info('Starting backend service', { room })
    this._cookies = store.get('config.cookies', '') as string
    if (this._cookies == '') {
      log.info('No avaliable cookies')
    }
    const status_response = await checkLiveStatus(room)
    const room_owner_uid = status_response.uid
    const room_real_id = status_response.room
    const room_response = await getRoomInfo(room_real_id)
    const online_response = await getOnlineNum(room_owner_uid, room_real_id)
    window_manager.sendTo(
      WindowType.WMAIN,
      JEvent[JEvent.EVENT_UPDATE_ROOM],
      room_response
    )
    window_manager.sendTo(
      WindowType.WMAIN,
      JEvent[JEvent.EVENT_UPDATE_ONLINE],
      online_response
    )
  }
}

async function startBackendService() {
  const giftList = new Map()
  const statusRes = await checkLiveStatus(room)
  realroom = statusRes.room
  uid = statusRes.uid
  const roomRes = await getRoomInfo(realroom)
  mainWindow?.webContents.send('update-room', roomRes)
  const onlineRes = await getOnlineNum(uid, realroom)
  mainWindow?.webContents.send('update-online', onlineRes)
  service.updateTask = setInterval(() => {
    getRoomInfo(realroom).then((res) => {
      mainWindow?.webContents.send('update-room', res)
    })
    getOnlineNum(uid, realroom).then((res) => {
      mainWindow?.webContents.send('update-online', res)
    })
  }, 10 * 1000)
  getGiftList(realroom).then((res: any) => {
    log.info('Get gift list info', { length: res.list.length })
    res.list.forEach((e: any) => {
      giftList.set(e.id, {
        animation_frame_num: e.animation_frame_num,
        png: e.frame_animation,
        gif: e.gif,
      })
    })
    Promise.all([loadPreGifts(), loadPreGuards(), loadPreSuperchats()]).then(
      async () => {
        // All Preload Data Loaded
        const msgHandler = function (type: number, msg: any) {
          switch (type) {
            case 3:
              mainWindow?.webContents.send('update-heat', msg)
              break
            case 5: {
              if (msg.cmd.includes('DANMU_MSG')) {
                mainWindow?.webContents.send('danmu', msg)
                break
              }
              if (msg.cmd.includes('SEND_GIFT')) {
                if (
                  msg.data.coin_type === 'silver' &&
                  msg.data.giftName == '辣条'
                ) {
                  break
                }
                let id = msg.data.batch_combo_id
                if (id === '') {
                  id = uuidv4()
                }
                if (msg.data.coin_type === 'gold') {
                  db.insertTableContent(
                    'gifts',
                    {
                      room: room,
                      sid: id,
                      data: msg,
                    },
                    () => {}
                  )
                }
                let giftInfo = {
                  animation_frame_num: 1,
                  png: '',
                  gif: '',
                }
                if (giftList.has(msg.data.giftId)) {
                  giftInfo = giftList.get(msg.data.giftId)
                }
                const giftData = {
                  gif: {
                    frame: giftInfo.animation_frame_num,
                    png: giftInfo.png,
                    gif: giftInfo.gif,
                  },
                  data: msg.data,
                }
                giftWindow?.webContents.send('gift', {
                  id: id,
                  msg: giftData,
                })
                mainWindow?.webContents.send('gift', {
                  id: id,
                  msg: giftData,
                })
                break
              }
              if (msg.cmd.includes('INTERACT_WORD')) {
                mainWindow?.webContents.send('interact', msg)
                break
              }
              if (msg.cmd.includes('GUARD_BUY')) {
                // getUserInfo(msg.data.uid).then((userinfo) => {
                const id = uuidv4()
                db.insertTableContent(
                  'guards',
                  {
                    room: room,
                    sid: id,
                    data: msg,
                  },
                  () => {}
                )
                const guardBuy = {
                  medal: msg.data.medal,
                  face: msg.data.face,
                  name: msg.data.username,
                  gift_name: msg.data.gift_name,
                  guard_level: msg.data.guard_level,
                  price: msg.data.price,
                  timestamp: msg.data.start_time,
                }
                giftWindow?.webContents.send('guard', {
                  id: id,
                  msg: guardBuy,
                })
                mainWindow?.webContents.send('guard', {
                  id: id,
                  msg: guardBuy,
                })
                // })
                break
              }
              if (msg.cmd == 'SUPER_CHAT_MESSAGE') {
                const id = uuidv4()
                db.insertTableContent(
                  'superchats',
                  {
                    room: room,
                    sid: id,
                    data: msg,
                  },
                  () => {}
                )
                superchatWindow?.webContents.send('superchat', {
                  id: id,
                  msg: msg,
                })
                mainWindow?.webContents.send('superchat', {
                  id: id,
                  msg: msg,
                })
                break
              }
              // 在线舰长数
              // { cmd: 'ONLINE_RANK_COUNT', data: { count: 22 } }
              if (msg.cmd.includes('ONLINE_RANK_COUNT')) {
                mainWindow?.webContents.send('online_guard', msg.data.count)
                break
              }
              // STOP_LIVE_ROOM_LIST 没啥用
              if (msg.cmd === 'STOP_LIVE_ROOM_LIST') {
                break
              }
              // ENTRY_EFFECT 舰长入场
              if (msg.cmd.includes('ENTRY_EFFECT')) {
                mainWindow?.webContents.send(
                  'entry-effect',
                  msg.data.copy_writing.replace(/(<%)|(%>)/g, '')
                )
                break
              }
            }
          }
        }
        const wsInfo = {} as WsInfo
        wsInfo.roomid = realroom
        if (cookies != '') {
          wsInfo.uid = Number(cookies['DedeUserID'])
        }
        const danmuInfo = await getDanmuInfo(cookies, realroom)
        if (danmuInfo['code'] != 0) {
          wsInfo.uid = 0
          wsInfo.server = 'wss://broadcastlv.chat.bilibili.com/sub'
          log.warn('Get room websocket info failed, using default setting', {
            wsInfo,
          })
        } else {
          wsInfo.token = danmuInfo['data']['token']
          wsInfo.server =
            'wss://' + danmuInfo['data']['host_list'][0]['host'] + '/sub'
          log.info('Get room websocket success', { wsInfo })
        }
        service.stopConn = connecting(wsInfo, msgHandler)
        // For debugging, if dev is true, then every 10 secs this will generate a gift message for displaying
        if (dev) {
          setInterval(() => {
            switch (Math.floor(Math.random() * 4)) {
              case 0: {
                msgHandler(5, SuperChatMockMessage)
                break
              }
              case 1: {
                msgHandler(5, GuardMockMessage)
                break
              }
              case 2: {
                msgHandler(5, DanmuMockMessage)
                break
              }
              case 3: {
                msgHandler(5, GiftMockMessage)
                break
              }
              default:
                break
            }
          }, 10 * 1000)
        }
      }
    )
  })
}

function stopBackendService() {
  if (service.updateTask) {
    clearInterval(service.updateTask)
    service.updateTask = null
  }
  if (service.stopConn) {
    service.stopConn()
    service.conn = null
  }
}
