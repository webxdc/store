import { Component, } from 'solid-js';
import { AppInfo } from './bindings/AppInfo';


const AppDetail: Component = () => {

    const appInfo: AppInfo = {
        name: "Mock App",
        author_name: "John Doe",
        author_email: "johndoe@example.com",
        source_code_url: "https://github.com/johndoe/mockapp",
        image: "https://via.placeholder.com/150",
        description: "This is a mock app for demonstration purposes.",
        version: "1.0.0",
    };
    return (
        <div class="p-4 rounded shadow w-full bg-white">
            <img src={appInfo.image || ''} alt={appInfo.name} class="w-20 h-20 object-cover rounded-xl" />
            <h2 class="text-xl font-semibold">{appInfo.name}</h2>
            <p class="text-gray-600">{appInfo.description}</p>
            <p class="text-gray-600 text-sm">Author: {appInfo.author_name} {appInfo.author_email ? `<${appInfo.author_email}>` : ''}</p>
            {appInfo.source_code_url && <a href={appInfo.source_code_url} class="text-blue-500 hover:underline text-sm">Source Code</a>}
            <p class="text-gray-600 text-sm">Version: {appInfo.version || 'Unknown'}</p>
        </div>
    );
};

export default AppDetail;

