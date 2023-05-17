import { Component, Show, createSignal } from 'solid-js';
import Submit from './submit'
import Shop from './shop'

const App: Component = () => {
  const [showPublish, setPublish] = createSignal(true)

  return (
    <div class="c-grid">
      <Show when={showPublish()} fallback={
        <Shop></Shop>
      }>
        <Submit />
      </Show>
    </div>
  )
};

export default App;
