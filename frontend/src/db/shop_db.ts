import type { AppInfo } from '../bindings/AppInfo'
import type { AppInfoWithState } from '../types'
import type { XDCFile } from '../webxdc'

export class AppInfoDB {
  private dbName: string
  private db: IDBDatabase | undefined

  constructor(dbName: string) {
    this.dbName = dbName
  }

  open(): Promise<IDBDatabase> {
    if (this.db)
      return Promise.resolve(this.db)
    return new Promise((resolve, reject) => {
      const request = indexedDB.open(this.dbName, 3)
      request.onerror = () => reject(request.error)
      request.onsuccess = () => resolve((this.db = request.result))
      request.onupgradeneeded = () => {
        const db = request.result
        db.createObjectStore('appInfo', { keyPath: 'id' })
        db.createObjectStore('apps')
      }
    })
  }

  async insert(data: AppInfoWithState): Promise<void> {
    const db = await this.open()
    return new Promise((resolve, reject) => {
      const transaction = db.transaction('appInfo', 'readwrite')
      transaction.onerror = () => reject(transaction.error)
      const store = transaction.objectStore('appInfo')
      const request = store.add(data)
      request.onsuccess = () => resolve()
    })
  }

  async update(data: AppInfoWithState): Promise<void> {
    const db = await this.open()
    return new Promise((resolve, reject) => {
      const transaction = db.transaction('appInfo', 'readwrite')
      transaction.onerror = () => reject(transaction.error)
      const store = transaction.objectStore('appInfo')
      const request = store.put(data)
      request.onsuccess = () => resolve()
    })
  }

  async insertMultiple(data: AppInfoWithState[] | AppInfo[]): Promise<void> {
    const db = await this.open()
    return new Promise((resolve, reject) => {
      const transaction = db.transaction('appInfo', 'readwrite')
      transaction.onerror = () => reject(transaction.error)
      const store = transaction.objectStore('appInfo')
      data.forEach((item: AppInfoWithState | AppInfo) => store.add(item))
      transaction.oncomplete = () => resolve()
    })
  }

  async updateMultiple(data: AppInfoWithState[] | AppInfo[]): Promise<void> {
    const db = await this.open()
    return new Promise((resolve, reject) => {
      const transaction = db.transaction('appInfo', 'readwrite')
      transaction.onerror = () => reject(transaction.error)
      const store = transaction.objectStore('appInfo')
      data.forEach((item: AppInfoWithState | AppInfo) => store.put(item))
      transaction.oncomplete = () => resolve()
    })
  }

  async remove_multiple_app_infos(ids: number[]): Promise<void> {
    const db = await this.open()
    return new Promise((resolve, reject) => {
      const transaction = db.transaction('appInfo', 'readwrite')
      transaction.onerror = () => reject(transaction.error)
      const store = transaction.objectStore('appInfo')
      ids.forEach((id: number) => store.delete(id))
      transaction.oncomplete = () => resolve()
    })
  }

  async get_all(): Promise<AppInfoWithState[]> {
    const db = await this.open()
    return new Promise((resolve, reject) => {
      const transaction = db.transaction('appInfo', 'readonly')
      transaction.onerror = () => reject(transaction.error)
      const store = transaction.objectStore('appInfo')
      const request = store.openCursor()
      const result: AppInfoWithState[] = []
      request.onsuccess = () => {
        const cursor = request.result
        if (cursor) {
          result.push(cursor.value)
          cursor.continue()
        }
        else {
          resolve(result)
        }
      }
    })
  }

  // Add base64 encoded webxdc to the db.
  async add_webxdc(webxdc: XDCFile, id: number): Promise<void> {
    const db = await this.open()
    return new Promise((resolve, reject) => {
      const transaction = db.transaction('apps', 'readwrite')
      transaction.onerror = () => reject(transaction.error)
      const store = transaction.objectStore('apps')
      const request = store.add(webxdc, id)
      request.onsuccess = () => resolve()
    })
  }

  // Get base64 encoded webxdc from the db.
  async get_webxdc(id: number): Promise<XDCFile> {
    const db = await this.open()
    return new Promise((resolve, reject) => {
      const transaction = db.transaction('apps', 'readonly')
      transaction.onerror = () => reject(transaction.error)
      const store = transaction.objectStore('apps')
      const request = store.get(id)
      request.onsuccess = () => resolve(request.result)
    })
  }
}
