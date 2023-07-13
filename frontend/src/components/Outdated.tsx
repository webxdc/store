import { type Component, Show, createSignal } from 'solid-js'
import type { WebxdcStatusUpdatePayload } from '../bindings/WebxdcStatusUpdatePayload'

interface OutdatedViewProps {
  updated_received: boolean
}

type UpdateRequest = Extract<WebxdcStatusUpdatePayload, { type: 'UpdateWebxdc' }>

const AppInfoPreview: Component<OutdatedViewProps> = (props) => {
  const [buttonUsed, setButtonUsed] = createSignal(false)

  const update_req = () => {
    window.webxdc.sendUpdate({ payload: { type: 'UpdateWebxdc' } as UpdateRequest }, '')
    setButtonUsed(true)
  }
  return (
    <div class="absolute right-0 top-0 grid h-screen w-screen place-content-center p-2">
      <div class="flex flex-col gap-2 border border-red-700 rounded-xl bg-white p-4">
        <h1 class='text-center font-bold text-red-700'> Outdated Version </h1>
        <p> A newer version of the store is available. </p>
        <Show when={!buttonUsed()} fallback={
          <Show when={props.updated_received} fallback={
            <div class="self-center btn">

              <div class="loading-spinner border border-blue-500 rounded" i-eos-icons-loading></div>
            </div>
          }>
            <p class="self-center unimportant">
              Update received in chat
            </p>
          </Show>
        }>
          <button class="self-center btn" onclick={update_req}> Download </button>
        </Show>
      </div>
    </div>
  )
}

export default AppInfoPreview
