import type { Component } from 'solid-js'
import { Show, createMemo, createSignal } from 'solid-js'
import { useStorage } from 'solidjs-use'
import { render } from 'solid-js/web'
import type { AppInfo } from '../bindings/AppInfo'
import AppInfoPreview from '../components/AppInfo'
import mock from '../mock'
import '../index.sass'
import 'virtual:uno.css'
import '@unocss/reset/tailwind.css'
import { isAppInfo } from '../utils'
import type { ReviewResponse } from '../bindings/ReviewResponse'
import type { ReviewRequest } from '../bindings/ReviewRequest'
import eruda from 'eruda'

if (import.meta.env.DEV) {
  console.log('Starting eruda')
  eruda.init()
}

interface TestStatus {
  android: boolean
  ios: boolean
  desktop: boolean
}

interface ReviewStateProps {
  testStatus: TestStatus
  setTestStatus(appInfo: TestStatus): void
}

function isReviewResponse(p: any): p is ReviewResponse {
  return Object.prototype.hasOwnProperty.call(p, 'okay')
}

const ReviewState: Component<ReviewStateProps> = (props) => {
  const handleInputChange = (e: Event) => {
    const target = e.target as HTMLInputElement
    const name = target.name
    const value = target.checked
    props.setTestStatus({ ...props.testStatus, [name]: value })
  }

  return (
    <form class="max-width flex flex-col gap-2 border rounded bg-white p-4 shadow">
      <label class="flex items-center">
        <input class="mb-2" type="checkbox" name='android' checked={props.testStatus.android} onClick={handleInputChange} />
        <span class="ml-2">Works on Android</span>
      </label>
      <label class="flex items-center">
        <input class="mb-2" type="checkbox" name='ios' checked={props.testStatus.ios} onClick={handleInputChange} />
        <span class="ml-2">Works on IOS</span>
      </label>
      <label class="flex items-center">
        <input class="mb-2" type="checkbox" name='desktop' checked={props.testStatus.desktop} onClick={handleInputChange} />
        <span class="ml-2">Works on Desktop</span>
      </label>
    </form>
  )
}

const Review: Component = () => {
  const [appInfo, setAppInfo] = useStorage('app-info', {} as AppInfo)
  const [testStatus, setTestStatus] = useStorage('test-status', { android: false, ios: false, desktop: false })
  const [lastSerial, setlastSerial] = useStorage('last-serial', 0)
  const [showButton, setShowButton] = createSignal(true)
  const [success, setSuccess] = createSignal<undefined | boolean>(undefined)

  if (import.meta.env.DEV) {
    setAppInfo(mock[0])
  }

  const is_appdata_complete = createMemo(() => Object.values(appInfo()).reduce((init, v) => init && !(v === undefined || v === null || v === ''), true))
  const is_testing_complete = createMemo(() => testStatus().android && testStatus().ios && testStatus().desktop)
  const is_complete = createMemo(() => is_appdata_complete() && is_testing_complete())

  window.webxdc.setUpdateListener((resp) => {
    setlastSerial(resp.serial)
    if (isAppInfo(resp.payload)) {
      setAppInfo(resp.payload)
    }
    else if (isReviewResponse(resp.payload)) {
      setShowButton(!resp.payload.okay)
      setSuccess(resp.payload.okay)
    }
  }, lastSerial())

  function submit(e: any) {
    e.preventDefault()
    setShowButton(false)
    window.webxdc.sendUpdate({
      payload: { Publish: { app_info: appInfo().id } } as ReviewRequest,
    }, '')
  }

  return (
    <div class="c-grid m-4">
      <div class="min-width flex flex-col gap-3">
        <h1 class="text-center text-2xl font-bold text-blue-500"> App Publishing Status</h1>
        <Show when={appInfo() !== undefined} fallback={
          <p>Waiting for setup message...</p>
        }>
          <AppInfoPreview appinfo={appInfo()} setAppInfo={setAppInfo} disable_all={true} />
          <p class="unimportant">Testing Status</p>
          <ReviewState testStatus={testStatus()} setTestStatus={setTestStatus} />
          <p class="unimportant">
            {showButton() && <Show when={!is_complete()} fallback={
              <p>Ready for publishing!</p>
            }>
              <Show when={!is_appdata_complete()}>
                <p>TODO: Some app data is still missing</p>
              </Show>
              <Show when={!is_testing_complete()}>
                <p>TODO: Testing is incomplete</p>
              </Show>
            </Show>}
          </p>
          {showButton() && <button class="w-full btn" classList={{ 'text-gray-700  hover:bg-button': !is_complete() }}
            disabled={!is_complete()} onClick={submit}>Publish</button>}
          {success() === true && <p class="text-green-500">Successfully published!</p>}
          {success() === false && <p class="text-red">Some problem occured while trying to pubslish.</p>}
        </Show>
      </div>
    </div >
  )
}

const root = document.getElementById('root')
render(() => <Review />, root!)
