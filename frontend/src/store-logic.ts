import type { Accessor, Setter } from 'solid-js'
import type { SetStoreFunction } from 'solid-js/store'
import { produce } from 'solid-js/store'
import type { AppInfoWithState, AppInfosById } from './types'
import { AppState } from './types'
import type { WebxdcStatusUpdatePayload } from './bindings/WebxdcStatusUpdatePayload'
import type { AppInfoDB } from './db/store_db'
import { isOutdatedResponse, isUpdateSendResponse as isUpdateSentResponse } from './utils'

export type DownloadResponseOkay = Extract<WebxdcStatusUpdatePayload, { type: 'DownloadOkay' }>
export type DownloadResponseError = Extract<WebxdcStatusUpdatePayload, { type: 'DownloadError' }>
export type UpdateResponse = Extract<WebxdcStatusUpdatePayload, { type: 'Update' }>
export type InitResponse = Extract<WebxdcStatusUpdatePayload, { type: 'Init' }>

function isDownloadResponseOkay(p: any): p is DownloadResponseOkay {
  return p.type === 'DownloadOkay'
}

function isDownloadResponseError(p: any): p is DownloadResponseError {
  return p.type === 'DownloadError'
}

function isUpdateResponse(p: any): p is UpdateResponse {
  return p.type === 'Update'
}

function isInit(p: any): p is InitResponse {
  return p.type === 'Init'
}


export function cmpApps(a: AppInfoWithState, b: AppInfoWithState): number {
    if (a.state !== b.state) {
        if (a.state === AppState.Received) {
            return -1
        } else if (b.state === AppState.Received) {
            return 1
        }
    }
    return Number(b.date - a.date)
}

export function to_app_infos_by_id<T extends { app_id: string }>(app_infos: T[]): Record<string, T> {
  return app_infos.reduce((acc, appinfo) => {
    acc[appinfo.app_id] = appinfo
    return acc
  }, {} as Record<string, T>)
}

export async function updateHandler(
  payload: object,
  db: AppInfoDB,
  appInfo: AppInfosById,
  lastSerial: Accessor<number>,
  setAppInfo: SetStoreFunction<AppInfosById>,
  setlastUpdateSerial: Setter<number>,
  setIsUpdating: Setter<boolean>,
  setlastUpdate: Setter<Date>,
  setUpdateNeeded: Setter<boolean>,
  setUpdateReceived: Setter<boolean>,
) {
  if (isInit(payload)) {
    console.log('Initialising apps')
    const app_infos = to_app_infos_by_id((payload.app_infos).map(app_info => ({ ...app_info, state: AppState.Initial })))
    setAppInfo(app_infos)
    await db.insertMultiple(Object.values(app_infos))
    setIsUpdating(false)
    setlastUpdateSerial(payload.serial)
  }
  else if (isUpdateResponse(payload)) {
    if (lastSerial() === payload.old_serial) {
      console.log('Reconceiling updates')
      const app_infos = payload.app_infos
      const added: string[] = []
      const updated: string[] = []
      const removed: string[] = []
      setAppInfo(produce((s) => {
        for (const key in app_infos) {
          if (app_infos[key] === null) {
            delete s[key]
            removed.push(key)
          }
          else if (s[key] === undefined) {
            // As we don't know that app_id, we can assert the partial updates to be complete here.
            s[key] = { ...(app_infos[key] as AppInfoWithState) }
            added.push(key)
          }
          else {
            s[key] = Object.assign(s[key], { ...app_infos[key] })
            updated.push(key)
          }
        }
        for (const key of payload.updating) {
          s[key] = Object.assign(s[key], { state: AppState.Updating })
        }
      }))

      await db.insertMultiple(added.map(key => ({ ...app_infos[key], state: AppState.Initial })) as AppInfoWithState[])
      await db.updateMultiple(updated.map(key => ({ ...appInfo[key], ...app_infos[key] })))
      await db.remove_multiple_app_infos(removed)
      removed.forEach(key => db.remove_webxdc(key))
      setlastUpdateSerial(payload.serial)
      setIsUpdating(false)
      setlastUpdate(new Date())
    }
    else {
      console.log('Update serial mismatch')
      setIsUpdating(false)
    }
  }
  else if (isDownloadResponseOkay(payload)) {
    console.log('Received webxdc')
    const file = { base64: payload.data, name: `${payload.name}.xdc` }
    await db.add_webxdc(file, payload.app_id)
    await db.updateState(payload.app_id, AppState.Received)
    setAppInfo(payload.app_id, 'state', AppState.Received)
  }
  else if (isDownloadResponseError(payload)) {
    console.log('Problem downloading some webxdc')
    setAppInfo(payload.app_id, 'state', AppState.DownloadCancelled)
  }
  else if (isOutdatedResponse(payload)) {
    console.log('Current tag_name is outdated')
    setUpdateNeeded(true)
  }
  else if (isUpdateSentResponse(payload)) {
    console.log('Update received')
    setUpdateReceived(true)
  }
}
