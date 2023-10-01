export class Cookies {
  DedeUserID: string
  DedeUserID__ckMd5: string
  Expires: string
  SESSDATA: string
  bili_jct: string
  gourl: string

  public constructor() {}
  public fromJSON(object: Object): Cookies {
    this.DedeUserID = object['DedeUserID']
    this.DedeUserID__ckMd5 = object['DedeUserID__ckMd5']
    this.Expires = object['Expires']
    this.SESSDATA = object['SESSDATA']
    this.bili_jct = object['bili_jct']
    this.gourl = object['gourl']
    return this
  }
  public str(): string {
    // If SESSDATA contains %, then it has already been encoded.
    if (this.SESSDATA == undefined) {
      return ''
    }
    return (
      'SESSDATA=' +
      (this.SESSDATA.includes('%')
        ? encodeURIComponent(this.SESSDATA)
        : this.SESSDATA) +
      '; DedeUserID=' +
      this.DedeUserID +
      '; DedeUserID_ckMd5=' +
      this.DedeUserID__ckMd5 +
      '; bili_jct=' +
      this.bili_jct +
      '; Expires=' +
      this.Expires
    )
  }
}

export type DBCondition = {
  room?: number
  sid?: string
}

export type Sender = {
  uid: number
  uname: string
  face: string
  medal_info: MedalInfo
}

export type MedalInfo = {
  anchor_roomid: number
  anchor_uname: string
  guard_level: number
  icon_id: number
  is_lighted: number
  medal_color: number
  medal_color_border: number
  medal_color_end: number
  medal_color_start: number
  medal_level: number
  medal_name: string
  special: string
  target_id: number
}

export type Gift = {
  id: number
  room: number
  name: string
  price: number
  coin_type: string
  animation_frame_num: number
  png: string
  gif: string
  sender: Sender
}

export type Guard = {
  id: number
  room: number
  level: number
  price: number
  sender: Sender
}

export type SuperChat = {
  id: number
  room: number
  level: number
  message: string
  start_time: number
  end_time: number
  price: number
  sender: Sender
}
