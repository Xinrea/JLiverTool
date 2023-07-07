import WS = require('ws')
var FormData = require('form-data');
import ReconnectingWebSocket from 'reconnecting-websocket'
import pako = require('pako')
import https = require('https')
import { Cookies } from './types'

const textEncoder = new TextEncoder()
const textDecoder = new TextDecoder('utf-8')

const readInt = function (buffer, start, len): number {
  let result = 0
  for (let i = len - 1; i >= 0; i--) {
    result += Math.pow(256, len - i - 1) * buffer[start + i]
  }
  return result
}

const writeInt = function (buffer, start, len, value) {
  let i = 0
  while (i < len) {
    buffer[start + i] = value / Math.pow(256, len - i - 1)
    i++
  }
}

const encode = function (str, op) {
  let data = textEncoder.encode(str)
  let packetLen = 16 + data.byteLength
  let header = [0, 0, 0, 0, 0, 16, 0, 1, 0, 0, 0, op, 0, 0, 0, 1]
  writeInt(header, 0, 4, packetLen)
  return new Uint8Array(header.concat(...Array.from(data))).buffer
}

interface PackResult {
  packetLen: number,
  headerLen: number,
  ver: number,
  op: number,
  seq: number,
  body: any
}

const decode = function (blob): Promise<PackResult> {
  return new Promise(function (resolve, reject) {
    let buffer = new Uint8Array(blob)
    let result: PackResult = {
      packetLen: 0,
      headerLen: 0,
      ver: 0,
      op: 0,
      seq: 0,
      body: null
    }
    result.packetLen = readInt(buffer, 0, 4)
    result.headerLen = readInt(buffer, 4, 2)
    result.ver = readInt(buffer, 6, 2)
    result.op = readInt(buffer, 8, 4)
    result.seq = readInt(buffer, 12, 4)
    if (result.op === 5) {
      result.body = []
      if (result.ver === 0) {
        let data = buffer.slice(result.headerLen, result.packetLen)
        let body = textDecoder.decode(data)
        result.body.push(JSON.parse(body))
      } else if (result.ver === 2) {
        let newbuffer = pako.inflate(
          buffer.slice(result.headerLen, result.packetLen)
        )
        let offset = 0
        while (offset < newbuffer.length) {
          let packetLen = readInt(newbuffer, offset + 0, 4)
          let headerLen = 16 // readInt(buffer,offset + 4,4)
          let data = newbuffer.slice(offset + headerLen, offset + packetLen)
          /**
           *    引入pako做message解压处理，具体代码链接如下
           *    https://github.com/nodeca/pako/blob/master/dist/pako.js
           */
          let body = textDecoder.decode(data)
          if (body) {
            result.body.push(JSON.parse(body))
          }
          offset += packetLen
        }
      }
    } else if (result.op === 3) {
      result.body = {
        count: readInt(buffer, 16, 4)
      }
    }
    resolve(result)
  })
}

export interface WsInfo {
  server: string
  roomid: number
  uid: number
  token: string
}

export function getDanmuInfo(cookies, room) {
  // https://api.live.bilibili.com/xlive/web-room/v1/index/getDanmuInfo
  return new Promise(function (resolve, reject) {
    const options = {
      hostname: 'api.live.bilibili.com',
      path: `/xlive/web-room/v1/index/getDanmuInfo?id=${room}`,
      headers: {
        cookie: cookiesToString(cookies)
      }
    }
    https
      .get(
        options,
        (res) => {
          let data = ''
          res.on('data', (chunk) => {
            data += chunk
          })
          res.on('end', () => {
            const result = JSON.parse(data)
            resolve(result)
          })
        }
      )
      .on('error', (e) => {
        reject(e)
      })
  })
}
export function connecting(info: WsInfo, msgHandler) {
  console.log("Connecting", info.server)
  let authInfo = {
    uid: Number(info.uid),
    roomid: Number(info.roomid),
    protover: 2,
    type: 2,
    platform: 'web',
    key: info.token
  }
  const ws = new ReconnectingWebSocket(
    info.server,
    [],
    {
      WebSocket: WS
    }
  )
  ws.onopen = function () {
    ws.send(
      encode(
        JSON.stringify(authInfo),
        7
      )
    )
  }

  let heartBeatTask = setInterval(function () {
    ws.send(encode('', 2))
  }, 30000)

  ws.onmessage = async function (msgEvent) {
    const packet = await decode(msgEvent.data)
    switch (packet.op) {
      case 8:
        break
      case 3:
        msgHandler(3, packet.body.count)
        break
      case 5:
        packet.body.forEach((body) => {
          msgHandler(5, body)
        })
        break
      default:
        console.warn(packet)
    }
  }
  return function () {
    clearInterval(heartBeatTask)
    ws.close()
  }
}

