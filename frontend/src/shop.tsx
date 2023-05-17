import { Component, ComponentProps, Show, createSignal } from 'solid-js';
import { For } from "solid-js/web";
import { Transition } from 'solid-transition-group';
import { useStorage } from 'solidjs-use';
import { AppInfo } from '../../bindings/AppInfo';
import { ReceivedStatusUpdate } from './webxdc';
import { format } from 'date-fns';

function create_item(item: AppInfo) {
    const [isExpanded, setIsExpanded] = createSignal(false);
    return (
        <li class="p-4 rounded shadow w-full">
            <div class="flex justify-between items-center gap-2">
                <img src={item.image!} alt={item.name} class="w-20 h-20 object-cover rounded-xl" />
                <div class="overflow-hidden flex-grow-1">
                    <h2 class="text-xl font-semibold">{item.name}</h2>
                    <Transition name="slide-fade">
                        {!isExpanded() && <p class="text-gray-600 truncate max-width-text">{item.description}</p>}
                    </Transition>
                </div>
                <button class="btn justify-self-center"> Add </button>
            </div>
            {isExpanded() && (
                <>
                    <p class="my-2 text-gray-600">{item.description}</p>
                    <div class='flex wrap gap-2'>
                        <p class="text-gray-600 text-sm">{item.author_name} &lt{item.author_email}&gt</p>
                        <a href={item.source_code_url!} class="text-blue-500 hover:underline text-sm">Source Code</a>
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
        <div class="max-width p-3">
            <div class="flex justify-between">
                <h1 class="text-2xl">Webxdc Appstore</h1>
                <div class="rounded border border-green text-gray p-1">
                    <Show when={isUpdating()} fallback={
                        <div>
                            <button onclick={update} class="flex items-center gap-2">
                                <span>Last update: {format(lastUpdate(), 'cccc H:m')}</span>
                                <div i-carbon-reset></div>
                            </button>
                        </div>
                    }>
                        updating..
                    </Show>
                </div>
            </div>

            <div class="c-grid p-4 item-stretch">
                <ul class="flex flex-col gap-2 w-full flex-grow-1">
                    <li class="button" onClick={onopen}>publish your own app</li>
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
