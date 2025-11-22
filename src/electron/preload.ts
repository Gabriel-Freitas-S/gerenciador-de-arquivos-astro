import { contextBridge, ipcRenderer } from 'electron';
import type { MovementPayload, StoragePayload } from './types.js';

contextBridge.exposeInMainWorld('archive', {
	login: (credentials: { login: string; password: string }) => ipcRenderer.invoke('auth:login', credentials),
	session: (token: string) => ipcRenderer.invoke('auth:session', { token }),
	logout: (token: string) => ipcRenderer.invoke('auth:logout', { token }),
	storage: {
		list: (token: string) => ipcRenderer.invoke('storage:list', { token }),
		create: (token: string, data: StoragePayload) =>
			ipcRenderer.invoke('storage:create', { token, data }),
	},
	movements: {
		list: (token: string) => ipcRenderer.invoke('movements:list', { token }),
		record: (token: string, data: MovementPayload) =>
			ipcRenderer.invoke('movements:record', { token, data }),
	},
});