export interface LiveStatus {
  status: boolean
  room: number
  uid: number
}

export function checkLiveStatus(room): Promise<LiveStatus> {
  return new Promise(function (resolve, reject) {
    https
      .get(
        `https://api.live.bilibili.com/room/v1/Room/room_init?id=${room}`,
        (res) => {
          let data = ''
          res.on('data', (chunk) => {
            data += chunk
          })
          res.on('end', () => {
            const result = JSON.parse(data)
            if (result.code === 0) {
              if (result.data.liveStatus === 1) {
                resolve({
                  status: true,
                  room: result.data.room_id,
                  uid: result.data.uid
                })
              } else {
                resolve({
                  status: false,
                  room: result.data.room_id,
                  uid: result.data.uid
                })
              }
            } else {
              resolve({
                status: false,
                room: 0,
                uid: 0
              })
            }
          })
        }
      )
      .on('error', (e) => {
        console.error(e)
      })
  })
}

export function getRoomInfo(room) {
  return new Promise(function (resolve, reject) {
    https
      .get(
        `https://api.live.bilibili.com/room/v1/Room/get_info?id=${room}`,
        (res) => {
          let data = ''
          res.on('data', (chunk) => {
            data += chunk
          })
          res.on('end', () => {
            const result = JSON.parse(data)
            if (result.code === 0) {
              resolve(result.data)
            }
          })
        }
      )
      .on('error', (e) => {
        console.error(e)
      })
  })
}

export function getGiftList(room) {
  return new Promise(function (resolve, reject) {
    https
      .get(
        `https://api.live.bilibili.com/xlive/web-room/v1/giftPanel/giftConfig?platform=pc&room_id=${room}`,
        (res) => {
          let data = ''
          res.on('data', (chunk) => {
            data += chunk
          })
          res.on('end', () => {
            const result = JSON.parse(data)
            if (result.code === 0) {
              resolve(result.data)
            }
          })
        }
      )
      .on('error', (e) => {
        console.error(e)
      })
  })
}

export function getOnlineNum(uid, room) {
  return new Promise(function (resolve, reject) {
    https
      .get(
        `https://api.live.bilibili.com/xlive/general-interface/v1/rank/getOnlineGoldRank?ruid=${uid}&roomId=${room}&page=1&pageSize=1`,
        (res) => {
          let data = ''
          res.on('data', (chunk) => {
            data += chunk
          })
          res.on('end', () => {
            const result = JSON.parse(data)
            if (result.code === 0) {
              resolve(result.data.onlineNum)
            }
          })
        }
      )
      .on('error', (e) => {
        console.error(e)
      })
  })
}

function cookiesToString(cookies: Cookies): string {
  return "SESSDATA=" + encodeURIComponent(cookies.SESSDATA)
    + "; DedeUserID=" + cookies.DedeUserID
    + "; DedeUserID_ckMd5=" + cookies.DedeUserID__ckMd5
    + "; bili_jct=" + cookies.bili_jct
    + "; Expires=" + cookies.Expires
}

export function GetUserInfo(cookies, mid) {
  return new Promise((resolve, reject) => {
    // https://line3-h5-mobile-api.biligame.com/game/center/h5/user/space/info?uid=475210&sdk_type=1
    let options = {
      hostname: 'line3-h5-mobile-api.biligame.com',
      path: `/game/center/h5/user/space/info?uid=${mid}&sdk_type=1`,
      port: 443,
      method: 'GET',
      headers: {
        'cookie': cookiesToString(cookies)
      }
    }
    let req = https.request(options, res => {
      let dd = ''
      res.on('data', chunk => {
        dd += chunk
      })
      res.on('end', () => {
        let resp = JSON.parse(dd.toString())
        if (resp.code === 0) {
          resolve(resp.data)
        } else {
          reject(resp)
        }
      })
      res.on('error', err => {
        reject(err)
      })
    })
    req.end()
  })
}

