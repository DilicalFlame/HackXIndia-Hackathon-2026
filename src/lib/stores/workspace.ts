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
                console.log('Not running in Tauri, skipping workspace creation');
                return false;
            }
        },
        getFiles: async (workspaceName: string) => {
            if (isTauri) {
                try {
                    return await invoke<string[]>('get_workspace_files', { workspaceName });
                } catch (error) {
                    console.error('Failed to get workspace files:', error);
                    return [];
                }
            } else {
                return [];
            }
        },
        addFile: async (workspaceName: string) => {
             if (isTauri) {
                try {
                    // @ts-ignore
                    const { open } = await import('@tauri-apps/plugin-dialog');
                    const selected = await open({
                        multiple: true,
                        title: 'Select files to add to workspace'
                    });

                    if (selected) {
                        const files = Array.isArray(selected) ? selected : [selected]; 
                        // Implementation handled in UI component usually, but keeping stub here
                        return true;
                     }
                     return false;
                } catch (e) {
                    return false;
                }
             }
             return false;
        },
        saveAnnotations: async (workspaceName: string, fileName: string, document: any) => {
            if (isTauri) {
                try {
                    await invoke('save_annotations', { workspaceName, fileName, document });
                    return true;
                } catch (error) {
                    console.error('Failed to save annotations:', error);
                    return false;
                }
            }
            return false;
        },
        loadAnnotations: async (workspaceName: string, fileName: string) => {
            if (isTauri) {
                try {
                    return await invoke<any>('load_annotations', { workspaceName, fileName });
                } catch (error) {
                    console.error('Failed to load annotations:', error);
                    return null;
                }
            }
            return null;
        }
    };
}

export const workspaces = createWorkspaceStore();
