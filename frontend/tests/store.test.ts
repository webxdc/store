import { describe, expect, test, vi } from 'vitest'
import { type Setter } from 'solid-js'
import type { SetStoreFunction } from 'solid-js/store'
import { createStore } from 'solid-js/store'
import { AppInfoDB } from '../src/db/store_db'
import { AppState } from '../src/types'
import type { AppInfoWithState, AppInfosById } from '../src/types'
import type { DownloadResponseError, DownloadResponseOkay, InitResponse, UpdateResponse } from '../src/store-logic'
import { updateHandler } from '../src/store-logic'
import type { WebxdcOutdatedResponse, WebxdcUpdateSentResponse } from '../src/utils'
import 'fake-indexeddb/auto'
import mock from '../src/mock'
import type { AppInfo } from '~/bindings/AppInfo'

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
      app_id: 'app_12',
      name: 'test',
      data: 'test',
    }

    const setAppInfo = vi.spyOn(handlers, 'setAppInfo')
    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.getLastSerial, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
    expect(await db.get_webxdc(payload.app_id)).matchSnapshot()
    expect(await db.get(payload.app_id)).toStrictEqual({ ...mock.app_12, state: AppState.Received })
    expect(setAppInfo).toHaveBeenCalledWith(payload.app_id, 'state', AppState.Received)

    // Test updating an existing webxdc
    await db.add_webxdc({ name: 'aa', plainText: 'bee' }, 'app_16')

    payload = {
      type: 'DownloadOkay',
      app_id: 'app_16',
      name: 'test',
      data: 'test',
    }

    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.getLastSerial, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
    expect(setAppInfo).toHaveBeenCalledWith(payload.app_id, 'state', AppState.Received)
    expect(await db.get(payload.app_id)).toStrictEqual({ ...mock.app_16, state: AppState.Received })
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
      type: 'Init',
      app_infos: Object.values(mock),
      serial: 12,
    } as InitResponse

    const setAppInfo = vi.spyOn(handlers, 'setAppInfo')
    const satlastUpdateSerial = vi.spyOn(handlers, 'setlastUpdateSerial')
    const setIsUpdating = vi.spyOn(handlers, 'setIsUpdating')
    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.getLastSerial, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)

    const apps_with_initial_state = Object.keys(mock).reduce((res, key) => {
      res[key] = { ...mock[key], state: AppState.Initial }
      return res
    }, {} as AppInfosById)
    expect(setAppInfo).toHaveBeenCalledWith(apps_with_initial_state)
    expect(await db.get_all()).toStrictEqual(Object.values(apps_with_initial_state))
    expect(satlastUpdateSerial).toHaveBeenCalledWith(12)
    expect(setIsUpdating).toHaveBeenCalledWith(false)
  })

  test('Handles ongoing AppIndex updates', async () => {
    const db = new AppInfoDB('storetesting3')

    // Create some initial state
    const advanced_state = { app_12: mock.app_12, app_15: mock.app_15 }
    await db.insertMultiple(Object.values(advanced_state))
    expect(await db.get_all()).toStrictEqual(Object.values(advanced_state))

    const [appInfo, setAppInfo] = createStore(advanced_state as Record<string, AppInfoWithState>)

    const handlers = {
      ...general_handlers,
      db,
      appInfo,
      setAppInfo,
    }

    const crazy_update = Object.keys(mock).reduce((res, key) => {
      res[key] = { ...mock[key] }
      // @ts-expect-error true updates do not have state
      delete res[key].state
      return res
    }, {} as Record<string, AppInfo>)
    crazy_update.app_12.tag_name = 'v2'
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

    expect(insertMultiple).toHaveBeenCalledWith([{ ...mock.app_13, state: AppState.Initial }, { ...mock.app_14, state: AppState.Initial }, { ...mock.app_16, state: AppState.Initial }])
    expect(updateMultiple).toHaveBeenCalledWith([mock.app_12, mock.app_15])
    expect(await db.get_all()).toMatchSnapshot()
    expect(satlastUpdateSerial).toHaveBeenCalledWith(12)
    expect(setIsUpdating).toHaveBeenCalledWith(false)
    expect(setlastUpdate).toHaveBeenCalled()

    payload = {
      type: 'Update',
      app_infos: mock,
      serial: 12,
      old_serial: 10,
      updating: ['app_15'],
    } as UpdateResponse

    await updateHandler(payload, handlers.db, handlers.appInfo, () => 10, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
    expect(appInfo.app_15.state).toBe(AppState.Updating)

    const download: DownloadResponseOkay = {
      type: 'DownloadOkay',
      app_id: 'app_15',
      name: 'test',
      data: 'test',
    }

    await updateHandler(download, handlers.db, handlers.appInfo, handlers.getLastSerial, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
    expect(appInfo.app_15.state).toBe(AppState.Received)
  })

  test('Handles partial updates', async () => {
    const db = new AppInfoDB('storetesting4')
    const [appInfo, setAppInfo] = createStore(mock)
    const handlers = {
      db,
      ...general_handlers,
      appInfo,
      setAppInfo,
    }

    const payload = {
      type: 'Update',
      app_infos: {
        app_12: {
          app_id: 'app_12',
          tag_name: 'v10',
          description: 'pupu',
        },
      },
      serial: 12,
      old_serial: 10,
      updating: [],
    } as UpdateResponse

    await updateHandler(payload, handlers.db, handlers.appInfo, () => 10, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
    expect(await db.get('app_12')).toStrictEqual({ ...mock.app_12, description: 'pupu' })
  })

  test('Handles Remove', async () => {
    const db = new AppInfoDB('storetesting5')
    const [appInfo, setAppInfo] = createStore(mock)
    const handlers = {
      db,
      ...general_handlers,
      appInfo,
      setAppInfo,
    }

    const payload = {
      type: 'Update',
      app_infos: {
        app_12: null,
      },
      serial: 12,
      old_serial: 10,
      updating: [],
    } as UpdateResponse

    const removeSpy = vi.spyOn(db, 'remove_multiple_app_infos')
    const cacheDeleteSpy = vi.spyOn(db, 'remove_webxdc')
    await updateHandler(payload, handlers.db, handlers.appInfo, () => 10, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
    expect(removeSpy).not.toHaveBeenCalledWith([['app_12']])
    expect(cacheDeleteSpy).not.toHaveBeenCalledWith([['app_12']])
  })
})
