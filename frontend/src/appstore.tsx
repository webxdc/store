import { Component, Show, createSignal } from 'solid-js';
import Submit from './submit'
import Shop from './shop'

const App: Component = () => {
  const [showPublish, setPublish] = createSignal(false)

  return (
    <div class="c-grid p-3">
      <Show when={showPublish()} fallback={
        <Shop onopen={() => { setPublish(true) }}></Shop>
      }>
        <Submit onclose={() => { setPublish(false) }} />
      </Show>
    </div>
  )
};

export default App;
