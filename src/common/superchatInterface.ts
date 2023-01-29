export interface SuperChat {
  cmd: string
  data: Data
  roomid: number
}

export interface Data {
  background_bottom_color: string
  background_color: string
  background_color_end: string
  background_color_start: string
  background_icon: string
  background_image: string
  background_price_color: string
  color_point: number
  dmscore: number
  end_time: number
  gift: Gift
  id: number
  is_ranked: number
  is_send_audit: number
  medal_info: any
  message: string
  message_font_color: string
  message_trans: string
  price: number
  rate: number
  start_time: number
  time: number
  token: string
  trans_mark: number
  ts: number
  uid: number
  user_info: UserInfo
}

export interface Gift {
  gift_id: number
  gift_name: string
  num: number
}

export interface UserInfo {
  face: string
  face_frame: string
  guard_level: number
  is_main_vip: number
  is_svip: number
  is_vip: number
  level_color: string
  manager: number
  name_color: string
  title: string
  uname: string
  user_level: number
}