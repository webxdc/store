import { Component, Show, createEffect, createMemo, createSignal } from 'solid-js';
import { AppInfo } from './bindings/AppInfo';
import { useStorage } from 'solidjs-use';
import { ReceivedStatusUpdate } from './webxdc';

interface AppInfoPreviewProps {
    appinfo: AppInfo
    setAppInfo(appInfo: AppInfo): void;
}

const AppInfoPreview: Component<AppInfoPreviewProps> = (props) => {
    const handleInputChange = (e: Event) => {
        const target = e.target as HTMLInputElement;
        const name = target.name;
        const value = target.value;
        props.setAppInfo({ ...props.appinfo, [name]: value });
    };

    let errors = createMemo(() => Object.values(props.appinfo).map((v) => v === undefined || v === null || v === ''));

    return (
        <form class="flex flex-col p-4 rounded shadow max-width bg-white border">
            <Show when={!errors()[5]} fallback={
                <div class="flex flex-col items-center">
                    <img src="https://via.placeholder.com/150" alt={props.appinfo.name} class="w-20 h-20 rounded-xl" />
                    <p class="text-red">Please add an image.png to your webxdc bundle</p>
                </div>
            }>
                <img src={"data:image/png;base64," + props.appinfo.image!} alt={props.appinfo.name} class="w-20 h-20 rounded-xl self-center" />
            </Show>
            <label>App Name</label>
            <Show when={errors()[0]}><p class="text-red">You have to give a name.</p></Show>
            <input class="mb-2" name="name" value={props.appinfo.name} onInput={handleInputChange} placeholder="App Name" />
            <label>Description</label>
            <Show when={errors()[1]}><p class="text-red">You have to give a description.</p></Show>
            <textarea name="description" value={props.appinfo.description} onInput={handleInputChange} placeholder="Description" />
            <label>Author</label>
            <Show when={errors()[2]}><p class="text-red">You have to give an author name.</p></Show>
            <input class="mb-2" name="author_name" value={props.appinfo.author_name} onInput={handleInputChange} placeholder="Author Name" />
            <label>Author Email</label>
            <input class="mb-2" name="author_email" value={props.appinfo.author_email || ''} disabled placeholder="Author Email" />
            <label>Source code url </label>
            <Show when={errors()[4]}><p class="text-red">Please specify a source code url in your manifest.toml.</p></Show>
            <input class="mb-2" name="source_code_url" value={props.appinfo.source_code_url || ''} disabled placeholder="Source Code URL" />
            <label>Version</label>
            <Show when={errors()[6]}><p class="text-red">Please specify a version in your manifest.toml.</p></Show>
            <input class="mb-2" name="version" value={props.appinfo.version || ''} disabled placeholder="Version" />
        </form>
    )
}

interface TestStatus {
    android: boolean;
    ios: boolean;
    desktop: boolean;
}

interface ReviewStateProps {
    testStatus: TestStatus;
    setTestStatus(appInfo: TestStatus): void;
}

const ReviewState: Component<ReviewStateProps> = (props) => {
    const handleInputChange = (e: Event) => {
        const target = e.target as HTMLInputElement;
        const name = target.name;
        const value = target.checked;
        props.setTestStatus({ ...props.testStatus, [name]: value });
    };

    return (
        <form class="flex flex-col gap-2 p-4 rounded shadow max-width bg-white border">
            <label class="flex items-center">
                <input class="mb-2" type="checkbox" name='android' checked={props.testStatus.android} onClick={handleInputChange} />
                <span class="ml-2">Works on Android</span>
            </label>
            <label class="flex items-center">
                <input class="mb-2" type="checkbox" name='ios' checked={props.testStatus.ios} onClick={handleInputChange} />
                <span class="ml-2">Works on iOS</span>
            </label>
            <label class="flex items-center">
                <input class="mb-2" type="checkbox" name='desktop' checked={props.testStatus.desktop} onClick={handleInputChange} />
                <span class="ml-2">Works on Desktop</span>
            </label>
        </form>
    );
};

const AppDetail: Component = () => {

    const [appInfo, setAppInfo] = useStorage('app-info', {
        name: "",
        description: "",
        author_name: "",
        author_email: "",
        source_code_url: "",
        image: "",
        version: "",
    } as AppInfo)

    const [testStatus, setTestStatus] = useStorage('test-status', { android: false, ios: false, desktop: false })
    const [lastSerial, setlastSerial] = useStorage('last-serial', 0)

    window.webxdc.setUpdateListener((resp: ReceivedStatusUpdate<AppInfo>) => {
        setlastSerial(resp.serial)
        //@ts-ignore
        if (resp.payload.request_type === undefined) {
            setAppInfo(resp.payload)
            console.log("Received app info", appInfo())
        }
    }, lastSerial())

    if (import.meta.env.DEV) {
        setAppInfo({
            name: "Poll",
            description: "Poll app where you can create crazy cool polls. This is a very long description for the pepe.",
            author_name: "Jonas Arndt",
            author_email: "xxde@you.de",
            source_code_url: "https://example.com",
            image: "a",
            version: "1.11",
        });
    }

    const is_appdata_complete = createMemo(() => Object.values(appInfo()).reduce((init, v) => init && !(v === undefined || v === null || v === ''), true))
    const is_testing_complete = createMemo(() => testStatus().android && testStatus().ios && testStatus().desktop)
    const is_complete = createMemo(() => is_appdata_complete() && is_testing_complete())

    function submit() {

    }

    return (
        <div class="c-grid m-4">
            <div class="min-width flex flex-col gap-3">
                <h1 class="text-2xl text-center font-bold text-indigo-500"> App Publishing statatus</h1>
                <Show when={appInfo() !== undefined} fallback={
                    <p>Waiting for setup message...</p>
                }>
                    <AppInfoPreview appinfo={appInfo()} setAppInfo={setAppInfo} />
                    <p class="text-gray-500 font-italic">Testing Status</p>
                    <div>
                        <ReviewState testStatus={testStatus()} setTestStatus={setTestStatus} />
                    </div>
                    <p class="text-gray-500 font-italic">
                        <Show when={!is_complete()} fallback={
                            <p>Ready for publishing!</p>
                        }>
                            <Show when={!is_appdata_complete()}>
                                <p>TODO: Some app data is still missing</p>
                            </Show>
                            <Show when={!is_testing_complete()}>
                                <p>TODO: Testing is incomplete</p>
                            </Show>
                        </Show>
                    </p>
                    <button class="btn font-semibold text-indigo-500 w-full" classList={{ "bg-gray-100 border-gray-500": !is_complete() }}
                        disabled={is_complete()} onClick={submit}>Submit</button>
                </Show>
            </div>
        </div>
    )
};

export default AppDetail;

