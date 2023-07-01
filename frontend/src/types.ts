import type { AppInfo } from './bindings/AppInfo'

export enum AppState {
  Initial,
  Downloading,
  DownloadCancelled,
  Received,
  Updating,
}

export interface AppInfoWithState extends AppInfo {
  state: AppState
  cached: boolean
}

export type AppInfosById = Record<string, AppInfoWithState>
