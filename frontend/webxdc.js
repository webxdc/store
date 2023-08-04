// debug friend: document.writeln(JSON.stringify(value));
// @ts-check
/** @type {import('./webxdc').Webxdc<any>} */
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
      const _update = {
        payload: update.payload,
        summary: update.summary,
        info: update.info,
        serial,
      }
      updates.push(_update)
      window.localStorage.setItem(updatesKey, JSON.stringify(updates))
      _update.max_serial = serial
      console.log(
        `[Webxdc] description="${description}", ${JSON.stringify(_update)}`,
      )
      updateListener(_update)
    },
    sendToChat: async (content) => {
      if (!content.file && !content.text) {
        alert('🚨 Error: either file or text need to be set. (or both)')
        return Promise.reject(
          'Error from sendToChat: either file or text need to be set',
        )
      }

      /** @type {(file: Blob) => Promise<string>} */
      const blob_to_base64 = (file) => {
        const data_start = ';base64,'
        return new Promise((resolve, reject) => {
          const reader = new FileReader()
          reader.readAsDataURL(file)
          reader.onload = () => {
            /** @type {string} */
            // @ts-expect-error
            const data = reader.result
            resolve(data.slice(data.indexOf(data_start) + data_start.length))
          }
          reader.onerror = () => reject(reader.error)
        })
      }

      let base64Content
      if (content.file) {
        if (!content.file.name) {
          return Promise.reject('file name is missing')
        }
        if (
          Object.keys(content.file).filter(key =>
            ['blob', 'base64', 'plainText'].includes(key),
          ).length > 1
        ) {
          return Promise.reject(
            'you can only set one of `blob`, `base64` or `plainText`, not multiple ones',
          )
        }

        // @ts-expect-error - needed because typescript imagines that blob would not exist
        if (content.file.blob instanceof Blob) {
          // @ts-expect-error - needed because typescript imagines that blob would not exist
          base64Content = await blob_to_base64(content.file.blob)
          // @ts-expect-error - needed because typescript imagines that base64 would not exist
        }
        else if (typeof content.file.base64 === 'string') {
          // @ts-expect-error - needed because typescript imagines that base64 would not exist
          base64Content = content.file.base64
          // @ts-expect-error - needed because typescript imagines that plainText would not exist
        }
        else if (typeof content.file.plainText === 'string') {
          base64Content = await blob_to_base64(
            // @ts-expect-error - needed because typescript imagines that plainText would not exist
            new Blob([content.file.plainText]),
          )
        }
        else {
          return Promise.reject(
            'data is not set or wrong format, set one of `blob`, `base64` or `plainText`, see webxdc documentation for sendToChat',
          )
        }
      }
      const msg = `The app would now close and the user would select a chat to send this message:\nText: ${content.text ? `"${content.text}"` : 'No Text'
        }\nFile: ${content.file
          ? `${content.file.name} - ${base64Content.length} bytes`
          : 'No File'
        }`
      if (content.file) {
        const confirmed = confirm(
          `${msg}\n\nDownload the file in the browser instead?`,
        )
        if (confirmed) {
          const element = document.createElement('a')
          element.setAttribute(
            'href',
            `data:application/octet-stream;base64,${base64Content}`,
          )
          element.setAttribute('download', content.file.name)
          document.body.appendChild(element)
          element.click()
          document.body.removeChild(element)
        }
      }
      else {
        alert(msg)
      }
    },
    importFiles: (filters) => {
      const element = document.createElement('input')
      element.type = 'file'
      element.accept = [
        ...(filters.extensions || []),
        ...(filters.mimeTypes || []),
      ].join(',')
      element.multiple = filters.multiple || false
      const promise = new Promise((resolve, _reject) => {
        element.onchange = (_ev) => {
          console.log('element.files', element.files)
          const files = Array.from(element.files || [])
          document.body.removeChild(element)
          resolve(files)
        }
      })
      element.style.display = 'none'
      document.body.appendChild(element)
      element.click()
      console.log(element)
      return promise
    },
  }
})();