export class AfdianAPI {
  public static async GetGoals(): Promise<GetGoalsResponse> {
    const url = `https://afdian.com/api/creator/get-goals?user_id=bbb3f596df9c11ea922752540025c377`
    const options = {
      method: 'GET',
    }
    const raw_response = await fetch(url, options)
    return (await raw_response.json()) as GetGoalsResponse
  }
}

export type GetGoalsResponse = {
  ec: number
  em: string
  data: {
    list: Array<{
      goal_id: string
      user_id: string
      status: number
      type: number
      desc: string
      monthly_fans: number
      monthly_income: string
      begin_time: number
      end_time: number
      sum_income: string
      percent: string
    }>
  }
}
