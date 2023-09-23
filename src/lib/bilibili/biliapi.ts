import { Cookies } from '../types'
import { getInfoResponse } from './api/room/get_info'
import roomInitResponse from './api/room/room_init'

class BiliApi {
  public static async roomInit(
    cookies: Cookies,
    room: number
  ): Promise<roomInitResponse> {
    const url = `https://api.live.bilibili.com/room/v1/Room/room_init?id=${room}`
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as roomInitResponse
  }

  public static async getRoomInfo(
    cookies: Cookies,
    real_room: number
  ): Promise<getInfoResponse> {
    const url = `https://api.live.bilibili.com/room/v1/Room/get_info?id=${real_room}`
    const options = {
      method: 'GET',
      headers: {
        Cookie: cookies.str(),
      },
    }
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as getInfoResponse
  }
}

export default BiliApi
