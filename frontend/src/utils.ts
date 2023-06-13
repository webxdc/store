import type { AppInfo } from './bindings/AppInfo'

export function isAppInfo(p: any): p is AppInfo {
  return Object.hasOwn(p, 'version')
}
