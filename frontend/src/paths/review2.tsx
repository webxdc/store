import { Component, Show, createMemo } from 'solid-js';
import { FrontendAppInfo } from '../bindings/FrontendAppInfo';
import { useStorage } from 'solidjs-use';
import { ReceivedStatusUpdate } from '../webxdc';
import AppInfoPreview from '../components/AppInfo';

const SubmitHelper: Component = () => {
    const [appInfo, setAppInfo] = useStorage('app-info', {} as FrontendAppInfo)
    const [lastSerial, setlastSerial] = useStorage('last-serial', 0)
    const is_appdata_complete = createMemo(() => Object.values(appInfo()).reduce((init, v) => init && !(v === undefined || v === null || v === ''), true))
    let lastAppinfo: FrontendAppInfo = {} as FrontendAppInfo
    const is_different = createMemo(() => JSON.stringify(appInfo()) !== JSON.stringify(lastAppinfo))
    const has_loaded = createMemo(() => Object.hasOwn(appInfo(), "version"))

    if (import.meta.env.DEV) {
        const mock_info = {
            name: "Poll",
            description: "Poll app where you can create crazy cool polls. This is a very long description for the pepe.",
            author_name: "Jonas Arndt",
            author_email: "xxde@you.de",
            source_code_url: "https://example.com",
            image: "a",
            version: "1.11",
            id: "hi",
        }
        lastAppinfo = mock_info
        setAppInfo(mock_info);
    }

    window.webxdc.setUpdateListener((resp: ReceivedStatusUpdate<FrontendAppInfo>) => {
        setlastSerial(resp.serial)
        // skip events that have a request_type and are hence self-send
        if (!Object.hasOwn(resp.payload, "request_type")) {
            if (!has_loaded()) {
                lastAppinfo = resp.payload
            }
            setAppInfo(resp.payload)
            console.log("Received app info", appInfo())
        }
    }, lastSerial())

    function submit() {
        window.webxdc.sendUpdate({
            payload: {
                request_type: "",
                data: appInfo()
            }
        }, "")
    }

    return (
        <div class="c-grid m-4">
            <div class="min-width flex flex-col gap-3">
                <h1 class="text-2xl text-center font-bold text-indigo-500"> App Metadata</h1>
                <p> {is_appdata_complete().toString()}</p>
                <Show when={has_loaded()} fallback={
                    <p>Waiting for setup message...</p>
                }>
                    <AppInfoPreview appinfo={appInfo()} setAppInfo={setAppInfo} />
                    {is_different() && <button class="btn" disabled={!is_appdata_complete()} onclick={submit}> Submit </button>}
                </Show>
            </div>
        </div>
    )
};

export default SubmitHelper;

