import type { Component } from 'solid-js'
import { Match, Show, Switch, createEffect, createMemo, createSignal } from 'solid-js'
import { For, render } from 'solid-js/web'
import { useStorage } from 'solidjs-use'
import { formatDuration, intervalToDuration } from 'date-fns'
import Fuse from 'fuse.js'
import { createStore } from 'solid-js/store'
import type { ReceivedStatusUpdate } from '../webxdc'
import type { ShopResponse } from '../bindings/ShopResponse'
import type { ShopRequest } from '../bindings/ShopRequest'
import mock from '../mock'
import type { AppInfoWithState, AppInfosById } from '../types'
import { AppState } from '../types'
import '../index.sass'
import 'virtual:uno.css'
import '@unocss/reset/tailwind.css'
import { AppInfoDB } from '../db/shop_db'
import OutdatedView from '../components/OutdatedView'

const fuse_options = {
  keys: [
    'name',
    'author_name',
    'description',
  ],
  threshold: 0.4,
}

type DownloadResponseOkay = Extract<ShopResponse, { type: 'DownloadOkay' }>
type UpdateResponse = Extract<ShopResponse, { type: 'Update' }>

function AppInfoModal(item: AppInfoWithState, onDownload: () => void, onForward: () => void) {
  const [isExpanded, setIsExpanded] = createSignal(false)

  return (
    <li class="w-full border rounded p-4 shadow" onClick={() => setIsExpanded(!isExpanded())}>
      <div class="flex items-center justify-between gap-2">
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
            <div class="i-eos-icons:bubble-loading text-white"></div>
            <button class="send-button bg-green-500" onClick={onForward}>
              <div class="i-material-symbols:forward text-white"></div>
            </button>
          </Match>
        </Switch>
      </div >
      {
        isExpanded() && (
          <>
            <p class="my-4 text-gray-600">{item.description}</p>
            <div class="my-2">
              <p class="text-sm text-gray-600"><span class="font-bold"> Submitter:</span>{item.submitter_uri}</p>
              <p class="break-words text-sm text-gray-600"><span class="font-bold"> Source code:</span>{item.source_code_url}</p>
              <p class="text-sm text-gray-600"><span class="font-bold"> Version:</span>{item.version}</p>
            </div>
          </>
        )
      }
      <div class="flex justify-center">
        <button class={`text-blue-800 ${isExpanded() ? 'i-carbon-up-to-top' : 'i-carbon-down-to-bottom'}`}>
        </button>
      </div>
    </li >
  )
}

/* const PublishButton: Component = () => {
  const [isOpen, setIsOpen] = createSignal(false)

  return (
    <button onClick={() => setIsOpen(true)} class="w-full border-gray-200 shadow btn">
      {isOpen() ? 'You can send me your webxdc in our 1:1 chat and I will help you publish it.' : 'Publish your own app'}
    </button>
  )
}
 */
const AppList: Component<{ items: AppInfoWithState[]; search: string; onDownload: (id: string) => void; onForward: (id: string) => void }> = (props) => {
  let fuse: Fuse<AppInfoWithState> = new Fuse(props.items, fuse_options)

  createEffect(() => {
    fuse = new Fuse(props.items, fuse_options)
  })

  const filtered_items = createMemo(() => {
    if (props.search !== '') {
      return fuse!.search(props.search).map(fr => fr.item)
    }
    else {
      return props.items
    }
  })

  return (
    <Show when={props.items.length !== 0} fallback={<p class="text-center unimportant">Loading Apps..</p>}>
      <For each={filtered_items() || props.items}>
        {
          item => AppInfoModal(item, () => props.onDownload(item.app_id), () => { props.onForward(item.app_id) })
        }
      </For>
    </Show>
  )
}

function to_app_infos_by_id<T extends { app_id: string }>(app_infos: T[]): Record<string, T> {
  return app_infos.reduce((acc, appinfo) => {
    acc[appinfo.app_id] = appinfo
    return acc
  }, {} as Record<string, T>)
}

