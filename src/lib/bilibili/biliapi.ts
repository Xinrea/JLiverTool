import { Cookies, RoomID } from '../types'
import GetInfoResponse from './api/room/get_info'
import RoomInitResponse from './api/room/room_init'
import GiftConfigResponse from './api/room/gift_config'
import GetDanmuInfoResponse from './api/room/get_danmuinfo'
import GetOnlineGoldRankResponse from './api/room/get_online_gold_rank'
import UserInfoResponse from './api/user/user_info'
import SendDanmuResponse from './api/room/send_danmu'
import UpdateRoomTitleResponse from './api/room/update_room_title'
import StopLiveResponse from './api/room/stop_live'
import { NavResponse } from './api/nav_response'
import StartLiveResponse from './api/room/start_live'
import JLogger from '../logger'
import wbi_sign from './wbi'

const log = JLogger.getInstance('biliapi')

// WARN: All these api should be checked regularly, any api change will broke this tool
class BiliApi {
  public static async RoomInit(
    cookies: Cookies,
    room: RoomID
  ): Promise<RoomInitResponse> {
    const url = `https://api.live.bilibili.com/room/v1/Room/room_init?id=${room.getRealID()}`
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    try {
      const raw_response = await fetch(url, options)
      return (await raw_response.json()) as RoomInitResponse
    } catch (e) {
      log.error(`RoomInit failed: ${e}`)
      return null
    }
  }

  public static async GetRoomInfo(
    cookies: Cookies,
    room: RoomID
  ): Promise<GetInfoResponse> {
    const url = `https://api.live.bilibili.com/room/v1/Room/get_info?id=${room.getRealID()}`
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    try {
      const raw_response = await fetch(url, options)
      return (await raw_response.json()) as GetInfoResponse
    } catch (e) {
      log.error(`GetRoomInfo failed: ${e}`)
      return null
    }
  }

  public static async GetGiftConfig(
    cookies: Cookies,
    room: RoomID
  ): Promise<GiftConfigResponse> {
    const url = `https://api.live.bilibili.com/xlive/web-room/v1/giftPanel/giftConfig?platform=pc&room_id=${room.getRealID()}`
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    try {
      const raw_response = await fetch(url, options)
      return (await raw_response.json()) as GiftConfigResponse
    } catch (e) {
      log.error(`GetGiftConfig failed: ${e}`)
      return null
    }
  }

