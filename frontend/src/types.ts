import type { AppInfo } from './bindings/AppInfo'

export enum AppState {
  Initial,
  Downloading,
  DownloadCancelled,
}

export interface AppInfoWithState extends AppInfo {
  state: AppState
}

export type AppInfosById = Record<number, AppInfoWithState>
