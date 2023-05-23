import { Component, ComponentProps, Show, createMemo, createSignal } from 'solid-js';
import { PublishRequest } from './bindings/PublishRequest';


interface Errors {
    appName?: string;
    description?: string;
    hasRights?: string;
}

const AppSubmitForm: ComponentProps<any> = (props: any) => {
    const [appName, setAppName] = createSignal('');
    const [description, setDescription] = createSignal('');
    const [hasRights, setHasRights] = createSignal(false);

    const [errors, setErrors] = createSignal({} as Errors)

    const onSubmit = (e: Event) => {
        e.preventDefault();

        let errors: Errors = {}

        if (appName() == '') {
            errors['appName'] = 'You have to give an app name.'
        }
        if (description() == '') {
            errors['description'] = 'You have to give a description.'
        }
        if (!hasRights()) {
            errors['hasRights'] = 'You need to confirm that you own all the rights to this software.'
        }

        if (Object.keys(errors).length > 0) {
            setErrors(errors)
            return
        }

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
            <form onSubmit={onSubmit} class="p-4 rounded shadow border min-width max-width">
                <div class="flex justify-between items-center">
                    <h2 class="text-2xl mb-5 text-indigo-500">Submit App</h2>
                    <button class="i-carbon-home" onClick={onClose}></button>
                </div>
                <label class="block text-gray-700"> App name </label>
                <Show when={errors().appName}><p class="text-red font-normal">{errors().appName}</p></Show>
                <input type="text" class="mt-1 block w-full mb-2" onInput={(e: any) => setAppName(e.target.value)} />
                <label class="block text-gray-700"> Description </label>
                <Show when={errors().description}><p class="text-red font-normal">{errors().description}</p></Show>
                <textarea class="mt-1 block w-full mb-2" onInput={(e: any) => setDescription(e.target.value)}></textarea>
                <Show when={errors().hasRights}><p class="text-red font-normal">{errors().hasRights}</p></Show>
                <div class="flex items-center gap-2 mb-2">
                    <input type="checkbox" class="border-indigo-500" onClick={() => setHasRights(!hasRights())} />
                    <label class="block text-gray-700"> I have all the rights to this software </label>
                </div>
                <button class="btn w-full mt-5">Submit</button>
            </form>
        </div>
    );
};

export default AppSubmitForm;