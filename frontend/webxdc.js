window.webxdc = (() => {
  let updateListener = (_) => { }
  const updatesKey = '__xdcUpdatesKey__'
  window.addEventListener('storage', (event) => {
    if (event.key == null) {
      window.location.reload()
    }
    else if (event.key === updatesKey) {
      const updates = JSON.parse(event.newValue)
      const update = updates[updates.length - 1]
      update.max_serial = updates.length
      console.log(`[Webxdc] ${JSON.stringify(update)}`)
      updateListener(update)
    }
  })

  function getUpdates() {
    const updatesJSON = window.localStorage.getItem(updatesKey)
    return updatesJSON ? JSON.parse(updatesJSON) : []
  }

  const params = new URLSearchParams(window.location.hash.substr(1))
  return {
    selfAddr: params.get('addr') || 'device0@local.host',
    selfName: params.get('name') || 'device0',
    setUpdateListener: (cb, serial = 0) => {
      const updates = getUpdates()
      const maxSerial = updates.length
      updates.forEach((update) => {
        if (update.serial > serial) {
          update.max_serial = maxSerial
          cb(update)
        }
      })
      updateListener = cb
      return Promise.resolve()
    },
    getAllUpdates: () => {
      console.log('[Webxdc] WARNING: getAllUpdates() is deprecated.')
      return Promise.resolve([])
    },
    sendUpdate: (update, description) => {
      const updates = getUpdates()
      const serial = updates.length + 1
      const _update = { payload: update.payload, summary: update.summary, info: update.info, serial }
      updates.push(_update)
      window.localStorage.setItem(updatesKey, JSON.stringify(updates))
      _update.max_serial = serial
      console.log(`[Webxdc] description="${description}", ${JSON.stringify(_update)}`)
      updateListener(_update)
    },
    removeUpdateListener: () => {
      updateListener = (_) => { }
    },
  }
})()