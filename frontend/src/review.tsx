import { Component, Show, createEffect, createMemo } from 'solid-js';
import { AppInfo } from './bindings/AppInfo';
import { useStorage } from 'solidjs-use';
import { ReceivedStatusUpdate } from './webxdc';

interface CompProps {
    appinfo: AppInfo
    setAppInfo(appInfo: AppInfo): void;
}

const Form: Component<CompProps> = (props) => {
    const handleInputChange = (e: Event) => {
        const target = e.target as HTMLInputElement;
        const name = target.name;
        const value = target.value;
        props.setAppInfo({ ...props.appinfo, [name]: value });
    };

    let errors = createMemo(() => Object.values(props.appinfo).map((v) => v === undefined || v === null || v === ''));

    return (
        <form class="flex flex-col gap-2 p-4 m-4 rounded shadow max-width bg-white border rounded">
            <Show when={!errors()[5]} fallback={
                <div class="flex flex-col items-center">
                    <img src="https://via.placeholder.com/150" alt={props.appinfo.name} class="w-20 h-20 rounded-xl" />
                    <p class="text-red">Please add a image.png to your webxdc bundle</p>
                </div>
            }>
                <img src={"data:image/png;base64," + props.appinfo.image!} alt={props.appinfo.name} class="w-20 h-20 rounded-xl self-center" />
            </Show>
            <label>App Name</label>
            <Show when={errors()[0]}><p class="text-red">You have to give a name</p></Show>
            <input name="name" value={props.appinfo.name} onInput={handleInputChange} placeholder="App Name" />
            <label>Description</label>
            <Show when={errors()[1]}><p class="text-red">You have to give a description</p></Show>
            <textarea name="description" value={props.appinfo.description} onInput={handleInputChange} placeholder="Description" />
            <label>Author</label>
            <input name="author_name" value={props.appinfo.author_name} onInput={handleInputChange} placeholder="Author Name" />
            <label>Author Email</label>
            <input name="author_email" value={props.appinfo.author_email || ''} disabled placeholder="Author Email" />
            <label>Source code url </label>
            <Show when={errors()[4]}><p class="text-red">Please specify a source code url in your manifest.toml</p></Show>
            <input name="source_code_url" value={props.appinfo.source_code_url || ''} disabled placeholder="Source Code URL" />
            <Show when={errors()[4]}><p class="text-red">Please specify a version in your manifest.toml</p></Show>
            <label>Version</label>
            <input name="version" value={props.appinfo.version || ''} disabled placeholder="Version" />
        </form>
    )
}

const AppDetail: Component = () => {

    const appInfo_mock: AppInfo = {
        name: "",
        description: "",
        author_name: "",
        author_email: "",
        source_code_url: "",
        image: "",
        version: "",
    };

    const [appInfo, setAppInfo] = useStorage('app-info', appInfo_mock)
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
        setAppInfo(appInfo_mock);
    }

    return (
        <div class="c-grid h-screen">
            <Show when={appInfo() !== undefined} fallback={
                <p>Waiting for setup message...</p>
            }>
                <Form appinfo={appInfo()} setAppInfo={setAppInfo} />
            </Show>
        </div>
    )
};

export default AppDetail;

