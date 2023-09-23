import BasicResponse from '../basic_response'

export type UserInfoResponse = BasicResponse & {
  code: number
  data: {
    played_game_num: number
    booked_game_num: number
    commented_num: number
    gift_num: number
    up_count: number
    following_count: number
    follower_count: number
    strategy_article_count: number
    is_followed: boolean
    space_image: string
    block_status: string
    mid: string
    uname: string
    face: string
    sex: number
    level: number
    vip: {
      vip_type: number
      vip_status: number
    }
    official_verify: {
      type: number
      desc: string
    }
    attestation_display: {
      type: number
      desc: string
    }
  }
  ts: number
  request_id: string
}

export default UserInfoResponse
