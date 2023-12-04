import { WebSocket } from 'ws'
import pako = require('pako')
import brotli = require('brotli')
import JLogger from '../logger'

const log = JLogger.getInstance('biliws')

export type WsInfo = {
  server: string
  room_id: number
  uid: number
  token: string
}

export type PackResult = {
  packetLen: number
  headerLen: number
  ver: number
  op: number
  seq: number
  body: any[]
}

export enum MessageOP {
  KEEP_ALIVE = 2,
  KEEP_ALIVE_REPLY = 3,
  SEND_MSG = 4,
  SEND_MSG_REPLY = 5,
  AUTH = 7,
  AUTH_REPLY = 8,
}

export enum WsBodyVer {
  NORMAL,
  HEARTBEAT,
  DEFLATE,
  BROTLI,
}

export class BiliWsMessage {
  private _buffer: Uint8Array
  private _text_encoder: TextEncoder
  private _text_decoder: TextDecoder

  constructor(op?: MessageOP, str?: string) {
    this._text_encoder = new TextEncoder()
    this._text_decoder = new TextDecoder()
    if (!op) {
      this._buffer = new Uint8Array()
      return
    }
    const header = new Uint8Array([
      0,
      0,
      0,
      0,
      0,
      16,
      0,
      1,
      0,
      0,
      0,
      op,
      0,
      0,
      0,
      1,
    ])
    const data = this._text_encoder.encode(str)
    const packet_len = header.length + data.byteLength
    // Set data into buffer
    this._buffer = new Uint8Array(packet_len)
    this._buffer.set(header, 0)
    this._buffer.set(data, header.length)
    // Update packet_len in header
    this.writeInt(this._buffer, 0, 4, packet_len)
  }

  public SetBuffer(buffer: Uint8Array): BiliWsMessage {
    this._buffer = buffer
    return this
  }

  public GetBuffer(): Buffer {
    return Buffer.from(this._buffer)
  }

  // ToPack decodes buffer into PackResult
  public ToPack(): PackResult {
    const result: PackResult = {
      packetLen: 0,
      headerLen: 0,
      ver: 0,
      op: 0,
      seq: 0,
      body: [],
    }
    result.packetLen = this.readInt(this._buffer, 0, 4)
    result.headerLen = this.readInt(this._buffer, 4, 2)
    result.ver = this.readInt(this._buffer, 6, 2)
    result.op = this.readInt(this._buffer, 8, 4)
    result.seq = this.readInt(this._buffer, 12, 4)
    switch (result.op) {
      case MessageOP.AUTH_REPLY: {
        log.debug('Received auth reply')
        break
      }
      case MessageOP.KEEP_ALIVE_REPLY: {
        log.debug('Received keepalive reply')
        result.body = [
          {
            count: this.readInt(this._buffer, 16, 4),
          },
        ]
        break
      }
      case MessageOP.SEND_MSG_REPLY: {
        log.debug('Received msg', {
          length: result.packetLen - result.headerLen,
          ver: result.ver,
        })
        result.body = []
        switch (result.ver) {
          case WsBodyVer.NORMAL: {
            const data = this._buffer.slice(result.headerLen, result.packetLen)
            const body = this._text_decoder.decode(data)
            result.body.push(JSON.parse(body))
            break
          }
          case WsBodyVer.DEFLATE: {
            const next_buffer = pako.inflate(
              this._buffer.slice(result.headerLen, result.packetLen)
            )
            result.body = this.parseDecompressed(next_buffer)
            break
          }
          case WsBodyVer.BROTLI: {
            const body_buffer = Buffer.from(
              this._buffer.slice(result.headerLen, result.packetLen)
            )
            const decompressed_body = brotli.decompress(
              body_buffer,
              result.packetLen
            )
            result.body = this.parseDecompressed(decompressed_body)
            break
          }
          default: {
            log.error('Unknown message body ver', { ver: result.ver })
          }
        }
        break
      }
      default: {
        log.error('Message op known', { op: result.op })
      }
    }
    return result
  }

  private parseDecompressed(buffer: Uint8Array): any[] {
    let bodys = []
    let offset = 0
    while (offset < buffer.length) {
      const packetLen = this.readInt(buffer, offset, 4)
      const headerLen = 16 // readInt(buffer,offset + 4,4)
      const data = buffer.slice(offset + headerLen, offset + packetLen)
      /**
       *    引入pako做message解压处理，具体代码链接如下
       *    https://github.com/nodeca/pako/blob/master/dist/pako.js
       */
      const body = this._text_decoder.decode(data)
      if (body) {
        bodys.push(JSON.parse(body))
      }
      offset += packetLen
    }
    return bodys
  }

  private readInt(buffer: Uint8Array, start: number, len: number): number {
    let result = 0
    for (let i = len - 1; i >= 0; i--) {
      result += Math.pow(256, len - i - 1) * buffer[start + i]
    }
    return result
  }

  private writeInt(
    buffer: Uint8Array,
    start: number,
    len: number,
    value: number
  ) {
    let i = 0
    while (i < len) {
      buffer[start + i] = value / Math.pow(256, len - i - 1)
      i++
    }
  }
}

export class BiliWebSocket {
  private readonly _ws_info: WsInfo
  private _ws: WebSocket
  private _heartbeat_task: any
  private _is_manual_close: boolean = false
  private _try_reconnect_count: number = 0

  public msg_handler: Function

  constructor(ws_info: WsInfo) {
    this._ws_info = ws_info
  }

  public Connect(reconnect?: boolean) {
    log.info('Connecting to room websocket', this._ws_info)
    // Clean up old connection
    this.Disconnect()

    // Setup new connection
    this._ws = new WebSocket(this._ws_info.server)
    this._ws.on('open', () => {
      this._try_reconnect_count = 0
      // Prepare auth info
      const auth_info = {
        uid: Number(this._ws_info.uid),
        roomid: Number(this._ws_info.room_id),
        protover: 3,
        type: 2,
        platform: 'web',
        key: this._ws_info.token,
      }
      const auth_msg = new BiliWsMessage(
        MessageOP.AUTH,
        JSON.stringify(auth_info)
      )
      this._ws.send(auth_msg.GetBuffer())

      // Setup task for heart beating
      const heart_msg = new BiliWsMessage(MessageOP.KEEP_ALIVE, '')
      this._ws.send(heart_msg.GetBuffer())
      this._heartbeat_task = setInterval(() => {
        this._ws.send(heart_msg.GetBuffer())
      }, 10 * 1000)
    })

    this._ws.on('message', (data: Buffer) => {
      const msg = new BiliWsMessage().SetBuffer(data)
      if (this.msg_handler) {
        this.msg_handler(msg.ToPack())
      }
    })

    this._ws.on('close', () => {
      log.info('Websocket closed', this._ws_info)
      if (this._heartbeat_task) {
        clearInterval(this._heartbeat_task)
        this._heartbeat_task = null
      }
      if (!this._is_manual_close && reconnect) {
        if (this._try_reconnect_count < 5) {
          this._try_reconnect_count++
        }
        setTimeout(() => {
          log.info('Reconnecting to room websocket', this._ws_info)
          this.Connect(true)
        }, 1000 * this._try_reconnect_count)
      }
    })
  }

  public Disconnect() {
    this._is_manual_close = true
    if (this._ws) {
      this._ws.close()
      this._ws = null
    }
  }
}
