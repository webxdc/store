import { describe, expect, test, vi } from 'vitest'
import { type Setter } from 'solid-js'
import type { SetStoreFunction } from 'solid-js/store'
import { createStore } from 'solid-js/store'
import { AppInfoDB } from '../src/db/store_db'
import { AppState } from '../src/types'
import type { AppInfoWithState, AppInfosById } from '../src/types'
import type { DownloadResponseError, DownloadResponseOkay, UpdateResponse } from '../src/store-logic'
import { StoreState, to_app_infos_by_id, updateHandler } from '../src/store-logic'
import type { WebxdcOutdatedResponse, WebxdcUpdateSentResponse } from '../src/utils'
import 'fake-indexeddb/auto'
import mock from '../src/mock'

const general_handlers = {
  setAppInfo: ((() => { }) as SetStoreFunction<AppInfosById>),
  setlastUpdateSerial: ((() => { }) as Setter<number>),
  setIsUpdating: ((() => { }) as Setter<boolean>),
  setlastUpdate: ((() => { }) as Setter<Date>),
  setStoreState: ((() => { }) as Setter<StoreState>),
}

global.fetch = vi.fn()

const mock_manifest = `
[webxdc-2048]
app_id = "webxdc-2048"
version = 1
tag_name = "v1.2.1"
url = "https://github.com/webxdc/2048/releases/download/v1.2.1/2048.xdc"
date = "2023-07-11T16:33:26Z"
cache_relname = "2048.xdc"
description = "Join numbers to a 2048 tile\nThe classic 2048 puzzle game.\nMove tiles with the same number together and get a 2048 tile to win.\nHighscores are shared with the group."
source_code_url = "https://github.com/webxdc/2048"
name = "2048"
`

