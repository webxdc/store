import { type Component, createSignal, useContext } from 'solid-js'
import { metadataContext } from '~/index'

const Info: Component = () => {
  const [dateNow, setDateNow] = createSignal(new Date())
  setInterval(() => setDateNow(new Date()), 2000)
  /* <Show when={isUpdating()} fallback={
        <button class="flex items-center gap-2" onclick={handleUpdate}>
            <span>{formatDistance(lastUpdate(), dateNow())}</span>
            <div class="border border-blue-500 rounded" i-material-symbols-sync></div>
        </button>
    }>
        <div class="flex items-center gap-2">
            <span class="tracking-wide">Updating..</span>
            <div class="loading-spinner border border-blue-500 rounded" i-material-symbols-sync></div>
        </div>
    </Show> */

  const { meta } = useContext(metadataContext)

  return (
    <div class="c-grid h-screen place-content-center">
      <div class="min-width border rounded-xl p-4 shadow">
        <h1 class="flex-shrink text-2xl font-bold">
          Webxdc Store
        </h1>

        <div class="stretch-item">
          <p> Version: </p>
          <p class="unimportant"> {meta.version} </p>
        </div>
      </div>
    </div>
  )
}

export default Info
