import { For, render } from 'solid-js/web'
import { Match, Show, Switch, createEffect, createMemo, createSignal } from 'solid-js'
import { createStore } from 'solid-js/store'
import '~/index.sass'
import 'virtual:uno.css'
import '@unocss/reset/tailwind.css'
import type { Component } from 'solid-js'
import { useStorage } from 'solidjs-use'
import Fuse from 'fuse.js'
import { formatDistanceToNow } from 'date-fns'
import OutdatedView from './components/Outdated'
import type { WebxdcStatusUpdatePayload } from '~/bindings/WebxdcStatusUpdatePayload'

import { AppInfoDB } from '~/db/store_db'
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

type DownloadResponseOkay = Extract<WebxdcStatusUpdatePayload, { type: 'DownloadOkay' }>
type UpdateResponse = Extract<WebxdcStatusUpdatePayload, { type: 'Update' }>

function AppInfoModal(item: AppInfoWithState, onDownload: () => void, onForward: () => void, onRemove: () => void, onDragStart?: (ev: DragEvent) => void) {
  const [isExpanded, setIsExpanded] = createSignal(false)
  const summary = item.description.split('\n')[0]
  const description = item.description.slice(summary.length + 1)
  return (
    <li class="w-full p-3">
      <div class="flex cursor-pointer items-center justify-between gap-2" onClick={() => setIsExpanded(!isExpanded())}>
        <img
          src={`${item.image}`}
          alt={item.name}
          class="h-16 w-16 rounded-xl object-cover"
          ondragstart={onDragStart}
          draggable={onDragStart && (item.state === AppState.Received || item.state === AppState.Updating)} />
        <div class="flex-grow-1 overflow-hidden">
          <h2 class="text-xl font-semibold">{item.name}</h2>
          <p class="max-width-text truncate text-gray-600">{summary}</p>
          <button class="text-blue-700">
            {isExpanded() ? 'Less' : 'More'}
          </button>
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
              <div class="i-ri-share-forward-fill text-white"></div>
            </button>
          </Match>
          <Match when={item.state === AppState.Updating}>
            <div class="i-eos-icons:bubble-loading text-black"></div>
            <button class="send-button bg-green-500" onClick={onForward}>
              <div class="i-ri-share-forward-fill text-white"></div>
            </button>
          </Match>
        </Switch>
      </div >
      <Show when={isExpanded()}>
        <div class="flex flex-col">
          <p class="my-2 text-gray-600">{description}</p>
          <div class="my-2">
            <p class="text-sm text-gray-600"><span class="font-bold"> Date: </span>{new Date(Number(item.date) * 1000).toLocaleDateString()} ({item.tag_name})</p>
            <div class="flex items-center gap-1">
              <p class="text-sm text-gray-600"><span class="font-bold"> Size: </span>{(Number(item.size) / 1000).toFixed(1).toString()} kb</p>
              <Show when={(item.state === AppState.Received || item.state === AppState.Updating)}>
                -
                <button class="text-sm text-blue-500" onClick={onRemove}>Remove</button>
              </Show>
            </div>
            <p class="break-all text-sm text-gray-600"><span class="font-bold"> Source: </span>{item.source_code_url}</p>
          </div>
        </div>
      </Show>
    </li >
  )
}

