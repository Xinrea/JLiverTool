import BiliApi from './biliapi'
import { Cookies, DefaultRoomID } from '../types'
import { readFileSync } from 'fs'
require('chai').should()

function readTestingCookies(): Cookies {
  const rawdata = readFileSync('.cookiestest')
  const cookies = new Cookies(JSON.parse(rawdata.toString()))
  return cookies
}

describe('BiliApi', function () {
  describe('#roomInit', function () {
    it('🤔should get response with correct value', async function () {
      const resp = await BiliApi.RoomInit(new Cookies({}), DefaultRoomID)
      resp.code.should.eq(0)
      resp.data.room_id.should.eq(21484828)
      resp.data.uid.should.eq(61639371)
    })
  })
  describe('#getRoomInfo', function () {
    it('🤔should get response with correct value', async function () {
      const resp = await BiliApi.GetRoomInfo(new Cookies({}), DefaultRoomID)
      resp.code.should.eq(0)
      resp.data.room_id.should.eq(21484828)
      resp.data.uid.should.eq(61639371)
    })
  })
  describe('#giftConfig', function () {
    it('🤔should get response with correct value', async function () {
      const resp = await BiliApi.GetGiftConfig(new Cookies({}), DefaultRoomID)
      resp.code.should.eq(0)
    })
  })
  describe('#getDanmuInfo', function () {
    it('🤔should get response with correct value', async function () {
      const resp = await BiliApi.GetDanmuInfo(new Cookies({}), DefaultRoomID)
      resp.code.should.eq(0)
      resp.data.host_list.length.should.gt(0)
    })
  })
  describe('#getOnlineGoldRank', function () {
    it('🤔should get response with correct value', async function () {
      const resp = await BiliApi.GetOnlineGoldRank(
        new Cookies({}),
        DefaultRoomID
      )
      resp.code.should.eq(0)
    })
  })
  describe('#getUserInfo', function () {
    it('🤔should get response with correct value', async function () {
      const resp = await BiliApi.GetUserInfo(new Cookies({}), 475210)
      resp.code.should.eq(0)
      resp.data.level.should.eq(6)
      resp.data.sex.should.eq(1)
    })
  })
  describe('#sendDanmu', function () {
    it('🤔should send danmu successfully with valid cookies', async function () {
      const cookies = readTestingCookies()
      const resp = await BiliApi.SendDanmu(
        cookies,
        DefaultRoomID,
        'test from jlivertool'
      )
      resp.code.should.eq(0)
    })
  })
  describe('#updateRoomTitle', function () {
    it('🤔should update title successfully with valid cookies', async function () {
      const cookies = readTestingCookies()
      const resp = await BiliApi.UpdateRoomTitle(
        cookies,
        DefaultRoomID,
        'test' + Math.floor(Date.now() / 1000).toString()
      )
      resp.code.should.eq(0)
    })
  })
  describe('#stopRoomLive', function () {
    it('🤔should stop live successfully with valid cookies', async function () {
      const cookies = readTestingCookies()
      const resp = await BiliApi.StopRoomLive(cookies, DefaultRoomID)
      resp.code.should.eq(0)
    })
  })
})
