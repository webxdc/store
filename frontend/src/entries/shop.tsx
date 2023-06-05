import { Component, Show, createEffect, createMemo, createSignal } from 'solid-js';
import { For } from "solid-js/web";
import { useStorage } from 'solidjs-use';
import { ReceivedStatusUpdate } from '../webxdc';
import { format } from 'date-fns';
import Fuse from 'fuse.js';
import { DownloadResponse } from '../bindings/DownloadResponse';
import { UpdateResponse } from '../bindings/UpdateResponse';
import { createStore, reconcile } from 'solid-js/store';
import mock from '../mock'
import { AppInfoWithState, AppState, AppInfosById } from '../types';
import { render } from 'solid-js/web';
import '../index.sass';
import "virtual:uno.css"
import '@unocss/reset/tailwind.css'

const fuse_options = {
    keys: [
        "name",
        "author_name"
    ]
}

function isEmpty(obj: any) {
    for (var prop in obj) {
        if (obj.hasOwnProperty(prop))
            return false;
    }
    return true;
}

function isDownloadResponse(p: any): p is DownloadResponse {
    return Object.hasOwn(p, "okay")
}

function isUpdateResponse(p: any): p is UpdateResponse {
    return Object.hasOwn(p, "app_infos")
}


function AppInfoModal(item: AppInfoWithState, onDownload: () => void) {
    const [isExpanded, setIsExpanded] = createSignal(false);

    return (
        <li class="p-4 rounded shadow border w-full">
            <div class="flex justify-between items-center gap-2">
                <img src={"data:image/png;base64," + item.image!} alt={item.name} class="w-20 h-20 object-cover rounded-xl" />
                <div class="overflow-hidden flex-grow-1">
                    <h2 class="text-xl font-semibold">{item.name}</h2>
                    <p class="text-gray-600 truncate max-width-text">{item.description}</p>
                </div>
                {item.state == AppState.Initial && <button class="btn justify-self-center" onClick={onDownload}> Add </button>}
                {item.state == AppState.Downloading && <p class="unimportant"> Downloading.. </p>}
                {item.state == AppState.Received && <p class="text-amber-400 font-bold"> Received in Chat </p>}
            </div>
            {
                isExpanded() && (
                    <>
                        <p class="my-2 text-gray-600">{item.description}</p>
                        <hr />
                        <div class="my-2">
                            <p class="text-gray-600 text-sm"><span class="font-bold"> author:</span> {item.author_name}</p>
                            <p class="text-gray-600 text-sm"><span class="font-bold"> contact:</span>  {item.author_email}</p>
                            <p class="text-gray-600 text-sm"><span class="font-bold"> source code:</span>  {item.source_code_url}</p>
                        </div>
                    </>
                )
            }
            <div class="flex justify-center">
                <button onClick={() => setIsExpanded(!isExpanded())} class={`text-indigo-500 ${isExpanded() ? 'i-carbon-up-to-top' : 'i-carbon-down-to-bottom'}`}>
                </button>
            </div>
        </li >
    )
}


const PublishButton: Component = () => {
    const [isOpen, setIsOpen] = createSignal(false)

    return (
        <button onClick={() => setIsOpen(true)} class="btn w-full">
            {isOpen() ? "You can send me your webxdc in our 1:1 chat and I will help you publish it." : "Publish your own app"}
        </button>
    )
}

const AppList: Component<{ items: AppInfoWithState[], search: string, onDownload: (id: bigint) => void }> = (props) => {
    let fuse: Fuse<AppInfoWithState> = new Fuse(props.items, fuse_options);

    createEffect(() => {
        fuse = new Fuse(props.items, fuse_options);
    })

    let filtered_items = createMemo(() => {
        if (props.search !== '') {
            return fuse!.search(props.search).map((fr) => fr.item)
        } else {
            props.items
        }
    })

    return (
        <For each={filtered_items() || props.items}>
            {
                item => AppInfoModal(item, () => { props.onDownload(item.id) })
            }
        </For>
    );
};


