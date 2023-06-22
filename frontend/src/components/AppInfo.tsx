import type { Component } from 'solid-js'
import type { AppInfo } from '../bindings/AppInfo'

interface AppInfoPreviewProps {
  appinfo: AppInfo
  setAppInfo(appInfo: AppInfo): void
  disable_all: boolean
}

const AppInfoPreview: Component<AppInfoPreviewProps> = (props) => {
  const handleInputChange = (e: Event) => {
    const target = e.target as HTMLInputElement
    const name = target.name
    const value = target.value
    props.setAppInfo({ ...props.appinfo, [name]: value })
  }

  return (
    <form class="max-width flex flex-col border rounded bg-white p-4 shadow">
      <img src={`data:image/png;base64,${props.appinfo.image!}`} alt={props.appinfo.name} class="h-20 w-20 self-center rounded-xl" />
      <label>App Name</label>
      <input class="mb-3" name="name" value={props.appinfo.name} onInput={handleInputChange} disabled placeholder="App Name" />
      <label>Description</label>
      <textarea class="mb-3" name="description" value={props.appinfo.description} onInput={handleInputChange} disabled placeholder="Description" />
      <label>Submitter</label>
      <input class="mb-3" name="submitter_uri" value={props.appinfo.submitter_uri || ''} onInput={handleInputChange} disabled={props.disable_all} placeholder="Submitter Uri" />
      <label>Source code url </label>
      <input class="mb-3" name="source_code_url" value={props.appinfo.source_code_url ?? ''} disabled placeholder="Source Code URL" />
      <label>Version</label>
      <input class="mb-3" name="version" value={props.appinfo.version} disabled placeholder="Version" />
      <div class="flex items-center gap-2">
        <label>I own all rights to publish this software</label>
        <input type="checkbox" name="rights_provided" disabled={props.disable_all} />
      </div>
    </form>
  )
}

export default AppInfoPreview