  public static async GetDanmuInfo(
    cookies: Cookies,
    room: RoomID
  ): Promise<GetDanmuInfoResponse> {
    let param = {
      id: room.getRealID(),
    }
    const url = `https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?${await wbi_sign(
      param
    )}`
    log.info(`GetDanmuInfo url: ${url}`)
    // generate uuid for buvid3
    const uuid = crypto.randomUUID()
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str() + `; buvid3=${uuid}`,
        'User-Agent':
          'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/120.0.0.0 Safari/537.36',
      },
    }
    try {
      const raw_response = await fetch(url, options)
      const resp = (await raw_response.json()) as GetDanmuInfoResponse
      return resp
    } catch (e) {
      log.error(`GetDanmuInfo failed: ${e}`)
      return null
    }
  }

  public static async GetOnlineGoldRank(
    cookies: Cookies,
    room: RoomID,
    page: number,
    pageSize: number
  ): Promise<GetOnlineGoldRankResponse> {
    const url = `https://api.live.bilibili.com/xlive/general-interface/v1/rank/queryContributionRank?ruid=${room.getOwnerID()}&room_id=${room.getRealID()}&page=${page}&page_size=${pageSize}&type=online_rank&switch=contribution_rank`
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    try {
      const raw_response = await fetch(url, options)
      return (await raw_response.json()) as GetOnlineGoldRankResponse
    } catch (e) {
      log.error(`GetOnlineGoldRank failed: ${e}`)
      return null
    }
  }

  public static async GetUserInfo(
    cookies: Cookies,
    uid: number
  ): Promise<UserInfoResponse> {
    const url = `https://line3-h5-mobile-api.biligame.com/game/center/h5/user/space/info?uid=${uid}&sdk_type=1`
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    try {
      const raw_response = await fetch(url, options)
      return (await raw_response.json()) as UserInfoResponse
    } catch (e) {
      log.error(`GetUserInfo failed: ${e}`)
      return null
    }
  }

  /** Valid cookies must be provided to send danmu. */
  public static async SendDanmu(
    cookies: Cookies,
    room: RoomID,
    content: string
  ): Promise<SendDanmuResponse> {
    // Build form of danmu message
    const params = new URLSearchParams()
    params.append('bubble', '0')
    params.append('msg', content)
    params.append('color', '16777215')
    params.append('mode', '1')
    params.append('fontsize', '25')
    params.append('room_type', '0')
    params.append('rnd', Math.floor(Date.now() / 1000).toString())
    params.append('roomid', room.getRealID().toString())
    params.append('csrf', cookies.bili_jct)
    params.append('csrf_token', cookies.bili_jct)

    const url = `https://api.live.bilibili.com/msg/send`
    const options = {
      method: 'POST',
      headers: {
        Cookie: cookies.str(),
      },
      body: params,
    }
    try {
      const raw_response = await fetch(url, options)
      return (await raw_response.json()) as SendDanmuResponse
    } catch (e) {
      log.error(`SendDanmu failed: ${e}`)
      return null
    }
  }

  public static async UpdateRoomTitle(
    cookies: Cookies,
    room: RoomID,
    title: string
  ): Promise<UpdateRoomTitleResponse> {
    // Build form of room title updating
    const params = new URLSearchParams()
    params.append('platform', 'pc')
    params.append('room_id', room.getRealID().toString())
    params.append('title', title)
    params.append('csrf', cookies.bili_jct)
    params.append('csrf_token', cookies.bili_jct)

    const url = `https://api.live.bilibili.com/room/v1/Room/update`
    const options = {
      method: 'POST',
      headers: {
        Cookie: cookies.str(),
      },
      body: params,
    }
    try {
      const raw_response = await fetch(url, options)
      return (await raw_response.json()) as UpdateRoomTitleResponse
    } catch (e) {
      log.error(`UpdateRoomTitle failed: ${e}`)
      return null
    }
  }

  public static async StartRoomLive(
    cookies: Cookies,
    room: RoomID,
    area_v2: string
  ): Promise<StartLiveResponse> {
    // Build form of room title updating
    const post_data = new URLSearchParams()
    post_data.append('room_id', room.getRealID().toString())
    post_data.append('platform', 'pc')
    post_data.append('area_v2', area_v2)
    post_data.append('csrf', cookies.bili_jct)
    post_data.append('csrf_token', cookies.bili_jct)
    // post form data
    const url = `https://api.live.bilibili.com/room/v1/Room/startLive`
    const options = {
      method: 'POST',
      headers: {
        Cookie: cookies.str(),
      },
      body: post_data,
    }
    try {
      const raw_response = await fetch(url, options)
      return (await raw_response.json()) as StartLiveResponse
    } catch (e) {
      log.error(`StartRoomLive failed: ${e}`)
      return null
    }
  }

  public static async StopRoomLive(
    cookies: Cookies,
    room: RoomID
  ): Promise<StopLiveResponse> {
    // Build form of room title updating
    const params = new URLSearchParams()
    params.append('room_id', room.getRealID().toString())
    params.append('csrf', cookies.bili_jct)
    params.append('csrf_token', cookies.bili_jct)

    const url = `https://api.live.bilibili.com/room/v1/Room/stopLive`
    const options = {
      method: 'POST',
      headers: {
        Cookie: cookies.str(),
      },
      body: params,
    }
    try {
      const raw_response = await fetch(url, options)
      return (await raw_response.json()) as StopLiveResponse
    } catch (e) {
      log.error(`StopRoomLive failed: ${e}`)
      return null
    }
  }

  public static async Nav(cookies: Cookies): Promise<NavResponse> {
    const url = 'https://api.bilibili.com/x/web-interface/nav'
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    try {
      const raw_response = await fetch(url, options)
      return (await raw_response.json()) as NavResponse
    } catch (e) {
      log.error(`Nav failed: ${e}`)
      return null
    }
  }
}

export default BiliApi
