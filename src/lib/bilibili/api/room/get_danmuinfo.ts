import BasicResponse from '../basic_response'

export type GetDanmuInfoResponse = BasicResponse & {
  data: {
    group: string
    business_id: number
    refresh_row_factor: number
    refresh_rate: number
    max_delay: number
    token: string
    host_list: Array<{
      host: string
      port: number
      wss_port: number
      ws_port: number
    }>
  }
}

export default GetDanmuInfoResponse
