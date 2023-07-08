import type { Component } from 'solid-js'
import { Match, Show, Switch, createEffect, createMemo, createSignal, useContext } from 'solid-js'
import { For } from 'solid-js/web'
import { useStorage } from 'solidjs-use'
import Fuse from 'fuse.js'
import { createStore } from 'solid-js/store'
import { A } from '@solidjs/router'
import { metadataContext } from '~/index'
import type { StoreResponse } from '~/bindings/StoreResponse'
import type { StoreRequest } from '~/bindings/StoreRequest'
import { AppInfoDB } from '~/db/store_db'
import OutdatedView from '~/components/OutdatedView'
import { to_app_infos_by_id, updateHandler } from '~/store-logic'
import { AppState } from '~/types'
import type { AppInfoWithState, AppInfosById } from '~/types'
import mock from '~/mock'
import type { ReceivedStatusUpdate } from '~/webxdc'

const fuse_options = {
  keys: [
    'name',
    'author_name',
    'description',
  ],
  threshold: 0.4,
}

type DownloadResponseOkay = Extract<StoreResponse, { type: 'DownloadOkay' }>
type UpdateResponse = Extract<StoreResponse, { type: 'Update' }>

function AppInfoModal(item: AppInfoWithState, onDownload: () => void, onForward: () => void, onRemove: () => void) {
  const [isExpanded, setIsExpanded] = createSignal(false)

  return (
    <li class="w-full border rounded p-4 shadow">
      <div class="flex cursor-pointer items-center justify-between gap-2" onClick={() => setIsExpanded(!isExpanded())}>
        <img src={`data:image/png;base64,${item.image!}`} alt={item.name} class="h-20 w-20 rounded-xl object-cover" />
        <div class="flex-grow-1 overflow-hidden">
          <h2 class="text-xl font-semibold">{item.name}</h2>
          <p class="max-width-text truncate text-gray-600">{item.description}</p>
        </div>
        <Switch>
          <Match when={item.state === AppState.Initial}>
            <button class="send-button bg-blue-500" onClick={onDownload}>
              <div class="i-material-symbols:download text-white"></div>
            </button>
          </Match>
          <Match when={item.state === AppState.Downloading}>
            <div class="send-button bg-gray-500">
              <div class="i-eos-icons:bubble-loading text-white"></div>
            </div>
          </Match>
          <Match when={item.state === AppState.DownloadCancelled}>
            <div class="send-button bg-red-500" >
              <div class="i-material-symbols:error text-white"></div>
            </div>
          </Match>
          <Match when={item.state === AppState.Received}>
            <button class="send-button bg-green-500" onClick={onForward}>
              <div class="i-material-symbols:forward text-white"></div>
            </button>
          </Match>
          <Match when={item.state === AppState.Updating}>
            <div class="i-eos-icons:bubble-loading text-black"></div>
            <button class="send-button bg-green-500" onClick={onForward}>
              <div class="i-material-symbols:forward text-white"></div>
            </button>
          </Match>
        </Switch>
      </div >
      {
        isExpanded() && (
          <div class="flex flex-col">
            <p class="my-4 text-gray-600">{item.description}</p>
            <div class="my-2">
              <p class="text-sm text-gray-600"><span class="font-bold"> Submitter: </span>{item.submitter_uri}</p>
              <p class="break-all text-sm text-gray-600"><span class="font-bold"> Source code: </span>{item.source_code_url}</p>
              <p class="text-sm text-gray-600"><span class="font-bold"> Version: </span>{item.version}</p>
            </div>
            {(item.state === AppState.Received || item.state === AppState.Updating) && <button class="self-center btn-gray" onClick={onRemove}>Remove from cache</button>}
          </div>
        )
      }
      <div class="mt-1 flex justify-center" onClick={() => setIsExpanded(!isExpanded())}>
        <button class={`text-blue-800 ${isExpanded() ? 'i-carbon-up-to-top' : 'i-carbon-down-to-bottom'}`}>
        </button>
      </div>
    </li >
  )
}

interface AppListProps {
  items: AppInfoWithState[]
  search_query: string
  onDownload: (id: string) => void
  onForward: (id: string) => void
  onRemove: (id: string) => void
}