const Shop: Component = () => {
  const [appInfo, setAppInfo] = createStore({} as AppInfosById)
  const [lastSerial, setlastSerial] = useStorage('last-serial', 0) // Last store-serial
  const [updateNeeded, setUpdateNeeded] = useStorage('update-needed', false) // Flag if the frontend is outdated
  const [updateReceived, setUpdateReceived] = useStorage('update-received', false)
  const [lastUpdateSerial, setlastUpdateSerial] = useStorage('last-update-serial', 0) // Last serial to initialize updateListener
  const [lastUpdate, setlastUpdate] = useStorage('last-update', new Date())
  const timeSinceLastUpdate = createMemo(() => intervalToDuration({
    start: lastUpdate(),
    end: new Date(),
  }))
  const [isUpdating, setIsUpdating] = createSignal(false)
  const [search, setSearch] = createSignal('')
  const [showCommit, setShowCommit] = createSignal(false) // Show the commit hash when heading was clicked
  const [showCache, setShowCache] = createSignal(false)
  const [cache, setCache] = useStorage('cache', {} as AppInfosById)

  const past_time = Math.abs(new Date().getTime() - lastUpdate().getTime()) / 1000
  if (appInfo === undefined || (past_time > 60 * 60)) {
    setIsUpdating(true)
  }

  const db = new AppInfoDB('webxdc')

  // This is for now _not_ synchronized with the update receival so a delayed
  // query could overwrite app updates. For now, this should be fine.
  db.get_all().then((apps) => {
    const app_infos = to_app_infos_by_id(apps)
    setAppInfo(app_infos)

    if (import.meta.env.DEV) {
      setAppInfo(to_app_infos_by_id(mock))
    }
  })

  window.webxdc.setUpdateListener(async (resp: ReceivedStatusUpdate<UpdateResponse | DownloadResponseOkay>) => {
    setlastSerial(resp.serial)
  }, lastSerial())

  async function handleUpdate() {
    setIsUpdating(true)
    window.webxdc.sendUpdate({
      payload: { Update: { serial: lastUpdateSerial() } } as ShopRequest,
    }, '')
  }

  async function handleDownload(app_id: string) {
    setAppInfo(app_id, 'state', AppState.Downloading)
    db.update({ ...appInfo[app_id], state: AppState.Downloading })
    window.webxdc.sendUpdate({
      payload: { Download: { app_id } } as ShopRequest,
    }, '')
  }

  async function handleForward(app_id: string) {
    const file = await db.get_webxdc(app_id)
    if (file === undefined) {
      throw new Error('No cached file found')
    }
    window.webxdc.sendToChat({ file })
  }

  return (
    <OutdatedView critical={updateNeeded()} updated_received={updateReceived()}>
      <div class="c-grid p-3">
        <div class="min-width">

          {/* header */}
          <div class="flex items-center justify-between gap-2">
            <div>
              <h1 class="flex-shrink text-2xl font-bold" onclick={() => setShowCommit(!showCommit())}>
                Webxdc Appstore
              </h1>
              {showCommit() && <p class="whitespace-nowrap text-sm unimportant"> @ {import.meta.env.VITE_COMMIT} </p>}
            </div>
            <div class="rounded-xl bg-gray-100 p-2 unimportant text-gray-500">
              <Show when={isUpdating()} fallback={
                <button class="flex items-center gap-2" onclick={handleUpdate}>
                  <span>{formatDuration(timeSinceLastUpdate(), { delimiter: ',' }).split(',')[0] || '0 sec'} ago</span>
                  <div class="border border-blue-500 rounded" i-material-symbols-sync></div>
                </button>
              }>
                <div class="flex items-center gap-2">
                  <span>Updating..</span>
                  <div class="loading-spinner border border-blue-500 rounded" i-material-symbols-sync></div>
                </div>
              </Show>
            </div>
          </div>

          {/* app list */}
          <div class="p-4">
            <ul class="w-full flex flex-col gap-2">
              <li class="my-5 w-full flex items-center justify-center gap-2">
                <input class="border-2 rounded-2xl" onInput={event => setSearch((event.target as HTMLInputElement).value)} />
                <button class="rounded-1/2 p-2 btn">
                  <div class="i-carbon-search" />
                </button>
              </li>
              <AppList items={Object.values(appInfo)} search={search()} onDownload={handleDownload} onForward={handleForward} ></AppList>
              {/* <li class="mt-3">
                <PublishButton></PublishButton>
              </li> */}
            </ul>
          </div>
        </div >
      </div>
    </OutdatedView>
  )
}

const root = document.getElementById('root')
render(() => <Shop />, root!)