export function DanmuSend(cookies: Cookies, room, content) {
  // https://api.live.bilibili.com/msg/send
  return new Promise((resolve, reject) => {
    let formData = {};
    formData['bubble'] = 0
    formData['msg'] = content
    formData['color'] = 16777215
    formData['mode'] = 1
    formData['fontsize'] = 25
    formData['room_type'] = 0
    formData['rnd'] = Math.floor(Date.now() / 1000)
    formData['roomid'] = room
    formData['csrf'] = cookies.bili_jct
    formData['csrf_token'] = cookies.bili_jct
    const postData = new FormData();
    for (const key in formData) {
      if (Object.prototype.hasOwnProperty.call(formData, key)) {
        const item = formData[key];
        postData.append(key, item);
      }
    }
    let postOptions = {
      hostname: 'api.live.bilibili.com',
      path: '/msg/send',
      method: 'POST',
      port: 443,
      headers: {
        ...postData.getHeaders(),
        cookie: cookiesToString(cookies),
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36'
      }
    }
    let sendReq = https.request(postOptions, res => {
      let dd = ''
      res.on('data', secCheck => {
        dd += secCheck
      })
      res.on('end', () => {
        let resp = JSON.parse(dd)
        resolve(resp)
      })
      res.on('error', err => {
        console.error(err)
        reject(err)
      })
    })
    postData.pipe(sendReq)
    sendReq.end()
  })
}

export function UpdateRoomTitle(cookies: Cookies, room, title) {
  // https://api.live.bilibili.com/room/v1/Room/update
  return new Promise((resolve, reject) => {
    // platform: pc
    // room_id: 843610
    // title: test
    // csrf_token: 87007ec04d7cbedcd8122aaf4cd3b180
    // csrf: 87007ec04d7cbedcd8122aaf4cd3b180
    let formData = {};
    formData['platform'] = 'pc'
    formData['room_id'] = room
    formData['title'] = title
    formData['csrf'] = cookies.bili_jct
    formData['csrf_token'] = cookies.bili_jct
    var formBody = [];
    for (const key in formData) {
      if (Object.prototype.hasOwnProperty.call(formData, key)) {
        var encodedKey = encodeURIComponent(key);
        var encodedValue = encodeURIComponent(formData[key]);
        formBody.push(encodedKey + "=" + encodedValue);
      }
    }
    let postData = formBody.join("&");
    let postOptions = {
      hostname: 'api.live.bilibili.com',
      path: '/room/v1/Room/update',
      method: 'POST',
      port: 443,
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
        cookie: cookiesToString(cookies),
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36'
      }
    }
    let sendReq = https.request(postOptions, res => {
      let dd = ''
      res.on('data', secCheck => {
        dd += secCheck
      })
      res.on('end', () => {
        let resp = JSON.parse(dd)
        resolve(resp)
      })
      res.on('error', err => {
        console.error(err)
        reject(err)
      })
    })
    sendReq.write(postData)
    sendReq.end()
  })
}

export function StopLive(cookies: Cookies, room) {
  // https://api.live.bilibili.com/room/v1/Room/stopLive
  return new Promise((resolve, reject) => {
    let formData = {};
    formData['room_id'] = room
    formData['csrf'] = cookies.bili_jct
    formData['csrf_token'] = cookies.bili_jct
    var formBody = [];
    for (const key in formData) {
      if (Object.prototype.hasOwnProperty.call(formData, key)) {
        var encodedKey = encodeURIComponent(key);
        var encodedValue = encodeURIComponent(formData[key]);
        formBody.push(encodedKey + "=" + encodedValue);
      }
    }
    let postData = formBody.join("&");
    let postOptions = {
      hostname: 'api.live.bilibili.com',
      path: '/room/v1/Room/stopLive',
      method: 'POST',
      port: 443,
      headers: {
        'Content-Type': 'application/x-www-form-urlencoded',
        cookie: cookiesToString(cookies),
        'User-Agent': 'Mozilla/5.0 (Macintosh; Intel Mac OS X 10_15_7) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/114.0.0.0 Safari/537.36'
      }
    }
    let sendReq = https.request(postOptions, res => {
      let dd = ''
      res.on('data', secCheck => {
        dd += secCheck
      })
      res.on('end', () => {
        let resp = JSON.parse(dd)
        resolve(resp)
      })
      res.on('error', err => {
        console.error(err)
        reject(err)
      })
    })
    sendReq.write(postData)
    sendReq.end()
  })
}