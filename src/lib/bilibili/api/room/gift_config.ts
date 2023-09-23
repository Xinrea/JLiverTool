import BasicResponse from '../basic_response'

export type GiftConfigResponse = BasicResponse & {
  data: {
    list: Array<{
      id: number
      name: string
      price: number
      type: number
      coin_type: string
      bag_gift: number
      effect: number
      corner_mark: string
      corner_background: string
      broadcast: number
      draw: number
      stay_time: number
      animation_frame_num: number
      desc: string
      rule: string
      rights: string
      privilege_required: number
      count_map: Array<{
        num: number
        text: string
        desc: string
        web_svga: string
        vertical_svga: string
        horizontal_svga: string
        special_color: string
        effect_id: number
      }>
      img_basic: string
      img_dynamic: string
      frame_animation: string
      gif: string
      webp: string
      full_sc_web: string
      full_sc_horizontal: string
      full_sc_vertical: string
      full_sc_horizontal_svga: string
      full_sc_vertical_svga: string
      bullet_head: string
      bullet_tail: string
      limit_interval: number
      bind_ruid: number
      bind_roomid: number
      gift_type: number
      combo_resources_id: number
      max_send_limit: number
      weight: number
      goods_id: number
      has_imaged_gift: number
      left_corner_text: string
      left_corner_background: string
      gift_banner?: {
        app_pic: string
        web_pic: string
        left_text: string
        left_color: string
        button_text: string
        button_color: string
        button_pic_color: string
        jump_url: string
        jump_to: number
        web_pic_url: string
        web_jump_url: string
      }
      diy_count_map: number
      effect_id: number
      first_tips: string
      gift_attrs: Array<number>
      corner_mark_color: string
      corner_color_bg: string
      web_light: {
        corner_mark: string
        corner_background: string
        corner_mark_color: string
        corner_color_bg: string
      }
      web_dark: {
        corner_mark: string
        corner_background: string
        corner_mark_color: string
        corner_color_bg: string
      }
    }>
    combo_resources: Array<{
      combo_resources_id: number
      img_one: string
      img_two: string
      img_three: string
      img_four: string
      color_one: string
      color_two: string
    }>
    guard_resources: Array<{
      level: number
      img: string
      name: string
    }>
    naming_gift: {
      text: {
        app_user: string
        app_user_selected: string
        web_user: string
        web_user_selected: string
        combo_user: string
        combo_anchor: string
        vtr: string
      }
    }
    send_disable_msg: {
      gift_for_owner: string
      no_send_obj: string
      no_fans_incr: string
      jump_fans_url: string
      web_no_fans_incr: string
    }
  }
}

export default GiftConfigResponse
