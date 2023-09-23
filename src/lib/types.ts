export class Cookies {
  DedeUserID: string
  DedeUserID__ckMd5: string
  Expires: string
  SESSDATA: string
  bili_jct: string
  gourl: string

  public constructor() {}
  public fromJSON(object: Object) {
    this.DedeUserID = object['DedeUserID']
    this.DedeUserID__ckMd5 = object['DedeUserID__ckMd5']
    this.Expires = object['Expires']
    this.SESSDATA = object['SESSDATA']
    this.bili_jct = object['bili_jct']
    this.gourl = object['gourl']
  }
  public str(): string {
    // If SESSDATA contains %, then it has already been encoded.
    if (this.SESSDATA == undefined) {
      return ''
    }
    return (
      'SESSDATA=' +
      (this.SESSDATA.includes('%')
        ? encodeURIComponent(this.SESSDATA)
        : this.SESSDATA) +
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
