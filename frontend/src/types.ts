import { FrontendAppInfo } from "./bindings/FrontendAppInfo"


export enum AppState {
    Initial,
    Downloading,
    Received
}

export interface AppInfoWithState extends FrontendAppInfo {
    state: AppState
}

export type AppInfosById = Record<string, AppInfoWithState>
