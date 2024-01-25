import BasicResponse from '../basic_response'

export type GetOnlineGoldRankResponse = BasicResponse & {
  data: {
    count: number
    item: Array<{
      uid: number
      name: string
      face: string
      rank: number
      score: number
      medal_info?: {
        guard_level: number
        medal_color_start: number
        medal_color_end: number
        medal_color_border: number
        medal_name: string
        level: number
        target_id: number
        is_light: number
      }
      guard_level: number
      wealth_level: number
      is_mystery: boolean
      uinfo: {
        uid: number
        base: {
          name: string
          face: string
          name_color: number
          is_mystery: boolean
          risk_ctrl_info: {
            name: string
            face: string
          }
          origin_info: {
            name: string
            face: string
          }
          official_info: {
            role: number
            title: string
            desc: string
            type: number
          }
        }
        medal?: {
          name: string
          level: number
          color_start: number
          color_end: number
          color_border: number
          color: number
          id: number
          typ: number
          is_light: number
          ruid: number
          guard_level: number
          score: number
          guard_icon: string
          honor_icon: string
        }
        wealth: {
          level: number
          dm_icon_key: string
        }
        title: {
          old_title_css_id: string
          title_css_id: string
        }
        guard: {
          level: number
          expired_str: string
        }
        uhead_frame: any
        guard_leader: any
      }
    }>
    own_info: {
      uid: number
      name: string
      face: string
      rank: number
      score: number
      rank_text: string
      need_score: number
      medal_info: {
        guard_level: number
        medal_color_start: number
        medal_color_end: number
        medal_color_border: number
        medal_name: string
        level: number
        target_id: number
        is_light: number
      }
      guard_level: number
      wealth_level: number
      score_lead: number
      score_behind: number
      is_mystery: boolean
      uinfo: {
        uid: number
        base: {
          name: string
          face: string
          name_color: number
          is_mystery: boolean
          risk_ctrl_info: {
            name: string
            face: string
          }
          origin_info: {
            name: string
            face: string
          }
          official_info: {
            role: number
            title: string
            desc: string
            type: number
          }
        }
        medal: {
          name: string
          level: number
          color_start: number
          color_end: number
          color_border: number
          color: number
          id: number
          typ: number
          is_light: number
          ruid: number
          guard_level: number
          score: number
          guard_icon: string
          honor_icon: string
        }
        wealth: {
          level: number
          dm_icon_key: string
        }
        title: {
          old_title_css_id: string
          title_css_id: string
        }
        guard: {
          level: number
          expired_str: string
        }
        uhead_frame: any
        guard_leader: any
      }
    }
    tips_text: string
    count_format: string
    desc_format: string
    config: {
      deadline_ts: number
      value_text: string
    }
  }
}

export default GetOnlineGoldRankResponse
