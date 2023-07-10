import { type Component, Show } from 'solid-js'
import { formatDistanceToNow } from 'date-fns'

interface Props {
  version: number
  updating: boolean
  last_update: Date
  onUpdate: () => void
  onClose: () => void
}

const Info: Component<Props> = (props) => {
  return (
    <div class="absolute top-0 m-2 h-screen w-screen flex items-center justify-center" onClick={props.onClose}>
      <div class="min-width flex flex-col gap-2 border rounded-xl bg-white p-4 shadow">
        <h1 class="text-2xl font-bold">
          Webxdc Store
        </h1>

        <div class="stretch-item">
          <p> Version: </p>
          <p> {props.version} </p>
        </div>
        <div class="stretch-item">
          <p>Last update: </p>
          <Show when={props.updating} fallback={
            <button class="flex items-center gap-2 px-2 unimportant btn" onclick={props.onUpdate}>
              <span>{formatDistanceToNow(props.last_update)}</span>
              <div class="border border-blue-500 rounded" i-material-symbols-sync></div>
            </button>
          }>
            <div class="flex items-center gap-2 unimportant">
              <span class="tracking-wide">Updating..</span>
              <div class="loading-spinner border border-blue-500 rounded" i-material-symbols-sync></div>
            </div>
          </Show>
        </div>
      </div>
    </div>
  )
}

export default Info
