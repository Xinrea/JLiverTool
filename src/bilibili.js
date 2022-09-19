const WS = require('ws')
const ReconnectingWebSocket = require('reconnecting-websocket')
const pako = require('pako')
const http = require('https')

const textEncoder = new TextEncoder('utf-8')
const textDecoder = new TextDecoder('utf-8')

const readInt = function (buffer, start, len) {
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
  return new Uint8Array(header.concat(...data)).buffer
}
const decode = function (blob) {
  return new Promise(function (resolve, reject) {
    let buffer = new Uint8Array(blob)
    let result = {}
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

function connecting(room, msgHandler) {
  const ws = new ReconnectingWebSocket(
    'wss://broadcastlv.chat.bilibili.com:2245/sub',
    [],
    {
      WebSocket: WS
    }
  )
  ws.onopen = function () {
    ws.send(
      encode(
        JSON.stringify({
          roomid: Number(room)
        }),
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

function checkLiveStatus(room) {
  return new Promise(function (resolve, reject) {
    http
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

function getRoomInfo(room) {
  return new Promise(function (resolve, reject) {
    http
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

function getGiftList(room) {
  return new Promise(function (resolve, reject) {
    http
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

function getOnlineNum(uid, room) {
  return new Promise(function (resolve, reject) {
    http
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

function getUserInfo(uid) {
  return new Promise(function (resolve, reject) {
    http.request()
    http
      .get(
        {
          hostname: 'api.bilibili.com',
          path: `/x/space/acc/info?mid=${uid}&token=&platform=web&jsonp=jsonp`,
          method: 'GET',
          headers: {
            'User-Agent':
              'Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/80.0.3987.132 Safari/537.36'
          }
        },
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

module.exports = {
  connecting,
  checkLiveStatus,
  getRoomInfo,
  getOnlineNum,
  getGiftList,
  getUserInfo
}
