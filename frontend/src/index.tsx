import { render } from 'solid-js/web'
import { createContext, createEffect, onMount } from 'solid-js'
import { createStore, unwrap } from 'solid-js/store'
import { Route, Router, Routes, hashIntegration } from '@solidjs/router'
import Info from './pages/info'
import Store from '~/pages/store'
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
      payload: { Update: { serial: meta.last_update_serial, apps: unwrap(meta.cached) } } as StoreRequest,
    }, '')
  },
}

export const metadataContext = createContext(wrapper)

render(
  () => {
    onMount(() => {
      const userString = localStorage.getItem('meta')
      if (!userString)
        return
      const parsed = JSON.parse(userString)
      setMeta(() => ({ ...parsed, last_update: new Date(parsed.last_update) }))
    })

    createEffect(() => localStorage.setItem('meta', JSON.stringify({ ...meta, last_update: meta.last_update.getTime() })))

    return (
      <metadataContext.Provider value={wrapper}>
        <Router source={hashIntegration()}>
          <Routes>
            <Route path="/" component={Store} />
            <Route path="/info" component={Info} />
          </Routes>
        </Router>
      </metadataContext.Provider>
    )
  },
  document.getElementById('root')!,
)
