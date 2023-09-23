export class Cookies {
  DedeUserID: string
  DedeUserID__ckMd5: string
  Expires: string
  SESSDATA: string
  bili_jct: string
  gourl: string

  public constructor() {}
  public str(): string {
    return (
      'SESSDATA=' +
      encodeURIComponent(this.SESSDATA) +
      '; DedeUserID=' +
      this.DedeUserID +
      '; DedeUserID_ckMd5=' +
      this.DedeUserID__ckMd5 +
      '; bili_jct=' +
      this.bili_jct +
      '; Expires=' +
      this.Expires
    )
  }
}

export type DBCondition = {
  room?: number
  sid?: string
}
