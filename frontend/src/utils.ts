import type { AppInfo } from './bindings/AppInfo'
import type { WebxdcOutdatedResponse } from './bindings/WebxdcOutdatedResponse'

export function isAppInfo(p: any): p is AppInfo {
  return Object.prototype.hasOwnProperty.call(p, 'version')
}

export function isOutdatedResponse(p: any): p is WebxdcOutdatedResponse {
  return Object.prototype.hasOwnProperty.call(p, 'critical')
}