const AppList: Component<AppListProps> = (props) => {
  let fuse: Fuse<AppInfoWithState> = new Fuse(props.items, fuse_options)

  createEffect(() => {
    fuse = new Fuse(props.items, fuse_options)
  })

  const filtered_items = createMemo(() => {
    if (props.search_query !== '') {
      return fuse!.search(props.search_query).map(fr => fr.item)
    }
    else {
      return props.items
    }
  })

  return (
    <Show when={props.items.length !== 0} fallback={<p class="text-center unimportant">There are no apps</p>}>
      <For each={filtered_items() || props.items}>
        {
          item => AppInfoModal(item, () => props.onDownload(item.app_id), () => { props.onForward(item.app_id) }, () => props.onRemove(item.app_id))
        }
      </For>
    </Show>
  )
}

const Store: Component = () => {
  const [appInfo, setAppInfo] = createStore({} as AppInfosById)
  const [lastSerial, setlastSerial] = useStorage('last-serial', 0) // Last store-serial
  const [updateNeeded, setUpdateNeeded] = useStorage('update-needed', false) // Flag if the frontend is outdated
  const [updateReceived, setUpdateReceived] = useStorage('update-received', false)
  const [lastUpdateSerial, setlastUpdateSerial] = useStorage('last-update-serial', 0) // Last serial to initialize updateListener
  const [lastUpdate, setlastUpdate] = useStorage('last-update', new Date())

  const [query, setSearch] = createSignal('')
  const cached = createMemo(() => Object.values(appInfo).filter(app_info => app_info.state !== AppState.Initial))

  const { meta, setCached, update, updateDone } = useContext(metadataContext)
  createEffect(() => setCached(cached().map(app_info => ([app_info.app_id, app_info.version]))))

  // automatically update the app list
  const past_time = Math.abs(new Date().getTime() - lastUpdate().getTime()) / 1000
  if (appInfo === undefined || (past_time > 60 * 60)) {
    update()
  }

  const db = new AppInfoDB('webxdc')
  // This is for now _not_ synchronized with the update receival so a delayed
  // query could overwrite app updates. For now, this should be fine.
  db.get_all().then((apps) => {
    const app_infos = to_app_infos_by_id(apps)
    setAppInfo(app_infos)

    if (import.meta.env.DEV) {
      setAppInfo(to_app_infos_by_id(mock))
      setlastSerial(1)
    }
  })
  window.webxdc.setUpdateListener(async (resp: ReceivedStatusUpdate<UpdateResponse | DownloadResponseOkay>) => {
    updateHandler(resp.payload, db, appInfo, setAppInfo, updateDone, setUpdateNeeded, setUpdateReceived)
    setlastSerial(resp.serial)
  }, lastSerial())

  async function handleDownload(app_id: string) {
    setAppInfo(app_id, 'state', AppState.Downloading)
    window.webxdc.sendUpdate({
      payload: { Download: { app_id } } as StoreRequest,
    }, '')
  }

  async function handleForward(app_id: string) {
    const file = await db.get_webxdc(app_id)
    if (file === undefined) {
      throw new Error('No cached file found')
    }
    window.webxdc.sendToChat({ file })
  }

  async function handleRemove(app_id: string) {
    setAppInfo(app_id, 'state', AppState.Initial)
    db.remove_webxdc(app_id)
  }

  return (
    <OutdatedView critical={updateNeeded()} updated_received={updateReceived()}>
      <div class="c-grid p-3">
        <div class="min-width">

          {/* header */}
          <div class="flex items-center gap-2">
            <div>
              <h1 class="flex-shrink text-2xl font-bold">
                Webxdc Store
              </h1>
            </div>
            <A class="rounded-xl p-2 btn-gray" href='/info'>
              <Show when={meta.updating} fallback={
                <div class="border border-blue-500 rounded" i-carbon-information></div>
              }>
                <div class="loading-spinner border border-blue-500 rounded" i-material-symbols-sync></div>
              </Show>
            </A>
          </div>

          {/* app list */}
          <div class="mt-4 p-4">
            <ul class="w-full flex flex-col gap-2">
              <li class="w-full flex flex-col items-center justify-center gap-2">
                <div class="flex items-center justify-center gap-2">
                  <input class="border-2 rounded-2xl" onInput={event => setSearch((event.target as HTMLInputElement).value)} />
                  <button class="rounded-1/2 p-2 btn">
                    <div class="i-carbon-search" />
                  </button>
                </div>
              </li>
              <Show when={!(lastSerial() === 0)} fallback={
                <p class="text-center unimportant">Loading store..</p>
              }>
                <AppList
                  items={Object.values(appInfo)} search_query={query()}
                  onDownload={handleDownload}
                  onForward={handleForward}
                  onRemove={handleRemove} ></AppList>
              </Show>
            </ul>
          </div>
        </div >
      </div>
    </OutdatedView>
  )
}

export default Store
