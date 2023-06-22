import type { AppInfo } from './bindings/AppInfo'
import type { GeneralFrontendResponse } from './bindings/GeneralFrontendResponse'

export function isAppInfo(p: any): p is AppInfo {
  return Object.prototype.hasOwnProperty.call(p, 'version')
}

export type WebxdcOutdatedResponse = Extract<GeneralFrontendResponse, { type: 'Outdated' }>

export function isOutdatedResponse(p: any): p is WebxdcOutdatedResponse {
  return p.type === 'Outdated'
}
