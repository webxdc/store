import type { AppInfo } from '../bindings/AppInfo'
import type { AppState, AppInfoWithState } from '../types'
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
        db.createObjectStore('appInfo', { keyPath: 'app_id' })
        db.createObjectStore('apps')
      }
    })
  }

  async insertMultiple(data: AppInfoWithState[]): Promise<void> {
    const db = await this.open()
    return new Promise((resolve, reject) => {
      const transaction = db.transaction('appInfo', 'readwrite')
      transaction.onerror = () => reject(transaction.error)
      const store = transaction.objectStore('appInfo')
      data.forEach((item: AppInfoWithState | AppInfo) => store.add(item))
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

  async get(id: string): Promise<AppInfoWithState | undefined> {
    const db = await this.open()
    return new Promise((resolve, reject) => {
      const transaction = db.transaction('appInfo', 'readonly')
      transaction.onerror = () => reject(transaction.error)
      const store = transaction.objectStore('appInfo')
      const request = store.get(id)
      request.onsuccess = () => resolve(request.result)
    })
  }

  // Set the state of the app.
  async updateState(app_id: string, state: AppState): Promise<void> {
    const db = await this.open()
    return new Promise((resolve, reject) => {
      const transaction = db.transaction('appInfo', 'readwrite')
      transaction.onerror = () => reject(transaction.error)
      const store = transaction.objectStore('appInfo')
      const request = store.get(app_id)
      request.onsuccess = () => {
          const appInfo = request.result
          appInfo.state = state
          const putRequest = store.put(appInfo)
          putRequest.onsuccess = () => resolve()
      }
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

  // Add base64 encoded webxdc to the db.
  async add_webxdc(webxdc: XDCFile, id: string): Promise<void> {
    const db = await this.open()
    return new Promise((resolve, reject) => {
      const transaction = db.transaction('apps', 'readwrite')
      transaction.onerror = () => reject(transaction.error)
      const store = transaction.objectStore('apps')
      const request = store.put(webxdc, id)
      request.onsuccess = () => resolve()
    })
  }

  // Get base64 encoded webxdc from the db.
  async get_webxdc(id: string): Promise<XDCFile> {
    const db = await this.open()
    return new Promise((resolve, reject) => {
      const transaction = db.transaction('apps', 'readonly')
      transaction.onerror = () => reject(transaction.error)
      const store = transaction.objectStore('apps')
      const request = store.get(id)
      request.onsuccess = () => resolve(request.result)
    })
  }

  async remove_webxdc(id: string): Promise<void> {
    const db = await this.open()
    return new Promise((resolve, reject) => {
      const transaction = db.transaction('apps', 'readwrite')
      transaction.onerror = () => reject(transaction.error)
      const store = transaction.objectStore('apps')
      const request = store.delete(id)
      request.onsuccess = () => resolve()
    })
  }
}
