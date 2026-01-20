<script lang="ts">
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { workspaces } from "$lib/stores/workspace";
  import * as Resizable from "$lib/components/ui/resizable/index.js";
  import * as Tabs from "$lib/components/ui/tabs/index.js";
  import * as ScrollArea from "$lib/components/ui/scroll-area/index.js";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import { File, Plus, MessageSquare, Files, ArrowLeft } from "@lucide/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { goto } from "$app/navigation";

  const workspaceName = $page.params.name;
  let files: string[] = [];
  let isTauri = false;

  onMount(async () => {
    if (typeof window !== "undefined") {
      isTauri = !!window.__TAURI__;
    }
    await loadFiles();
  });

  async function loadFiles() {
    // @ts-ignore
    files = await workspaces.getFiles(workspaceName);
  }

  function goBack() {
    goto("/");
  }

  async function handleAddFile() {
    if (!isTauri) return;
    try {
      const selected = await open({
        multiple: true,
        title: "Select files to add",
      });

      if (selected) {
        // @ts-ignore
        const selections = Array.isArray(selected) ? selected : [selected];

        for (const filePath of selections) {
          // @ts-ignore
          if (filePath && typeof filePath === "string") {
            // Should be string
            await invoke("add_file_to_workspace", {
              workspaceName,
              filePath: filePath,
            });
          } else if (filePath && (filePath as any).path) {
            // Depending on version, it might be an object
            await invoke("add_file_to_workspace", {
              workspaceName,
              filePath: (filePath as any).path,
            });
          }
        }
        await loadFiles();
      }
    } catch (err) {
      console.error("Failed to add file", err);
    }
  }
</script>

<div class="h-screen w-full overflow-hidden bg-background text-foreground">
  <Resizable.PaneGroup
    direction="horizontal"
    class="h-full w-full rounded-lg border"
  >
    <Resizable.Pane defaultSize={20} minSize={15} maxSize={30}>
      <div class="flex h-full flex-col">
        <div class="flex items-center justify-between p-4 font-semibold">
          <div class="flex items-center gap-2 overflow-hidden">
            {#if isTauri}
              <Button
                variant="ghost"
                size="icon"
                onclick={goBack}
                class="h-8 w-8 shrink-0"
              >
                <ArrowLeft class="h-4 w-4" />
              </Button>
            {/if}
            <span class="truncate">{workspaceName}</span>
          </div>
          <Button variant="ghost" size="icon" onclick={handleAddFile}>
            <Plus class="h-4 w-4" />
          </Button>
        </div>
        <Separator />
        <ScrollArea.Root class="flex-1">
          <div class="p-4">
            <h4 class="mb-4 text-sm font-medium leading-none">Files</h4>
            <div class="space-y-2">
              {#each files as file}
                <div
                  class="flex items-center gap-2 rounded-md p-2 hover:bg-muted cursor-pointer text-sm"
                >
                  <File class="h-4 w-4 text-muted-foreground" />
                  <span class="truncate">{file}</span>
                </div>
              {:else}
                <div class="text-sm text-muted-foreground">No files yet.</div>
              {/each}
            </div>
          </div>
        </ScrollArea.Root>
      </div>
    </Resizable.Pane>
    <Resizable.Handle />
    <Resizable.Pane defaultSize={80}>
      <div class="h-full p-6">
        <Tabs.Root value="chat" class="h-full flex flex-col">
          <Tabs.List class="grid w-full grid-cols-2 lg:w-[400px]">
            <Tabs.Trigger value="chat">
              <MessageSquare class="mr-2 h-4 w-4" />
              Chat
            </Tabs.Trigger>
            <Tabs.Trigger value="files">
              <Files class="mr-2 h-4 w-4" />
              Opened Files
            </Tabs.Trigger>
          </Tabs.List>
          <Tabs.Content
            value="chat"
            class="flex-1 border rounded-md mt-4 p-4 text-muted-foreground"
          >
            Chat functionality coming soon...
          </Tabs.Content>
          <Tabs.Content
            value="files"
            class="flex-1 border rounded-md mt-4 p-4 text-muted-foreground"
          >
            Select a file from the sidebar to view it here.
          </Tabs.Content>
        </Tabs.Root>
      </div>
    </Resizable.Pane>
  </Resizable.PaneGroup>
</div>
