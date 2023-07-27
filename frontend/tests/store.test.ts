import { describe, expect, test, vi } from 'vitest'
import { type Setter } from 'solid-js'
import type { SetStoreFunction } from 'solid-js/store'
import { createStore } from 'solid-js/store'
import { AppInfoDB } from '../src/db/store_db'
import { AppState } from '../src/types'
import type { AppInfoWithState, AppInfosById } from '../src/types'
import type { DownloadResponseError, DownloadResponseOkay, UpdateResponse } from '../src/store-logic'
import { to_app_infos_by_id, updateHandler } from '../src/store-logic'
import type { WebxdcOutdatedResponse, WebxdcUpdateSentResponse } from '../src/utils'
import 'fake-indexeddb/auto'
import mock from '../src/mock'

const general_handlers = {
  getLastSerial: (() => 0) as (() => number),
  setAppInfo: ((() => { }) as SetStoreFunction<AppInfosById>),
  setlastUpdateSerial: ((() => { }) as Setter<number>),
  setIsUpdating: ((() => { }) as Setter<boolean>),
  setlastUpdate: ((() => { }) as Setter<Date>),
  setUpdateNeeded: ((() => { }) as Setter<boolean>),
  setUpdateReceived: ((() => { }) as Setter<boolean>),
}

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
      tag_name: 'v1',
    } as WebxdcOutdatedResponse

    const updateNeeded = vi.spyOn(handlers, 'setUpdateNeeded')
    updateHandler(payload, handlers.db, handlers.appInfo, handlers.getLastSerial, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)

    expect(updateNeeded).toHaveBeenCalledWith(true)
  })

  test('Handes update received', () => {
    const handlers = {
      db: new AppInfoDB('storetesting'),
      appInfo: {},
      ...general_handlers,
    }

    const payload = {
      type: 'UpdateSent',
    } as WebxdcUpdateSentResponse

    const setUpdateReceived = vi.spyOn(handlers, 'setUpdateReceived')
    updateHandler(payload, handlers.db, handlers.appInfo, handlers.getLastSerial, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
    expect(setUpdateReceived).toHaveBeenCalledWith(true)
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
    updateHandler(payload, handlers.db, handlers.appInfo, handlers.getLastSerial, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
    expect(setAppInfo).toHaveBeenCalledWith(payload.app_id, 'state', AppState.DownloadCancelled)
  })

  test('Handles download okay', async () => {
    const db = new AppInfoDB('storetesting1')
    const handlers = {
      ...general_handlers,
      db,
      appInfo: mock,
    }

    await db.insertMultiple(Object.values(mock))

    // Test adding new webxdc downloads
    let payload: DownloadResponseOkay = {
      type: 'DownloadOkay',
      app_id: mock[0].app_id,
      name: 'test',
      data: 'test',
    }

    const setAppInfo = vi.spyOn(handlers, 'setAppInfo')
    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.getLastSerial, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
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

    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.getLastSerial, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
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
      old_serial: 0,
      updating: [],
    } as UpdateResponse

    const setAppInfo = vi.spyOn(handlers, 'setAppInfo')
    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.getLastSerial, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)

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
    crazy_update[0].tag_name = 'v2'
    let payload = {
      type: 'Update',
      app_infos: mock,
      serial: 12,
      old_serial: 10,
      updating: [],
    } as UpdateResponse

    const updateMultiple = vi.spyOn(db, 'updateMultiple')
    const insertMultiple = vi.spyOn(db, 'insertMultiple')
    const satlastUpdateSerial = vi.spyOn(handlers, 'setlastUpdateSerial')
    const setIsUpdating = vi.spyOn(handlers, 'setIsUpdating')
    const setlastUpdate = vi.spyOn(handlers, 'setlastUpdate')

    await updateHandler(payload, handlers.db, handlers.appInfo, () => 10, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)

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
      old_serial: 10,
      updating: ['15'],
    } as UpdateResponse

    await updateHandler(payload, handlers.db, handlers.appInfo, () => 10, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
    expect(appInfo['15'].state).toBe(AppState.Updating)

    const download: DownloadResponseOkay = {
      type: 'DownloadOkay',
      app_id: mock[3].app_id,
      name: 'test',
      data: 'test',
    }

    await updateHandler(download, handlers.db, handlers.appInfo, handlers.getLastSerial, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
    expect(appInfo['15'].state).toBe(AppState.Received)
  })

  test('Handles partial updates', async () => {
    const db = new AppInfoDB('storetesting4')
    const [appInfo, setAppInfo] = createStore(to_app_infos_by_id(mock))
    const handlers = {
      db,
      ...general_handlers,
      appInfo,
      setAppInfo,
    }

    const payload = {
      type: 'Update',
      app_infos: [{
        app_id: '12',
        tag_name: 'v10',
        description: 'pupu',
      }],
      serial: 12,
      old_serial: 10,
      updating: ['15'],
    } as UpdateResponse

    await updateHandler(payload, handlers.db, handlers.appInfo, () => 10, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
    expect(await db.get('12')).toStrictEqual({ ...mock[0], description: 'pupu' })
  })

  test('Handles Remove', async () => {
    const db = new AppInfoDB('storetesting5')
    const [appInfo, setAppInfo] = createStore(to_app_infos_by_id(mock))
    const handlers = {
      db,
      ...general_handlers,
      appInfo,
      setAppInfo,
    }

    const payload = {
      type: 'Update',
      app_infos: [{
        app_id: '12',
        tag_name: 'v10',
        description: 'pupu',
      }],
      serial: 12,
      old_serial: 10,
      updating: ['15'],
    } as UpdateResponse

    await updateHandler(payload, handlers.db, handlers.appInfo, () => 10, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
    expect(await db.get('12')).toStrictEqual({ ...mock[0], description: 'pupu' })
  })
})
