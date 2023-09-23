import WS = require('ws')
import ReconnectingWebSocket from 'reconnecting-websocket'
import pako = require('pako')
import { Cookies } from '../types'

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
  const data = textEncoder.encode(str)
  const packetLen = 16 + data.byteLength
  const header = [0, 0, 0, 0, 0, 16, 0, 1, 0, 0, 0, op, 0, 0, 0, 1]
  writeInt(header, 0, 4, packetLen)
  return new Uint8Array(header.concat(...Array.from(data))).buffer
}

type PackResult = {
  packetLen: number
  headerLen: number
  ver: number
  op: number
  seq: number
  body: any
}

const decode = function (blob: any): Promise<PackResult> {
  return new Promise(function (resolve, reject) {
    const buffer = new Uint8Array(blob)
    const result: PackResult = {
      packetLen: 0,
      headerLen: 0,
      ver: 0,
      op: 0,
      seq: 0,
      body: null,
    }
    result.packetLen = readInt(buffer, 0, 4)
    result.headerLen = readInt(buffer, 4, 2)
    result.ver = readInt(buffer, 6, 2)
    result.op = readInt(buffer, 8, 4)
    result.seq = readInt(buffer, 12, 4)
    if (result.op === 5) {
      result.body = []
      if (result.ver === 0) {
        const data = buffer.slice(result.headerLen, result.packetLen)
        const body = textDecoder.decode(data)
        result.body.push(JSON.parse(body))
      } else if (result.ver === 2) {
        const newbuffer = pako.inflate(
          buffer.slice(result.headerLen, result.packetLen)
        )
        let offset = 0
        while (offset < newbuffer.length) {
          const packetLen = readInt(newbuffer, offset + 0, 4)
          const headerLen = 16 // readInt(buffer,offset + 4,4)
          const data = newbuffer.slice(offset + headerLen, offset + packetLen)
          /**
           *    引入pako做message解压处理，具体代码链接如下
           *    https://github.com/nodeca/pako/blob/master/dist/pako.js
           */
          const body = textDecoder.decode(data)
          if (body) {
            result.body.push(JSON.parse(body))
          }
          offset += packetLen
        }
      }
    } else if (result.op === 3) {
      result.body = {
        count: readInt(buffer, 16, 4),
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
        cookie: cookiesToString(cookies),
      },
    }
    https
      .get(options, (res) => {
        let data = ''
        res.on('data', (chunk) => {
          data += chunk
        })
        res.on('end', () => {
          const result = JSON.parse(data)
          resolve(result)
        })
      })
      .on('error', (e) => {
        reject(e)
      })
  })
}
export function connecting(info: WsInfo, msgHandler: Function) {
  console.log('Connecting', info.server)
  const authInfo = {
    uid: Number(info.uid),
    roomid: Number(info.roomid),
    protover: 2,
    type: 2,
    platform: 'web',
    key: info.token,
  }
  const ws = new ReconnectingWebSocket(info.server, [], {
    WebSocket: WS,
  })
  ws.onopen = function () {
    ws.send(encode(JSON.stringify(authInfo), 7))
  }

  const heartBeatTask = setInterval(function () {
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
