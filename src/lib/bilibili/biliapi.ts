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
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as RoomInitResponse
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
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as GetInfoResponse
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
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as GiftConfigResponse
  }

  public static async GetDanmuInfo(
    cookies: Cookies,
    room: RoomID
  ): Promise<GetDanmuInfoResponse> {
    const url = `https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?id=${room.getRealID()}`
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as GetDanmuInfoResponse
  }

  public static async GetOnlineGoldRank(
    cookies: Cookies,
    room: RoomID
  ): Promise<GetOnlineGoldRankResponse> {
    const url = `https://api.live.bilibili.com/xlive/general-interface/v1/rank/getOnlineGoldRank?ruid=${room.getOwnerID()}&roomId=${room.getRealID()}&page=1&pageSize=1`
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as GetOnlineGoldRankResponse
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
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as UserInfoResponse
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
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as SendDanmuResponse
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
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as UpdateRoomTitleResponse
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
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as StopLiveResponse
  }

  public static async Nav(cookies: Cookies): Promise<NavResponse> {
    const url = 'https://api.bilibili.com/x/web-interface/nav'
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as NavResponse
  }
}

export default BiliApi
