import BasicResponse from '../basic_response'

export type StartLiveResponse = BasicResponse & {
  data: {
    change: number
    status: string
    room_type: number
    rtmp: {
      addr: string
      code: string
      new_link: string
      provider: string
    }
    protocols: Array<{
      protocol: string
      addr: string
      code: string
      new_link: string
      provider: string
    }>
    try_time: string
    live_key: string
    notice: {
      type: number
      status: number
      title: string
      msg: string
      button_text: string
      button_url: string
    }
  }
}

export default StartLiveResponse
