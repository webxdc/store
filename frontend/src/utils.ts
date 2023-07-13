import type { AppInfo } from './bindings/AppInfo'
import type { WebxdcStatusUpdatePayload } from './bindings/WebxdcStatusUpdatePayload'

export function isAppInfo(p: any): p is AppInfo {
  return Object.prototype.hasOwnProperty.call(p, 'version')
}

export type WebxdcOutdatedResponse = Extract<WebxdcStatusUpdatePayload, { type: 'Outdated' }>
export type WebxdcUpdateSentResponse = Extract<WebxdcStatusUpdatePayload, { type: 'UpdateSent' }>

export function isOutdatedResponse(p: any): p is WebxdcOutdatedResponse {
  return p.type === 'Outdated'
}

export function isUpdateSendResponse(p: any): p is WebxdcUpdateSentResponse {
  return p.type === 'UpdateSent'
}
