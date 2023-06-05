import { AppInfo } from "./bindings/AppInfo";
import { AppState } from "./types";

export default {
    id: BigInt(12),
    name: "Poll",
    description: "Poll app where you can create crazy cool polls. This is a very long description for the pepe.",
    author_name: "Jonas Arndt",
    author_email: "xxde@you.de",
    source_code_url: "https://example.com",
    image: "a",
    version: "1.11",
    state: AppState.Initial
} as AppInfo