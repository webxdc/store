import { Component, Show, createSignal } from 'solid-js';
import { For } from "solid-js/web";
import { Transition } from 'solid-transition-group';
import { useStorage } from 'solidjs-use';
import { AppInfo } from '../../bindings/AppInfo';
import { RequestType } from '../../bindings/RequestType';
import { WebxdcRequest } from '../../bindings/WebxdcRequest';

import { ReceivedStatusUpdate } from './webxdc';


async function update() {
  const response = window.webxdc.sendUpdate({
    payload: {
      request_type: "Update",
    }
  }, "")
}



const App: Component = () => {
  const [appInfo, setAppInfo] = createSignal([
    {
      name: "App 1",
      author_name: "Author 1",
      author_email: "author1@example.com",
      source_code_url: "https://github.com/author1/app1",
      description: "This is a description for App 1.",
      xdc_blob_url: "https://blobstore.com/app1",
      version: "1.0.0",
      image: "https://via.placeholder.com/640"
    },
    {
      name: "App 2",
      author_name: "Author 2",
      author_email: "author2@example.com",
      source_code_url: "https://github.com/author2/app2",
      description: "This is a description for App 2.",
      xdc_blob_url: "https://blobstore.com/app2",
      version: "2.0.0",
      image: "https://via.placeholder.com/640"
    },
  ])

  window.webxdc.setUpdateListener((resp: ReceivedStatusUpdate<AppInfo[]>) => {
    console.log("Received update", resp)
    setAppInfo(resp.payload)
  })


  const [lastUpdate, setData] = useStorage('last-update', new Date())
  const [isUpdating, setIsUpdating] = createSignal(true)

  return (
    <div class="p-5">
      <div class="flex justify-between">
        <h1 class="text-2xl">Webxd Appstore</h1>
        <Show when={isUpdating()} fallback={<div> Last update: {lastUpdate().toDateString()} </div>}>
          updating
        </Show>
      </div>

      <div class="c-grid p-4">
        <ul class="flex flex-col gap-2">
          <For each={appInfo()}>
            {item => {
              const [isExpanded, setIsExpanded] = createSignal(false);
              return (
                <li class="p-4 bg-white rounded shadow appstore-list-item">
                  <div class="flex justify-between items-center gap-2">
                    <div class="flex gap-4">
                      <img src={item.image} alt={item.name} class="w-20 h-20 object-cover rounded-xl" />
                      <div>
                        <h2 class="text-xl font-semibold">{item.name}</h2>
                        <Transition name="slide-fade">
                          {!isExpanded() && <p class="text-gray-600 truncate">{item.description}</p>}
                        </Transition>
                      </div>
                    </div>
                    <button class="rounded bg-green-400 p-1 text-gray-200 justify-self-center"> Add </button>
                  </div>
                  {isExpanded() && (
                    <>
                      <p class="my-2 text-gray-600">{item.description}</p>
                      <p class="text-gray-600">Author: {item.author_name}</p>
                      <p class="text-gray-600">Email: {item.author_email}</p>
                      <a href={item.source_code_url} class="text-blue-500 hover:underline">Source Code</a>
                    </>
                  )}
                  <div class="flex justify-center">

                    <button onClick={() => setIsExpanded(!isExpanded())} class={`text-blue-500 ${isExpanded() ? 'i-carbon-down-to-bottom' : 'i-carbon-up-to-top'}`}>
                      {isExpanded() ? "Collapse" : "Expand"}
                    </button>
                  </div>
                </li>
              )
            }}
          </For>
        </ul>
      </div>
    </div>
  );
};

export default App;