describe('Store receiving updates', () => {
  test('Handles outdated response', () => {
    const handlers = {
      db: new AppInfoDB('storetesting'),
      appInfo: {},
      ...general_handlers,
    }

    const payload = {
      type: 'Outdated',
      critical: true,
      version: 1,
    } as WebxdcOutdatedResponse

    const storeState = vi.spyOn(handlers, 'setStoreState')
    updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setStoreState)
    expect(storeState).toHaveBeenCalledWith(StoreState.Outdated)
  })

  test('Handles update received', async () => {
    const handlers = {
      db: new AppInfoDB('storetesting'),
      appInfo: {},
      ...general_handlers,
    }

    // @ts-expect-error - fetch is mocked
    fetch.mockReturnValue({ text: () => new Promise(resolve => resolve(mock_manifest)) })

    // test newer version received than the current instance runs
    let payload = {
      type: 'UpdateSent',
      version: 2,
    } as WebxdcUpdateSentResponse

    const storeState = vi.spyOn(handlers, 'setStoreState')
    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setStoreState)
    expect(storeState).toHaveBeenCalledWith(StoreState.WaitingForRestart)

    // test same version received as the current instance runs
    payload = {
      type: 'UpdateSent',
      version: 1,
    } as WebxdcUpdateSentResponse

    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setStoreState)
    expect(storeState).toHaveBeenCalledWith(StoreState.UpToDate)
  })

  test('Handles download error', () => {
    const handlers = {
      db: new AppInfoDB('storetesting'),
      appInfo: {},
      ...general_handlers,
    }

    const payload = {
      type: 'DownloadError',
      error: 'error',
      app_id: 'test',
    } as DownloadResponseError

    const setAppInfo = vi.spyOn(handlers, 'setAppInfo')
    updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setStoreState)
    expect(setAppInfo).toHaveBeenCalledWith(payload.app_id, 'state', AppState.DownloadCancelled)
  })

  test('Handles download okay', async () => {
    const db = new AppInfoDB('storetesting1')
    const handlers = {
      ...general_handlers,
      db,
      appInfo: to_app_infos_by_id(mock),
    }

    await db.insertMultiple(mock)

    // Test adding new webxdc downloads
    let payload: DownloadResponseOkay = {
      type: 'DownloadOkay',
      app_id: mock[0].app_id,
      name: 'test',
      data: 'test',
    }

    const setAppInfo = vi.spyOn(handlers, 'setAppInfo')
    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setStoreState)
    expect(await db.get_webxdc(payload.app_id)).matchSnapshot()
    expect(await db.get(payload.app_id)).toStrictEqual({ ...mock[0], state: AppState.Received })
    expect(setAppInfo).toHaveBeenCalledWith(payload.app_id, 'state', AppState.Received)

    // Test updating an existing webxdc
    await db.add_webxdc({ name: 'aa', plainText: 'bee' }, mock[4].app_id)

    payload = {
      type: 'DownloadOkay',
      app_id: mock[4].app_id,
      name: 'test',
      data: 'test',
    }

    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setStoreState)
    expect(setAppInfo).toHaveBeenCalledWith(payload.app_id, 'state', AppState.Received)
    expect(await db.get(payload.app_id)).toStrictEqual({ ...mock[4], state: AppState.Received })
    expect(await db.get_webxdc(payload.app_id)).toMatchSnapshot()
  })

  test('Handles new AppIndex', async () => {
    const db = new AppInfoDB('storetesting2')
    const handlers = {
      db,
      ...general_handlers,
      appInfo: {},
    }

    const payload = {
      type: 'Update',
      app_infos: mock,
      serial: 12,
      updating: [],
    } as UpdateResponse

    const setAppInfo = vi.spyOn(handlers, 'setAppInfo')
    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setStoreState)

    const initial_mock = mock.map(app_info => ({ ...app_info, state: AppState.Initial } as AppInfoWithState))
    expect(setAppInfo).toHaveBeenCalledWith(to_app_infos_by_id(initial_mock))
    expect(await db.get_all()).toStrictEqual(Object.values(initial_mock))
  })

  test('Handles ongoing AppIndex updates', async () => {
    const db = new AppInfoDB('storetesting3')
    const advanced_state = to_app_infos_by_id(mock.slice(0, 2))
    await db.insertMultiple(Object.values(advanced_state))
    expect(await db.get_all()).toStrictEqual(Object.values(advanced_state))

    const [appInfo, setAppInfo] = createStore(advanced_state)

    const handlers = {
      ...general_handlers,
      db,
      appInfo,
      setAppInfo,
    }

    const crazy_update = mock.slice()
    crazy_update[0].version = 2
    let payload = {
      type: 'Update',
      app_infos: mock,
      serial: 12,
      updating: [],
    } as UpdateResponse

    const updateMultiple = vi.spyOn(db, 'updateMultiple')
    const insertMultiple = vi.spyOn(db, 'insertMultiple')
    const satlastUpdateSerial = vi.spyOn(handlers, 'setlastUpdateSerial')
    const setIsUpdating = vi.spyOn(handlers, 'setIsUpdating')
    const setlastUpdate = vi.spyOn(handlers, 'setlastUpdate')

    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setStoreState)

    // Tests:
    // - Appinfo with state !== Initial are then in 'Updating'
    // - Appinfo with state === Initial are untouched
    expect(advanced_state).toMatchSnapshot()
    expect(insertMultiple).toHaveBeenCalledWith(mock.slice(2, undefined).map(app_info => ({ ...app_info, state: AppState.Initial } as AppInfoWithState)))
    expect(updateMultiple).toHaveBeenCalledWith(mock.slice(0, 2).map(app_info => ({ ...app_info } as AppInfoWithState)))
    expect(await db.get_all()).toMatchSnapshot()
    expect(satlastUpdateSerial).toHaveBeenCalledWith(12)
    expect(setIsUpdating).toHaveBeenCalledWith(false)
    expect(setlastUpdate).toHaveBeenCalled()

    payload = {
      type: 'Update',
      app_infos: mock,
      serial: 12,
      updating: ['15'],
    } as UpdateResponse

    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setStoreState)
    expect(appInfo['15'].state).toBe(AppState.Updating)

    const download: DownloadResponseOkay = {
      type: 'DownloadOkay',
      app_id: mock[3].app_id,
      name: 'test',
      data: 'test',
    }

    await updateHandler(download, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setStoreState)
    expect(appInfo['15'].state).toBe(AppState.Received)
  })
})
