import { WebSocket } from 'ws'
import pako = require('pako')
import JLogger from '../logger'

const log = JLogger.getInstance()

export type WsInfo = {
  server: string
  roomid: number
  uid: number
  token: string
}

export type PackResult = {
  packetLen: number
  headerLen: number
  ver: number
  op: number
  seq: number
  body: any
}

export enum MessageOP {
  KEEP_ALIVE = 2,
  AUTH = 7
}

export class BiliWsMessage {
  private buffer: Uint8Array
  private text_encoder: TextEncoder
  private text_decoder: TextDecoder

  constructor(op?: MessageOP, str?: string) {
    this.text_encoder = new TextEncoder()
    this.text_decoder = new TextDecoder()
    if (!op || !str) {
      this.buffer = new Uint8Array()
      return
    }
    const header = new Uint8Array([0, 0, 0, 0, 0, 16, 0, 1, 0, 0, 0, op, 0, 0, 0, 1])
    const data = this.text_encoder.encode(str)
    const packet_len = header.length + data.byteLength
    // Set data into buffer
    this.buffer = new Uint8Array(packet_len)
    this.buffer.set(header, 0)
    this.buffer.set(data, header.length)
    // Update packet_len in header
    this.writeInt(this.buffer, 0, 4, packet_len)
  }

  public SetBuffer(buffer: Uint8Array): BiliWsMessage {
    this.buffer = buffer
    return this
  }

  public GetBuffer(): Buffer {
    return Buffer.from(this.buffer)
  }

  // ToPack decodes buffer into PackResult
  public ToPack(): PackResult {
    const result: PackResult = {
      packetLen: 0,
      headerLen: 0,
      ver: 0,
      op: 0,
      seq: 0,
      body: null,
    }
    result.packetLen = this.readInt(this.buffer, 0, 4)
    result.headerLen = this.readInt(this.buffer, 4, 2)
    result.ver = this.readInt(this.buffer, 6, 2)
    result.op = this.readInt(this.buffer, 8, 4)
    result.seq = this.readInt(this.buffer, 12, 4)
    if (result.op === 5) {
      result.body = []
      if (result.ver === 0) {
        const data = this.buffer.slice(result.headerLen, result.packetLen)
        const body = this.text_decoder.decode(data)
        result.body.push(JSON.parse(body))
      } else if (result.ver === 2) {
        const next_buffer = pako.inflate(
          this.buffer.slice(result.headerLen, result.packetLen)
        )
        let offset = 0
        while (offset < next_buffer.length) {
          const packetLen = this.readInt(next_buffer, offset + 0, 4)
          const headerLen = 16 // readInt(buffer,offset + 4,4)
          const data = next_buffer.slice(offset + headerLen, offset + packetLen)
          /**
           *    引入pako做message解压处理，具体代码链接如下
           *    https://github.com/nodeca/pako/blob/master/dist/pako.js
           */
          const body = this.text_decoder.decode(data)
          if (body) {
            result.body.push(JSON.parse(body))
          }
          offset += packetLen
        }
      }
    } else if (result.op === 3) {
      result.body = {
        count: this.readInt(this.buffer, 16, 4),
      }
    }
    return result
  }

  private readInt(buffer: Uint8Array, start: number, len: number): number {
    let result = 0
    for (let i = len - 1; i >= 0; i--) {
      result += Math.pow(256, len - i - 1) * buffer[start + i]
    }
    return result
  }

  private writeInt(buffer: Uint8Array, start: number, len: number, value: number) {
    let i = 0
    while (i < len) {
      buffer[start + i] = value / Math.pow(256, len - i - 1)
      i++
    }
  }
}

export class BiliWebSocket {
  private ws_info: WsInfo
  private ws: WebSocket
  private heartbeat_task: any
  public msg_handler: Function

  constructor(ws_info: WsInfo) {
    this.ws_info = ws_info
  }

  public Connect(reconnect?: boolean) {
    log.info('Connecting to room websocket', this.ws_info)
    // Maybe connect from auto-reconnect
    this.Disconnect()

    // Setup new connection
    this.ws = new WebSocket(this.ws_info.server)
    this.ws.on('open', () => {
      // Prepare auth info
      const auth_info = {
        uid: Number(this.ws_info.uid),
        roomid: Number(this.ws_info.roomid),
        protover: 2,
        type: 2,
        platform: 'web',
        key: this.ws_info.token,
      }
      const auth_msg = new BiliWsMessage(MessageOP.AUTH, JSON.stringify(auth_info))
      this.ws.send(auth_msg.GetBuffer())

      // Setup task for heart beating
      const heart_msg = new BiliWsMessage(MessageOP.KEEP_ALIVE, '')
      this.ws.send(heart_msg.GetBuffer())
      this.heartbeat_task = setInterval(() => {
        this.ws.send(heart_msg.GetBuffer())
      }, 30000)
    })

    this.ws.on('message', (data: Buffer) => {
      const msg = new BiliWsMessage().SetBuffer(data)
      if (this.msg_handler) {
        this.msg_handler(msg.ToPack())
      }
    })

    this.ws.on('close', () => {
      log.info('Websocket closed', this.ws_info)
    })
  }

  public Disconnect() {
    if (this.heartbeat_task) {
      clearInterval(this.heartbeat_task)
      this.heartbeat_task = null
    }
    if (this.ws) {
      this.ws.close()
      this.ws = null
    }
  }
}


