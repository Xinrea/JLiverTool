import BasicResponse from '../basic_response'

export type SendDanmuResponse = BasicResponse & {
  data: {
    mode_info: {
      mode: number
      show_player_type: number
      extra: string
    }
    dm_v2: string
  }
}

export default SendDanmuResponse
