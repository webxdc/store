import { type Component, Show, useContext } from 'solid-js'
import { formatDistanceToNow } from 'date-fns'
const Info: Component = () => {

  return (
    <div class="m-2 h-screen flex items-center justify-center">
      <div class="min-width flex flex-col gap-2">

        {/* <A class="flex items-center self-start justify-self-start gap-1 rounded-md px-2 btn" href='/'>
          <div class="border border-blue-500 rounded" i-carbon-arrow-left></div>
          <p>Back</p>
        </A> */}
        <div class="flex flex-col gap-2 border rounded-xl p-4 shadow">
          <div class="mb-2 flex items-center gap-2">
            <h1 class="flex-shrink text-2xl font-bold">
              Webxdc Store
            </h1>
          </div>

          <div class="stretch-item">
            <p> Version: </p>
            {/* <p> {meta.version} </p> */}
          </div>
          <div class="stretch-item">
            <p>Last update: </p>
            {/* <Show when={meta.updating} fallback={
              <button class="flex items-center gap-2 px-2 unimportant btn" onclick={update}>
                <span>{formatDistanceToNow(meta.last_update)}</span>
                <div class="border border-blue-500 rounded" i-material-symbols-sync></div>
              </button>
            }>
              <div class="flex items-center gap-2 unimportant">
                <span class="tracking-wide">Updating..</span>
                <div class="loading-spinner border border-blue-500 rounded" i-material-symbols-sync></div>
              </div>
            </Show> */}
          </div>
        </div>
      </div>
    </div>
  )
}

export default Info
