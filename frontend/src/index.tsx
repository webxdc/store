import { render } from 'solid-js/web'
import { Route, Router, Routes } from '@solidjs/router'
import { createContext } from 'solid-js'
import { createStore } from 'solid-js/store'
import Store from '~/pages/store'
import Info from '~/pages/info'
import '~/index.sass'
import 'virtual:uno.css'
import '@unocss/reset/tailwind.css'
import type { StoreRequest } from '~/bindings/StoreRequest'

const metaData = {
  version: import.meta.env.VITE_COMMIT,
  updating: false,
  last_update: new Date(),
  last_update_serial: 0,
  // List of [app_id, version] for cached apps
  cached: [] as [string, number][],
}

const [meta, setMeta] = createStore(metaData)

const wrapper = {
  meta,
  setMeta,
  updateDone: (serial: number) => {
    setMeta('last_update', new Date())
    setMeta('last_update_serial', serial)
    setMeta('updating', false)
  },
  setCached: (cached: [string, number][]) => setMeta('cached', cached),
  update: () => {
    setMeta('updating', true)
    window.webxdc.sendUpdate({
      payload: { Update: { serial: meta.last_update_serial, apps: meta.cached } } as StoreRequest,
    }, '')
  },
}

export const metadataContext = createContext(wrapper)

render(
  () => (
    <metadataContext.Provider value={wrapper}>
      <Router>
        <Routes>
          <Route path="/" element={<Store />} />
          <Route path="/info" element={<Info />} />
        </Routes>
      </Router>
    </metadataContext.Provider>
  ),
  document.getElementById('root')!,
)
