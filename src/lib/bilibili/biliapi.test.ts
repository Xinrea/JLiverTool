import BiliApi from './biliapi'
import { Cookies } from '../types'
require('chai').should()

describe('BiliApi', function () {
  describe('#roomInit()', function () {
    it('should get response with correct value', async function () {
      const resp = await BiliApi.roomInit(new Cookies(), 21484828)
      resp.code.should.eq(0)
      resp.data.room_id.should.eq(21484828)
      resp.data.uid.should.eq(61639371)
    })
  })
  describe('#getRoomInfo', function () {
    it('should get response with correct value', async function () {
      const resp = await BiliApi.getRoomInfo(new Cookies, 21484828)
      resp.code.should.eq(0)
      resp.data.room_id.should.eq(21484828)
      resp.data.uid.should.eq(61639371)
    })
  })
})
