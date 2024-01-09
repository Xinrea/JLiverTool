export function typecast(Class, obj) {
  let t = new Class()
  return Object.assign(t, obj)
}

export class Cookies {
  DedeUserID: string
  DedeUserID__ckMd5: string
  Expires: string
  SESSDATA: string
  bili_jct: string
  gourl: string

  public constructor(object: Object) {
    this.DedeUserID = object['DedeUserID']
    this.DedeUserID__ckMd5 = object['DedeUserID__ckMd5']
    this.Expires = object['Expires']
    this.SESSDATA = object['SESSDATA']
    this.bili_jct = object['bili_jct']
    this.gourl = object['gourl']
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

export type MergeUserInfo = {
  index: number
  uid: string
  name: string
}

export class RoomID {
  // user may using short_id as room_id
  private short_id: number
  private room_id: number
  private owner_uid: number
  public constructor(short_id: number, room_id: number, owner_uid: number) {
    this.short_id = short_id
    this.room_id = room_id
    this.owner_uid = owner_uid
  }

  public same(room_id: number): boolean {
    return this.short_id === room_id || this.room_id === room_id
  }

  public equals(room: RoomID): boolean {
    return this.short_id === room.short_id && this.room_id === room.room_id
  }

  public getID(): number {
    if (this.short_id !== 0) {
      return this.short_id
    }
    return this.room_id
  }

  public getRealID(): number {
    return this.room_id
  }

  public getOwnerID(): number {
    return this.owner_uid
  }
}

export let DefaultRoomID = new RoomID(0, 21484828, 61639371)

export type DBCondition = {
  room?: number
  sid?: string
}

export class Sender {
  uid: number
  uname: string
  face: string
  medal_info: MedalInfo = new MedalInfo()
}

export class MedalInfo {
  anchor_roomid: number
  anchor_uname: string
  guard_level: number
  medal_color: number
  medal_color_border: number
  medal_color_start: number
  medal_color_end: number
  medal_level: number
  medal_name: string
  is_lighted: number
}

export type EmojiContent = {
  bulge_display: number
  emoticon_unique: string
  height: number
  in_player_area: number
  is_dynamic: number
  url: string
  width: number
}

export enum WindowType {
  WINVALID = 'invalid',
  WMAIN = 'main',
  WGIFT = 'gift',
  WSUPERCHAT = 'superchat',
  WSETTING = 'setting',
  WDETAIL = 'detail',
}

export enum RecordType {
  DANMU = 0,
  GIFT = 1,
  SUPERCHAT = 2,
  GUARD = 3,
  INTERACT = 4,
}

export type DanmuRecord = {
  type: RecordType
  content: string
  timestamp: number
}

export type DetailInfo = {
  sender: Sender
  danmus: DanmuRecord[]
}

export type DmExtraInfo = {
  send_from_me: boolean
  mode: number
  color: number
  dm_type: number
  font_size: number
  player_mode: number
  show_player_type: number
  content: string
  user_hash: string
  emoticon_unique: string
  bulge_display: number
  recommend_score: number
  main_state_dm_color: string
  objective_state_dm_color: string
  direction: number
  pk_direction: number
  quartet_direction: number
  anniversary_crowd: number
  yeah_space_type: string
  yeah_space_url: string
  jump_to_url: string
  space_type: string
  space_url: string
  animation: {}
  emots: any
  is_audited: boolean
  id_str: string
  icon: any
  show_reply: boolean
  reply_mid: number
  reply_uname: string
  reply_uname_color: string
  reply_is_mystery: boolean
  hit_combo: number
}
