import { Component, ComponentProps, Show, createSignal } from 'solid-js';
import { For } from "solid-js/web";
import { Transition } from 'solid-transition-group';
import { useStorage } from 'solidjs-use';
import { ReceivedStatusUpdate } from './webxdc';
import { format } from 'date-fns';
import { AppInfo } from './bindings/AppInfo';

function create_item(item: AppInfo) {
    const [isExpanded, setIsExpanded] = createSignal(false);

    function onAdd() {
        window.webxdc.sendUpdate({
            payload: {
                request_type: 'Dowload',
                //@ts-ignore
                data: item.id
            }
        }, "")
    }

    return (
        <li class="p-4 rounded shadow border w-full">
            <div class="flex justify-between items-center gap-2">
                <img src={"data:image/png;base64," + item.image!} alt={item.name} class="w-20 h-20 object-cover rounded-xl" />
                <div class="overflow-hidden flex-grow-1">
                    <h2 class="text-xl font-semibold">{item.name}</h2>
                    <Transition name="slide-fade">
                        {!isExpanded() && <p class="text-gray-600 truncate max-width-text">{item.description}</p>}
                    </Transition>
                </div>
                <button class="btn justify-self-center" onClick={onAdd}> Add </button>
            </div>
            {isExpanded() && (
                <>
                    <p class="my-2 text-gray-600">{item.description}</p>
                    <hr />
                    <div>
                        <p class="text-gray-600 text-sm"><span class="font-bold"> author:</span> {item.author_name}</p>
                        <p class="text-gray-600 text-sm"><span class="font-bold"> contact:</span>  {item.author_email}</p>
                        <p class="text-gray-600 text-sm"><span class="font-bold"> source code:</span>  {item.source_code_url}</p>
                    </div>
                </>
            )}
            <div class="flex justify-center">
                <button onClick={() => setIsExpanded(!isExpanded())} class={`text-blue-500 ${isExpanded() ? 'i-carbon-up-to-top' : 'i-carbon-down-to-bottom'}`}>
                </button>
            </div>
        </li>
    )
}

const App: ComponentProps<any> = (props: any) => {
    const [appInfo, setAppInfo] = useStorage('app-info', [] as AppInfo[])

    if (import.meta.env.DEV) {
        setAppInfo([{ "id": "hi", "active": true, "author_email": "xrxve@testrun.org", "author_name": "administrator", "description": "uidae", "image": "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", "name": "Poll", "source_code_url": "https://github.com/webxdc/webxdc-poll", "version": "1.0.0", "xdc_blob_dir": "/home/sebastian/coding/appstore-bot/deltachat.db/db.sqlite-blobs/poll-3714810120.xdc" }])
    }

    const [lastSerial, setlastSerial] = useStorage('last-serial', 0)
    const [lastUpdate, setlastUpdate] = useStorage('last-update', new Date())
    const [isUpdating, setIsUpdating] = createSignal(false)


    window.webxdc.setUpdateListener((resp: ReceivedStatusUpdate<AppInfo[]>) => {
        console.log("Received update", resp)
        setlastSerial(resp.serial)
        //@ts-ignore
        if (resp.payload.request_type === undefined) {
            setAppInfo(resp.payload)
            setIsUpdating(false)
            setlastUpdate(new Date())
        }
    }, lastSerial())

    async function update() {
        setIsUpdating(true)
        const response = window.webxdc.sendUpdate({
            payload: {
                request_type: "Update",
                data: undefined
            }
        }, "")
    }

    function onopen() {
        props.onopen()
    }

    return (
        <div class="max-width min-width p-3">
            <div class="flex gap-2 justify-between">
                <h1 class="text-2xl font-bold">Webxdc Appstore</h1>
                <div class="text-gray p-1">
                    <Show when={isUpdating()} fallback={
                        <div>
                            <button onclick={update} class="flex items-center gap-2">
                                <span>{format(lastUpdate(), 'cccc H:m')}</span>
                                <div class="rounded border border-green-500" i-carbon-reset></div>
                            </button>
                        </div>
                    }>
                        updating...
                    </Show>
                </div>
            </div>

            <div class="p-4">
                <ul class="flex flex-col gap-2 w-full">
                    <li >
                        <button onClick={onopen} class="border border-green-400 p-2 rounded w-full">
                            publish your own app
                        </button>
                    </li>
                    <For each={appInfo()}>
                        {
                            item => create_item(item)
                        }
                    </For>
                </ul>
            </div>
        </div>
    );
};

export default App;
