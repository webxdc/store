// This file was generated by [ts-rs](https://github.com/Aleph-Alpha/ts-rs). Do not edit this file manually.
import type { AppInfo } from "./AppInfo";

export type WebxdcStatusUpdatePayload = { type: "UpdateWebxdc" } | { type: "Outdated", critical: boolean, tag_name: string, } | { type: "UpdateSent" } | { type: "UpdateRequest", serial: number, apps: Array<[string, string]>, } | { type: "Download", app_id: string, } | { type: "DownloadOkay", app_id: string, name: string, data: string, } | { type: "DownloadError", app_id: string, error: string, } | { type: "Update", app_infos: Array<AppInfo>, serial: number, updating: Array<string>, };
