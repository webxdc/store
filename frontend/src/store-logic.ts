import type { Accessor, Setter } from 'solid-js'
import type { SetStoreFunction } from 'solid-js/store'
import { produce } from 'solid-js/store'
import type { AppInfoWithState, AppInfosById } from './types'
import { AppState } from './types'
import type { WebxdcStatusUpdatePayload } from './bindings/WebxdcStatusUpdatePayload'
import type { AppInfoDB } from './db/store_db'
import { isOutdatedResponse, isUpdateSendResponse as isUpdateSentResponse } from './utils'
import type { AppInfo } from './bindings/AppInfo'

export type DownloadResponseOkay = Extract<WebxdcStatusUpdatePayload, { type: 'DownloadOkay' }>
export type DownloadResponseError = Extract<WebxdcStatusUpdatePayload, { type: 'DownloadError' }>
export type UpdateResponse = Extract<WebxdcStatusUpdatePayload, { type: 'Update' }>

function isDownloadResponseOkay(p: any): p is DownloadResponseOkay {
  return p.type === 'DownloadOkay'
}

function isDownloadResponseError(p: any): p is DownloadResponseError {
  return p.type === 'DownloadError'
}

function isUpdateResponse(p: any): p is UpdateResponse {
  return p.type === 'Update'
}

function isComplete(elements: Record<string, Partial<AppInfo> | null>): elements is Record<string, AppInfo> {
  for (const element of Object.values(elements)) {
    if (element == null) {
      continue
    }
    if (!(!!element.app_id && !!element.date && !!element.description && !!element.name && !!element.size && !!element.source_code_url && !!element.tag_name)) {
      return false
    }
  }
  return true
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
  if (isUpdateResponse(payload)) {
    if (payload.old_serial === 0) {
      console.log('Initialising apps')
      const app_infos = payload.app_infos
      if (isComplete(app_infos)) {
        const app_infos_states: AppInfosById = Object.keys(app_infos).reduce((res, key) => {
          res[key] = { state: AppState.Initial, ...app_infos[key] }
          return res
        }, {} as AppInfosById)
        setAppInfo(app_infos_states)
        await db.insertMultiple(Object.values(app_infos_states))
        setIsUpdating(false)
      }
      else {
        console.error('Received incomplete initialization message')
      }
    }
    else if (payload.old_serial === lastSerial()) {
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
          if (s[key] === undefined) {
            // As we don't know that app we can  assert the partial updates to be complete here
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
      await db.updateMultiple(updated.map(key => ({ ...appInfo[key], ...app_infos[key], state: appInfo[key].state !== AppState.Initial ? AppState.Updating : AppState.Initial })))
      setIsUpdating(false)
    }
    else {
      console.log(`Ignoring outdated update with reference serial: ${payload.old_serial} and currente serial ${lastSerial()}`)
      setIsUpdating(false)
      return
    }

    setlastUpdateSerial(payload.serial)
    setIsUpdating(false)
    setlastUpdate(new Date())
  }
  else if (isDownloadResponseOkay(payload)) {
    console.log('Received webxdc')
    const file = { base64: payload.data, name: `${payload.name}.xdc` }
    await db.add_webxdc(file, payload.app_id)
    await db.update({ ...appInfo[payload.app_id], state: AppState.Received })
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
