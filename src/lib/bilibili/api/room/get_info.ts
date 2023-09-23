import BasicResponse from '../basic_response'

export type GetInfoResponse = BasicResponse & {
  data: {
    uid: number
    room_id: number
    short_id: number
    attention: number
    online: number
    is_portrait: boolean
    description: string
    live_status: number
    area_id: number
    parent_area_id: number
    parent_area_name: string
    old_area_id: number
    background: string
    title: string
    user_cover: string
    keyframe: string
    is_strict_room: boolean
    live_time: string
    tags: string
    is_anchor: number
    room_silent_type: string
    room_silent_level: number
    room_silent_second: number
    area_name: string
    pendants: string
    area_pendants: string
    hot_words: Array<string>
    hot_words_status: number
    verify: string
    new_pendants: {
      frame: {
        name: string
        value: string
        position: number
        desc: string
        area: number
        area_old: number
        bg_color: string
        bg_pic: string
        use_old_area: boolean
      }
      badge: {
        name: string
        position: number
        value: string
        desc: string
      }
      mobile_frame: {
        name: string
        value: string
        position: number
        desc: string
        area: number
        area_old: number
        bg_color: string
        bg_pic: string
        use_old_area: boolean
      }
      mobile_badge: any
    }
    up_session: string
    pk_status: number
    pk_id: number
    battle_id: number
    allow_change_area_time: number
    allow_upload_cover_time: number
    studio_info: {
      status: number
      master_list: Array<any>
    }
  }
}

export default GetInfoResponse
