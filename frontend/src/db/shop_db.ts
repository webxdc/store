import { AppInfo } from "../bindings/AppInfo";


export class AppInfoDB {
    private dbName: string;
    private db: IDBDatabase | undefined;

    constructor(dbName: string) {
        this.dbName = dbName;
    }

    open(): Promise<IDBDatabase> {
        if (this.db) return Promise.resolve(this.db);
        return new Promise((resolve, reject) => {
            const request = indexedDB.open(this.dbName, 3);
            request.onerror = () => reject(request.error);
            request.onsuccess = () => resolve((this.db = request.result));
            request.onupgradeneeded = () => {
                const db = request.result;
                console.log('wat');
                db.createObjectStore("appInfo", { keyPath: "id" });
            };
        });
    }

    async insert(data: AppInfo): Promise<void> {
        const db = await this.open();
        return new Promise((resolve, reject) => {
            const transaction = db.transaction("appInfo", "readwrite");
            transaction.onerror = () => reject('reason: ' + transaction.error);
            const store = transaction.objectStore("appInfo");
            const request = store.add(data);
            request.onsuccess = () => { resolve() };
        });
    }

    async update(data: AppInfo): Promise<void> {
        const db = await this.open();
        return new Promise((resolve, reject) => {
            const transaction = db.transaction("appInfo", "readwrite");
            transaction.onerror = () => reject(transaction.error);
            const store = transaction.objectStore("appInfo");
            const request = store.put(data);
            request.onsuccess = () => { resolve() };
        });
    }

    async insertMultiple(data: AppInfo[]): Promise<void> {
        const db = await this.open();
        return new Promise((resolve, reject) => {
            const transaction = db.transaction("appInfo", "readwrite");
            transaction.onerror = () => reject(transaction.error);
            const store = transaction.objectStore("appInfo");
            data.forEach((item: AppInfo) => store.add(item));
            transaction.oncomplete = () => { resolve() };
        });
    }

    async updateMultiple(data: AppInfo[]): Promise<void> {
        const db = await this.open();
        return new Promise((resolve, reject) => {
            const transaction = db.transaction("appInfo", "readwrite");
            transaction.onerror = () => reject(transaction.error);
            const store = transaction.objectStore("appInfo");
            data.forEach((item: AppInfo) => store.put(item));
            transaction.oncomplete = () => { resolve() };
        });
    }

    async get_all(): Promise<AppInfo[]> {
        const db = await this.open();
        return new Promise((resolve, reject) => {
            const transaction = db.transaction("appInfo", "readonly");
            transaction.onerror = () => reject(transaction.error);
            const store = transaction.objectStore("appInfo");
            const request = store.openCursor();
            const result: AppInfo[] = [];
            request.onsuccess = () => {
                const cursor = request.result;
                if (cursor) {
                    result.push(cursor.value);
                    cursor.continue();
                } else {
                    resolve(result);
                }
            };
        });
    }
}