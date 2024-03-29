import { Low } from 'lowdb'
import { JSONFile } from 'lowdb/node'
import JLogger from './logger'
import { app } from 'electron'
import * as types from './types'
import { GiftMessage, GuardMessage, SuperChatMessage } from './messages'
import { SuperChat } from './common/superchatInterface'

const log = JLogger.getInstance('gift_store')
const db_path = app.getPath('userData') + '/gift_store_v2.json'

type GiftDBData = {
  gift: GiftMessage[]
  guard: GuardMessage[]
  superchat: SuperChatMessage[]
}

export class GiftStore {
  private _db: Low<GiftDBData>

  constructor() {
    const default_db: GiftDBData = {
      gift: [],
      guard: [],
      superchat: [],
    }
    // Create database in user app data directory
    this._db = new Low<GiftDBData>(new JSONFile(db_path), default_db)

    this.init()
  }

  public async Get(type: string, room: number) {
    switch (type) {
      case 'gift':
        return this._db.data.gift.filter((item) => item.room == room)
      case 'guard':
        return this._db.data.guard.filter((item) => item.room == room)
      case 'superchat':
        return this._db.data.superchat.filter((item) => item.room == room)
      default:
        log.error('Unknown type to get', { type })
        return []
    }
  }

  public async Push(item: GiftMessage | GuardMessage | SuperChatMessage) {
    if (item instanceof GiftMessage) {
      this._db.data.gift.push(item)
    }
    if (item instanceof GuardMessage) {
      this._db.data.guard.push(item)
    }
    if (item instanceof SuperChatMessage) {
      this._db.data.superchat.push(item)
    }
    await this._db.write()
  }

  public async Delete(type: string, id: string) {
    switch (type) {
      case 'gift':
        this._db.data.gift = this._db.data.gift.filter((item) => item.id != id)
        break
      case 'guard':
        this._db.data.guard = this._db.data.guard.filter(
          (item) => item.id != id
        )
        break
      case 'superchat':
        this._db.data.superchat = this._db.data.superchat.filter(
          (item) => item.id != id
        )
        break
      default:
        log.error('Unknown type to delete', { type })
        break
    }
    await this._db.write()
  }

  public async Clear(type: string, room: number) {
    switch (type) {
      case 'gift':
        this._db.data.gift = this._db.data.gift.filter(
          (item) => item.room != room
        )
        break
      case 'guard':
        this._db.data.guard = this._db.data.guard.filter(
          (item) => item.room != room
        )
        break
      case 'superchat':
        this._db.data.superchat = this._db.data.superchat.filter(
          (item) => item.room != room
        )
        break
      default:
        log.error('Unknown type to clear', { type })
        break
    }
    await this._db.write()
  }

  private async init() {
    await this._db.read()
  }
}
