import { describe, expect, it, test, vi } from 'vitest'
import { type Setter } from 'solid-js'
import type { SetStoreFunction } from 'solid-js/store'
import { createStore } from 'solid-js/store'
import { AppInfoDB } from '../src/db/shop_db'
import { AppState } from '../src/types'
import type { AppInfoWithState, AppInfosById } from '../src/types'
import type { DownloadResponseError, DownloadResponseOkay, UpdateResponse } from '../src/shop-logic'
import { to_app_infos_by_id, updateHandler } from '../src/shop-logic'
import type { WebxdcOutdatedResponse, WebxdcUpdateSentResponse } from '../src/utils'
import 'fake-indexeddb/auto'
import mock from '../src/mock'

const general_handlers = {
  appInfo: {} as AppInfosById,
  setAppInfo: ((() => { }) as SetStoreFunction<AppInfosById>),
  setlastUpdateSerial: ((() => { }) as Setter<number>),
  setIsUpdating: ((() => { }) as Setter<boolean>),
  setlastUpdate: ((() => { }) as Setter<Date>),
  setUpdateNeeded: ((() => { }) as Setter<boolean>),
  setUpdateReceived: ((() => { }) as Setter<boolean>),
}

describe('Shop receiving updates', () => {
  test('Handles outdated response', () => {
    const handlers = {
      db: new AppInfoDB('shoptesting'),
      ...general_handlers,
    }

    const payload = {
      type: 'Outdated',
      critical: true,
      version: 1,
    } as WebxdcOutdatedResponse

    const updateNeeded = vi.spyOn(handlers, 'setUpdateNeeded')
    updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)

    expect(updateNeeded).toHaveBeenCalledWith(true)
  })

  test('Handes update received', () => {
    const handlers = {
      db: new AppInfoDB('shoptesting'),
      ...general_handlers,
    }

    const payload = {
      type: 'UpdateSent',
    } as WebxdcUpdateSentResponse

    const setUpdateReceived = vi.spyOn(handlers, 'setUpdateReceived')
    updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)

    expect(setUpdateReceived).toHaveBeenCalledWith(true)
  })

  test('Handles download error', () => {
    const handlers = {
      db: new AppInfoDB('shoptesting'),
      ...general_handlers,
    }

    const payload = {
      type: 'DownloadError',
      error: 'error',
      app_id: 'test',
    } as DownloadResponseError

    const setAppInfo = vi.spyOn(handlers, 'setAppInfo')
    updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
    expect(setAppInfo).toHaveBeenCalledWith(payload.app_id, 'state', AppState.DownloadCancelled)
  })

  test('Handles download okay', async () => {
    const db = new AppInfoDB('shoptesting')
    const handlers = {
      ...general_handlers,
      db,
      appInfo: to_app_infos_by_id(mock),
    }

    await db.insertMultiple(mock)

    const payload = {
      type: 'DownloadOkay',
      app_id: mock[0].app_id,
      name: 'test',
      data: 'test',
    } as DownloadResponseOkay

    const setAppInfo = vi.spyOn(handlers, 'setAppInfo')
    updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)
    expect(await db.get_webxdc(payload.app_id)).matchSnapshot()
    expect(await db.get(payload.app_id)).toStrictEqual({ ...mock[0], state: AppState.Received })
    expect(setAppInfo).toHaveBeenCalledWith(payload.app_id, 'state', AppState.Received)
  })

  test('Handles new AppIndex', async () => {
    const db = new AppInfoDB('shoptesting2')
    const handlers = {
      db,
      ...general_handlers,
      appInfo: {},
    }

    const payload = {
      type: 'Update',
      app_infos: mock,
      serial: 12,
    } as UpdateResponse

    const setAppInfo = vi.spyOn(handlers, 'setAppInfo')
    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)

    const initial_mock = mock.map(app_info => ({ ...app_info, state: AppState.Initial } as AppInfoWithState))
    expect(setAppInfo).toHaveBeenCalledWith(to_app_infos_by_id(initial_mock))
    expect(await db.get_all()).toStrictEqual(Object.values(initial_mock))
  })

  it('Handles Partly Updates', async () => {
    const db = new AppInfoDB('shoptesting3')
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
    const payload = {
      type: 'Update',
      app_infos: mock,
      serial: 12,
    } as UpdateResponse

    const updateMultiple = vi.spyOn(db, 'updateMultiple')
    const insertMultiple = vi.spyOn(db, 'insertMultiple')

    await updateHandler(payload, handlers.db, handlers.appInfo, handlers.setAppInfo, handlers.setlastUpdateSerial, handlers.setIsUpdating, handlers.setlastUpdate, handlers.setUpdateNeeded, handlers.setUpdateReceived)

    // tests:
    // - Appinfo with state !== Initial are then in 'Updating'
    // - Appinfo with state === Initial are untouched
    expect(advanced_state).toMatchSnapshot()
    expect(insertMultiple).toHaveBeenCalledWith(mock.slice(2, undefined).map(app_info => ({ ...app_info, state: AppState.Initial } as AppInfoWithState)))
    expect(updateMultiple).toHaveBeenCalledWith(mock.slice(0, 2).map(app_info => ({ ...app_info, state: advanced_state[app_info.app_id].state !== AppState.Initial ? AppState.Updating : advanced_state[app_info.app_id].state } as AppInfoWithState)))
    expect(await db.get_all()).toMatchSnapshot()
  })
})
