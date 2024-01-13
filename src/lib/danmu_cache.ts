import { DanmuRecord, RecordType } from './types'

export class DanmuCache {
  private _max_entries: number
  private _cache: Map<number, DanmuRecord[]> = new Map<number, DanmuRecord[]>()

  constructor(max_entries: number = 100) {
    this._max_entries = max_entries
  }

  public get(uid: number): DanmuRecord[] {
    return this._cache.get(uid)
  }

  public add(type: RecordType, uid: number, content: string): void {
    if (!this._cache.has(uid)) {
      this._cache.set(uid, [])
    }
    const cache = this._cache.get(uid)
    cache.push({
      type: type,
      timestamp: Date.now(),
      content: content,
    })
    while (cache.length > this._max_entries) {
      cache.shift()
    }
  }

  public updateMaxEntries(max_entries: number): void {
    this._max_entries = max_entries
  }

  public remove(uid: number): void {
    this._cache.delete(uid)
  }

  public clear(): void {
    this._cache.clear()
  }
}
