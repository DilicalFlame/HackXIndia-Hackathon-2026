import { writable } from 'svelte/store';
import { invoke } from '@tauri-apps/api/core';
import type { Workspace } from '../types';

function createWorkspaceStore() {
    const { subscribe, set, update } = writable<Workspace[]>([]);


    // Check if running in Tauri
    const isTauri = typeof window !== 'undefined' && (window.isTauri || !!window.__TAURI__);

    return {
        subscribe,
        loadWorkspaces: async () => {
            if (isTauri) {
                try {
                    const workspaces = await invoke<Workspace[]>('get_recent_workspaces');
                    set(workspaces);
                } catch (error) {
                    console.error('Failed to load workspaces:', error);
                }
            } else {
                console.log('Not running in Tauri, skipping workspace load');
                // Mock data for web if needed
                set([]);
            }
        },
        createWorkspace: async (name: string) => {
            if (isTauri) {
                try {
                    await invoke('create_workspace', { name });
                    // Refresh list
                    const workspaces = await invoke<Workspace[]>('get_recent_workspaces');
                    set(workspaces);
                    return true;
                } catch (error) {
                    console.error('Failed to create workspace:', error);
                    return false;
                }
            } else {
                console.log('Not running in Tauri, skipping workspace creation');
                return false;
            }
        }
    };
}

export const workspaces = createWorkspaceStore();
