import type { Component } from 'solid-js'
import { Show, createMemo, createSignal } from 'solid-js'
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
import type { SubmitResponse } from '../bindings/SubmitResponse'
import { isAppInfo } from '../utils'

function isSubmitResponse(p: any): p is SubmitResponse {
  return Object.prototype.hasOwnProperty.call(p, 'okay')
}

const Submit: Component = () => {
  const [appInfo, setAppInfo] = useStorage('app-info', {} as AppInfo)
  const [lastSerial, setlastSerial] = useStorage('last-serial', 0)
  const is_appdata_complete = createMemo(() => Object.values(appInfo()).reduce((init, v) => init && !(v === undefined || v === null || v === ''), true))
  let lastAppinfo: AppInfo = {} as AppInfo
  const is_different = createMemo(() => JSON.stringify(appInfo()) !== JSON.stringify(lastAppinfo))
  const has_loaded = createMemo(() => Object.prototype.hasOwnProperty.call(appInfo(), 'version'))
  const [showButton, setShowButton] = useStorage('show_submit', true)
  const [success, setSuccess] = createSignal<undefined | boolean>(undefined)

  if (import.meta.env.DEV) {
    lastAppinfo = mock
    setAppInfo(mock)
  }

  window.webxdc.setUpdateListener((resp: ReceivedStatusUpdate<AppInfo>) => {
    setlastSerial(resp.serial)
    if (isSubmitResponse(resp.payload)) {
      if (resp.payload.okay) {
        setSuccess(true)
      }
      else {
        setSuccess(false)
        setShowButton(true)
      }
    }
    else if (isAppInfo(resp.payload)) {
      lastAppinfo = resp.payload
      setAppInfo(resp.payload)
    }
  }, lastSerial())

  function submit() {
    setShowButton(false)
    lastAppinfo = appInfo()
    window.webxdc.sendUpdate({
      payload: { Submit: { app_info: appInfo() } } as SubmitRequest,
    }, '')
  }

  return (
    <div class="c-grid m-4">
      <div class="min-width flex flex-col gap-3">
        <h1 class="text-center text-2xl font-bold text-blue-500"> App Metadata</h1>
        <Show when={has_loaded()} fallback={
          <p>Waiting for setup message...</p>
        }>
          <AppInfoPreview appinfo={appInfo()} setAppInfo={setAppInfo} disable_all={false} />
          {success() === false
            && <p class="text-red"> Some problem occured while publishing your app. We will get in touch with you soon. </p>
          }
          {success() === true
            && <p class="text-green">I've send your app to some reviewers. We will soon get in touch with you again!</p>
          }
          {showButton() && <button class="w-full cursor-pointer font-semibold btn" classList={{ 'bg-gray-100 border-gray-500 text-gray-700': !is_different(), 'text-blue-500': is_different() }}
            disabled={!is_different() && !is_appdata_complete()} onClick={submit}>Submit</button>}
        </Show>
      </div>
    </div>
  )
}

const root = document.getElementById('root')
render(() => <Submit />, root!)
