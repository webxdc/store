import type { Component } from 'solid-js'
import { Show, createMemo } from 'solid-js'
import { useStorage } from 'solidjs-use'
import { render } from 'solid-js/web'
import type { AppInfo } from '../bindings/AppInfo'
import type { ReceivedStatusUpdate } from '../webxdc'
import AppInfoPreview from '../components/AppInfo'
import mock from '../mock'
import '../index.sass'
import 'virtual:uno.css'
import '@unocss/reset/tailwind.css'
import type { SubmitRequest } from '../bindings/SubmitRequest'

const Submit: Component = () => {
  const [appInfo, setAppInfo] = useStorage('app-info', {} as AppInfo)
  const [lastSerial, setlastSerial] = useStorage('last-serial', 0)
  const is_appdata_complete = createMemo(() => Object.values(appInfo()).reduce((init, v) => init && !(v === undefined || v === null || v === ''), true))
  let lastAppinfo: AppInfo = {} as AppInfo
  const is_different = createMemo(() => JSON.stringify(appInfo()) !== JSON.stringify(lastAppinfo))
  const has_loaded = createMemo(() => Object.hasOwn(appInfo(), 'version'))

  if (import.meta.env.DEV) {
    lastAppinfo = mock
    setAppInfo(mock)
  }

  window.webxdc.setUpdateListener((resp: ReceivedStatusUpdate<AppInfo>) => {
    setlastSerial(resp.serial)
    // skip events that have a request_type and are hence self-send
    if (!Object.hasOwn(resp.payload, 'request_type')) {
      if (!has_loaded()) {
        lastAppinfo = resp.payload
      }
      setAppInfo(resp.payload)
      console.log('Received app info', appInfo())
    }
  }, lastSerial())

  function submit() {
    lastAppinfo = appInfo()
    window.webxdc.sendUpdate({
      payload: { Submit: { app_info: appInfo() } } as SubmitRequest,
    }, '')
  }

  return (
        <div class="c-grid m-4">
            <div class="min-width flex flex-col gap-3">
                <h1 class="text-center text-2xl font-bold text-indigo-500"> App Metadata</h1>
                <Show when={has_loaded()} fallback={
                    <p>Waiting for setup message...</p>
                }>
                    <AppInfoPreview appinfo={appInfo()} setAppInfo={setAppInfo} />
                    {is_different() && <button class="btn" disabled={!is_appdata_complete()} onclick={submit}> Submit </button>}
                </Show>
            </div>
        </div>
  )
}

const root = document.getElementById('root')
render(() => <Submit />, root!)
