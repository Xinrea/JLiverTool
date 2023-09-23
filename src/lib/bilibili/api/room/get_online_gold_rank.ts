import BasicResponse from '../basic_response'

export type GetOnlineGoldRankResponse = BasicResponse & {
  data: {
    onlineNum: number
    OnlineRankItem: Array<{
      userRank: number
      uid: number
      name: string
      face: string
      score: number
      medalInfo?: {
        guardLevel: number
        medalColorStart: number
        medalColorEnd: number
        medalColorBorder: number
        medalName: string
        level: number
        targetId: number
        isLight: number
      }
      guard_level: number
      wealth_level: number
    }>
    ownInfo: {
      uid: number
      name: string
      face: string
      rank: number
      needScore: number
      score: number
      guard_level: number
      wealth_level: number
    }
    tips_text: string
    value_text: string
    ab: {
      guard_accompany_list: number
    }
  }
}

export default GetOnlineGoldRankResponse
