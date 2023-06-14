import type { AppInfo } from './bindings/AppInfo'

export function isAppInfo(p: any): p is AppInfo {
  return Object.prototype.hasOwnProperty.call(p, 'version')
}
