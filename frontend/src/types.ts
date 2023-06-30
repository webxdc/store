import type { AppInfo } from './bindings/AppInfo'

export enum AppState {
  Initial,
  Downloading,
  DownloadCancelled,
  Received,
}

export interface AppInfoWithState extends AppInfo {
  state: AppState
}

export type AppInfosById = Record<string, AppInfoWithState>
