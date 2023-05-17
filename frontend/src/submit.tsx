import { Component, ComponentProps, createSignal } from 'solid-js';
import { PublishRequest } from '../../bindings/PublishRequest'
import "./submit.module.sass"

const AppSubmitForm: ComponentProps<any> = (props: any) => {
    const [appName, setAppName] = createSignal('');
    const [description, setDescription] = createSignal('');
    const [hasRights, setHasRights] = createSignal(false);

    const onSubmit = (e: Event) => {
        e.preventDefault();
        window.webxdc.sendUpdate({
            payload: {
                request_type: "Publish",
                data: {
                    name: appName(),
                    description: description(),
                } as PublishRequest
            }
        }, "")
    };

    const onClose = (e: Event) => {
        e.preventDefault();
        props.onclose();
    }

    return (
        <div class="c-grid h-screen">
            <form onSubmit={onSubmit} class="p-4 rounded shadow max-width">
                <h2 class="text-2xl font-semibold mb-2" onClick={onClose}>Submit App</h2>
                <label class="block mb-2">
                    <span class="text-gray-700">App Name</span>
                    <input type="text" class="mt-1 block w-full" onInput={(e: any) => setAppName(e.target.value)} />
                </label>
                <label class="block mb-2">
                    <span class="text-gray-700">Description</span>
                    <textarea class="mt-1 block w-full" onInput={(e: any) => setDescription(e.target.value)}></textarea>
                </label>
                <label class="block mb-2">
                    <input type="checkbox" class="border-green-400" onClick={() => setHasRights(!hasRights())} />
                    <span class="ml-2 text-gray-700">I have all the rights to this software</span>
                </label>
                <button class="btn float-right">Submit</button>
            </form>
        </div>
    );
};

export default AppSubmitForm;