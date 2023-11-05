import BasicResponse from './basic_response'

export type NavResponse = BasicResponse & {
  data: {
    isLogin: boolean
    face: string
    mid: string
    uname: string
  }
}
