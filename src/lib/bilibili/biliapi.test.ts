import BiliApi from './biliapi'
import { Cookies } from '../types'
import { readFileSync } from 'fs'
require('chai').should()

function readTestingCookies(): Cookies {
  const rawdata = readFileSync('.cookiestest')
  const cookies = new Cookies()
  cookies.fromJSON(JSON.parse(rawdata.toString()))
  return cookies
}

describe('BiliApi', function () {
  describe('#roomInit', function () {
    it('should get response with correct value', async function () {
      const resp = await BiliApi.roomInit(new Cookies(), 21484828)
      resp.code.should.eq(0)
      resp.data.room_id.should.eq(21484828)
      resp.data.uid.should.eq(61639371)
    })
  })
  describe('#getRoomInfo', function () {
    it('should get response with correct value', async function () {
      const resp = await BiliApi.getRoomInfo(new Cookies(), 21484828)
      resp.code.should.eq(0)
      resp.data.room_id.should.eq(21484828)
      resp.data.uid.should.eq(61639371)
    })
  })
  describe('#giftConfig', function () {
    it('should get response with correct value', async function () {
      const resp = await BiliApi.getGiftConfig(new Cookies(), 21484828)
      resp.code.should.eq(0)
    })
  })
  describe('#getDanmuInfo', function () {
    it('should get response with correct value', async function () {
      const resp = await BiliApi.getDanmuInfo(new Cookies(), 21484828)
      resp.code.should.eq(0)
      resp.data.host_list.length.should.gt(0)
    })
  })
  describe('#getOnlineGoldRank', function () {
    it('should get response with correct value', async function () {
      const resp = await BiliApi.getOnlineGoldRank(
        new Cookies(),
        61639371,
        21484828
      )
      resp.code.should.eq(0)
    })
  })
  describe('#getUserInfo', function () {
    it('should get response with correct value', async function () {
      const resp = await BiliApi.getUserInfo(new Cookies(), 475210)
      resp.code.should.eq(0)
      resp.data.level.should.eq(6)
      resp.data.sex.should.eq(1)
    })
  })
  describe('#sendDanmu', function () {
    it('should send danmu successfully with valid cookies', async function () {
      const cookies = readTestingCookies()
      const resp = await BiliApi.sendDanmu(
        cookies,
        843610,
        'test from jlivertool'
      )
      resp.code.should.eq(0)
    })
  })
  describe('#updateRoomTitle', function () {
    it('should update title successfully with valid cookies', async function () {
      const cookies = readTestingCookies()
      const resp = await BiliApi.updateRoomTitle(
        cookies,
        843610,
        'test' + Math.floor(Date.now() / 1000).toString()
      )
      resp.code.should.eq(0)
    })
  })
  describe('#stopRoomLive', function () {
    it('should stop live successfully with valid cookies', async function () {
      const cookies = readTestingCookies()
      const resp = await BiliApi.stopRoomLive(cookies, 843610)
      resp.code.should.eq(0)
    })
  })
})
