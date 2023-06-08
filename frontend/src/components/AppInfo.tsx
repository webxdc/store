import { Component, Show, createMemo } from 'solid-js';
import { AppInfo } from '../bindings/AppInfo';

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

    const errors = createMemo(() => Object.entries(props.appinfo).reduce((acc, [key, v]) => {
        acc[key as keyof AppInfo] = v === undefined || v === null || v === ''
        return acc
    }, {} as { [key in keyof AppInfo]: boolean }
    ));

    return (
        <form class="flex flex-col p-4 rounded shadow max-width bg-white border">
            <Show when={!errors().image} fallback={
                <div class="flex flex-col items-center">
                    <img src="150.png" alt={props.appinfo.name} class="w-20 h-20 rounded-xl" />
                    <p class="text-red">Please add an image.png to your webxdc bundle</p>
                </div>
            }>
                <img src={"data:image/png;base64," + props.appinfo.image!} alt={props.appinfo.name} class="w-20 h-20 rounded-xl self-center" />
            </Show>
            <label>App Name</label>
            <Show when={errors().name}><p class="text-red">You have to give a name.</p></Show>
            <input class="mb-2" name="name" value={props.appinfo.name} onInput={handleInputChange} placeholder="App Name" />
            <label>Description</label>
            <Show when={errors().description}><p class="text-red">You have to give a description.</p></Show>
            <textarea name="description" value={props.appinfo.description} onInput={handleInputChange} placeholder="Description" />
            <label>Author</label>
            <Show when={errors().author_name}><p class="text-red">You have to give an author name.</p></Show>
            <input class="mb-2" name="author_name" value={props.appinfo.author_name} onInput={handleInputChange} placeholder="Author Name" />
            <label>Author Email</label>
            <input class="mb-2" name="author_email" value={props.appinfo.author_email || ''} disabled placeholder="Author Email" />
            <label>Source code url </label>
            <Show when={errors().source_code_url}><p class="text-red">Please specify a source code url in your manifest.toml.</p></Show>
            <input class="mb-2" name="source_code_url" value={props.appinfo.source_code_url || ''} disabled placeholder="Source Code URL" />
            <label>Version</label>
            <Show when={errors().version}><p class="text-red">Please specify a version in your manifest.toml.</p></Show>
            <input class="mb-2" name="version" value={props.appinfo.version || ''} disabled placeholder="Version" />
        </form>
    )
}

export default AppInfoPreview;