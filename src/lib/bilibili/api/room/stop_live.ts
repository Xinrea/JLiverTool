import BasicResponse from '../basic_response'

export type StopLiveResponse = BasicResponse & {
  data: {
    change: number
    status: string
  }
}

export default StopLiveResponse