interface AppListProps {
  items: AppInfoWithState[]
  search_query: string
  onDownload: (id: string) => void
  onForward: (id: string) => void
  onRemove: (id: string) => void
  onDragStart: (ev: DragEvent, item: AppInfoWithState) => void
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
    <Show when={filtered_items().length !== 0} fallback={
      <Show when={props.search_query.length > 0}><p class="text-center unimportant">No results for "{props.search_query}"</p></Show>}>
      <For each={filtered_items() || props.items}>
        {(item, index) => (
          <>
            {AppInfoModal(item, () => props.onDownload(item.app_id), () => { props.onForward(item.app_id) }, () => props.onRemove(item.app_id), event => props.onDragStart(event, item))}
            {index() !== filtered_items().length - 1 && <hr />}
          </>
        )
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
  const [isUpdating, setIsUpdating] = createSignal(false)
  const [query, setSearch] = createSignal('')
  const [showCommit, setShowCommit] = createSignal(false)
  const cached = createMemo(() => Object.values(appInfo).filter(app_info => app_info.state !== AppState.Initial))

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
      setAppInfo(mock)
      setlastSerial(1)
    }
  })

  window.webxdc.setUpdateListener(async (resp: ReceivedStatusUpdate<UpdateResponse | DownloadResponseOkay>) => {
    updateHandler(resp.payload, db, appInfo, lastSerial, setAppInfo, setlastUpdateSerial, setIsUpdating, setlastUpdate, setUpdateNeeded, setUpdateReceived)
    setlastSerial(resp.serial)
  }, lastSerial())

  async function update() {
    setIsUpdating(true)
    const cached_apps = cached().map(app_info => ([app_info.app_id, app_info.tag_name] as [string, string]))
    window.webxdc.sendUpdate({
      payload: { type: 'UpdateRequest', serial: lastUpdateSerial(), apps: cached_apps } as WebxdcStatusUpdatePayload,
    }, '')
  }

  async function handleDownload(app_id: string) {
    setAppInfo(app_id, 'state', AppState.Downloading)
    window.webxdc.sendUpdate({
      payload: { type: 'Download', app_id } as WebxdcStatusUpdatePayload,
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

  const supportsDraggingOut = !!(window as any).webxdc_custom?.desktopDragFileOut
  async function onDragStart(ev: DragEvent, item: AppInfoWithState) {
    if (supportsDraggingOut) {
      ev.preventDefault()
      const file = await db.get_webxdc(item.app_id)
      if (file === undefined) {
        throw new Error('No cached file found')
      }
      if (!Object.keys(file).includes('base64')) {
        console.error('Only base64 files are supported for drag-sending')
        return
      }
      (window as any).webxdc_custom?.desktopDragFileOut?.(file.name, (file as any).base64, item.image)
    }
  }

  return (
    <div>
      <div class="c-grid" classList={{ 'blur-xl': updateNeeded() }}>
        <div class="min-width">
          {/* app list */}
          <div class="min-h-screen flex flex-col">
            <div class="my-4 flex items-start justify-center gap-2 p-2">
              <div class="flex flex-col items-start gap-1">
                <input class="border-2 rounded-2xl px-3 py-1" placeholder={`Search ${Object.keys(appInfo).length} webxdc apps`} onInput={event => setSearch((event.target as HTMLInputElement).value)} />
              </div>
              <button class="rounded-1/2 p-2 btn">
                <div class="i-carbon-search text-blue-700" />
              </button>
            </div>
            <hr />
            <Show when={!(lastSerial() === 0)} fallback={<div class="text-center unimportant mt-5">
              <p class="mb-2">
                This Webxdc Store is not connected to a Bot to get apps from.
              </p>
              <p>
                To get a working Webxdc store, you need to send "Hi" to a store address, for example
                xstore@testrun.org, which is a community hosted instance with some curated apps.
              </p>
            </div>}>
              <ul class="w-full flex flex-grow flex-col gap-1 p-2">
                <AppList
                  items={Object.values(appInfo).sort((a, b) => Number(b.date - a.date))} search_query={query()}
                  onDownload={handleDownload}
                  onForward={handleForward}
                  onRemove={handleRemove}
                  onDragStart={onDragStart}></AppList>
              </ul>
              <hr />
              <div class="flex flex-col flex-wrap justify-center gap-2 py-4 pb-5 xs:flex-row">
                <button class="font-thin unimportant" onClick={() => setShowCommit(!showCommit())}>
                  Last update: {isUpdating() ? 'Updating..' : `${formatDistanceToNow(lastUpdate())} ago`}
                </button>
                <Show when={!isUpdating()}>
                  <span class="hidden unimportant xs:block">-</span>
                  <button class="text-blue-500" onclick={update}>
                    Update
                  </button>
                </Show>
              </div>
            </Show>
          </div>
        </div >
      </div>
      {/* modals */}
      <Show when={updateNeeded()}>
        <OutdatedView updated_received={updateReceived()} serial={lastUpdateSerial()} />
      </Show>
      {showCommit() && <p class="text-small mr-1 text-right text-sm text-gray-300"> {import.meta.env.VITE_COMMIT} </p>}
    </div>
  )
}

const root = document.getElementById('root')
render(() => <Store />, root!)
