import { Component, createSignal } from 'solid-js';
import { For } from "solid-js/web";

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
      image_url: "https://via.placeholder.com/640"
    },
    {
      name: "App 2",
      author_name: "Author 2",
      author_email: "author2@example.com",
      source_code_url: "https://github.com/author2/app2",
      description: "This is a description for App 2.",
      xdc_blob_url: "https://blobstore.com/app2",
      version: "2.0.0",
      image_url: "https://via.placeholder.com/640"
    },
  ])

  return (
    <div class="c-grid h-screen">
      <div class="min-width">
        <h1 class="text-3xl font-semibold mb-4">Shop Items</h1>
        <ul>
          <For each={appInfo()}>
            {item =>
              <li class="p-4 rounded shadow">
                <div class="flex justify-between">
                  <h2 class="text-xl font-semibold">{item.name}</h2>
                  <span class="text-gray-600">@{item.version}</span>
                </div>
                <div class="flex gap-2">
                  <img src={item.image_url} alt={item.name} class="w-64 h-64 object-cover mb-4" />
                  <div>
                    <p class="text-gray-600">Author: {item.author_name}</p>
                    <p class="text-gray-600">Email: {item.author_email}</p>
                    <a href={item.source_code_url} class="text-blue-500 hover:underline">Source Code</a>
                    <p class="my-2 text-gray-600">{item.description}</p>
                    <a href={item.xdc_blob_url} class="text-blue-500 hover:underline">Download</a>
                  </div>
                </div>
              </li>
            }
          </For>
        </ul>
      </div>
    </div>
  );
};

export default App;