const Shop: Component = () => {
    const [appInfo, setAppInfo] = createStore({} as AppInfosById)
    const [lastSerial, setlastSerial] = useStorage('last-serial', 0)
    const [lastUpdateSerial, setlastUpdateSerial] = useStorage('last-update-serial', 0)
    const [lastUpdate, setlastUpdate] = useStorage('last-update', new Date())
    const [isUpdating, setIsUpdating] = createSignal(false)
    const [search, setSearch] = createSignal("")


    if (import.meta.env.DEV) {
        setAppInfo(Number(mock.id), mock)
    }

    if (appInfo == undefined) {
        setIsUpdating(true)
    }

    window.webxdc.setUpdateListener((resp: ReceivedStatusUpdate<UpdateResponse | DownloadResponse>) => {
        setlastSerial(resp.serial)

        // Skip events that have a request_type and are hence self-send
        if (!Object.hasOwn(resp.payload, "request_type")) {
            if (isUpdateResponse(resp.payload)) {
                console.log('Received Update')
                let app_infos: AppInfosById = resp.payload.app_infos.reduce((acc, appinfo) => {
                    let index = Number(appinfo.id)
                    acc[index] = { ...appinfo, state: AppState.Initial }
                    return acc
                },
                    {} as AppInfosById)

                if (isEmpty(appInfo)) {
                    // initially write the newest update to state
                    setAppInfo(app_infos)
                } else {
                    // all but the first update only overwrite existing properties
                    console.log('Reconceiling updates')
                    setAppInfo(reconcile(app_infos))
                }

                setlastUpdateSerial(Number(resp.payload.serial))
                setIsUpdating(false)
                setlastUpdate(new Date())

            } else if (isDownloadResponse(resp.payload)) {
                if (resp.payload.okay) {
                    // id is set if resp is okay
                    let id = resp.payload.id!
                    setAppInfo(Number(id), 'state', AppState.Received)
                }
            }
        }
    }, lastSerial())

    async function update() {
        setIsUpdating(true)
        window.webxdc.sendUpdate({
            payload: {
                request_type: "Update",
                data: lastUpdateSerial()
            }
        }, "")
    }

    function handleDownload(id: bigint) {
        setAppInfo(Number(id), 'state', AppState.Downloading)
        window.webxdc.sendUpdate({
            payload: {
                request_type: 'Dowload',
                data: id
            }
        }, "")
    }

    return (
        <div class="c-grid p-3">
            <div class="max-width min-width">
                <div class="flex gap-2 justify-between">
                    <h1 class="text-2xl font-bold">Webxdc Appstore</h1>
                    <div class="unimportant p-1 flex items-center gap-2">
                        <Show when={isUpdating()} fallback={
                            <button onclick={update}>
                                <span>{format(lastUpdate(), 'cccc HH:mm')}</span>
                            </button>
                        }>
                            Updating..
                        </Show>
                        <div class="rounded border border-indigo-500" classList={{ "loading-spinner": isUpdating() }} i-carbon-reset></div>
                    </div>
                </div>

                <div class="p-4 mt-5">
                    <ul class="flex flex-col gap-2 w-full">
                        <li class="w-full flex justify-center items-center gap-2 mb-3">
                            <input class="rounded-2xl border-2" onInput={(event) => setSearch((event.target as HTMLInputElement).value)} />
                            <button class="btn rounded-1/2 p-2">
                                <div class="i-carbon-search text-indigo-500" />
                            </button>
                        </li>
                        <AppList items={Object.values(appInfo)} search={search()} onDownload={handleDownload} ></AppList>
                        <li class="mt-3">
                            <PublishButton></PublishButton>
                        </li>
                    </ul>
                </div>
            </div >
        </div>
    );
};

const root = document.getElementById('root');
render(() => <Shop />, root!);

