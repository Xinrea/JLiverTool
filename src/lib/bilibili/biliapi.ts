import { Cookies } from '../types'
import GetInfoResponse from './api/room/get_info'
import RoomInitResponse from './api/room/room_init'
import GiftConfigResponse from './api/room/gift_config'
import GetDanmuInfoResponse from './api/room/get_danmuinfo'
import GetOnlineGoldRankResponse from './api/room/get_online_gold_rank'
import UserInfoResponse from './api/user/user_info'
import SendDanmuResponse from './api/room/send_danmu'
import UpdateRoomTitleResponse from './api/room/update_room_title'
import StopLiveResponse from './api/room/stop_live'

// WARN: All these api should be checked regularly, any api change will broke this tool
class BiliApi {
  public static async roomInit(
    cookies: Cookies,
    room: number
  ): Promise<RoomInitResponse> {
    const url = `https://api.live.bilibili.com/room/v1/Room/room_init?id=${room}`
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as RoomInitResponse
  }

  public static async getRoomInfo(
    cookies: Cookies,
    real_room: number
  ): Promise<GetInfoResponse> {
    const url = `https://api.live.bilibili.com/room/v1/Room/get_info?id=${real_room}`
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as GetInfoResponse
  }

  public static async getGiftConfig(
    cookies: Cookies,
    real_room: number
  ): Promise<GiftConfigResponse> {
    const url = `https://api.live.bilibili.com/xlive/web-room/v1/giftPanel/giftConfig?platform=pc&room_id=${real_room}`
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as GiftConfigResponse
  }

  public static async getDanmuInfo(
    cookies: Cookies,
    real_room: number
  ): Promise<GetDanmuInfoResponse> {
    const url = `https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo?id=${real_room}`
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as GetDanmuInfoResponse
  }

  public static async getOnlineGoldRank(
    cookies: Cookies,
    owner_id: number,
    real_room: number
  ): Promise<GetOnlineGoldRankResponse> {
    const url = `https://api.live.bilibili.com/xlive/general-interface/v1/rank/getOnlineGoldRank?ruid=${owner_id}&roomId=${real_room}&page=1&pageSize=1`
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as GetOnlineGoldRankResponse
  }

  public static async getUserInfo(
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
  public static async sendDanmu(
    cookies: Cookies,
    real_room: number,
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
    params.append('roomid', real_room.toString())
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

  public static async updateRoomTitle(
    cookies: Cookies,
    real_room: number,
    title: string
  ): Promise<UpdateRoomTitleResponse> {
    // Build form of room title updating
    const params = new URLSearchParams()
    params.append('platform', 'pc')
    params.append('room_id', real_room.toString())
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

  public static async stopRoomLive(
    cookies: Cookies,
    real_room: number
  ): Promise<StopLiveResponse> {
    // Build form of room title updating
    const params = new URLSearchParams()
    params.append('room_id', real_room.toString())
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
}

export default BiliApi
