import { type Component, createSignal } from 'solid-js'
import type { UpdateRequest } from '../bindings/UpdateRequest'

interface OutdatedViewProps {
  critical: boolean
  children: any
}

const AppInfoPreview: Component<OutdatedViewProps> = (props) => {
  const [buttonUsed, setButtonUsed] = createSignal(false)

  const update_req = () => {
    window.webxdc.sendUpdate({ payload: { update: true } as UpdateRequest }, '')
    setButtonUsed(true)
  }
  return (
    <>
      <div classList={{ 'blur overflow-hidden h-screen': props.critical }}> {props.children} </div>
      {props.critical
        && <div class="absolute right-0 top-0 grid h-screen w-screen place-content-center p-2">
          <div class="flex flex-col gap-2 border border-red rounded bg-white p-2">
            <h1 class='text-center font-bold text-red-700'> Outdated Version </h1>
            <p> A newer version of this webxdc is available. </p>
            {!buttonUsed() && <button class="self-center btn" onclick={update_req}> Download </button>}
            {buttonUsed() && <p class="self-center unimportant"> Downloading.. </p>}
          </div>
        </div>
      }
    </>
  )
}

export default AppInfoPreview
