<script lang="ts">
  import { page } from "$app/stores";
  import { onMount } from "svelte";
  import { workspaces } from "$lib/stores/workspace";
  import * as Resizable from "$lib/components/ui/resizable/index.js";
  import * as Tabs from "$lib/components/ui/tabs/index.js"; // Can remove if unused
  import * as ScrollArea from "$lib/components/ui/scroll-area/index.js";
  import { Button, buttonVariants } from "$lib/components/ui/button/index.js";
  import { Separator } from "$lib/components/ui/separator/index.js";
  import * as Tooltip from "$lib/components/ui/tooltip/index.js";
  import { File, Plus, MessageSquare, Files, ArrowLeft } from "@lucide/svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { open } from "@tauri-apps/plugin-dialog";
  import { goto } from "$app/navigation";
  import FileViewer from "$lib/components/viewers/FileViewer.svelte";
  import { cn } from "$lib/utils";

  // Ensure workspaceName is string
  const workspaceName = $page.params.name ?? "";

  let files: string[] = [];
  let isTauri = false;
  let activeView: "chat" | "files" = "files";
  let activeFile: string | null = null;

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
                  class="flex items-center gap-2 rounded-md p-2 hover:bg-muted cursor-pointer text-sm {activeFile ===
                  file
                    ? 'bg-muted'
                    : ''}"
                  role="button"
                  tabindex="0"
                  onclick={() => {
                    activeFile = file;
                    activeView = "files";
                  }}
                  onkeydown={(e) => {
                    if (e.key === "Enter" || e.key === " ") {
                      activeFile = file;
                      activeView = "files";
                    }
                  }}
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
      <div class="flex h-full">
        <!-- Vertical Icon Tab Rail -->
        <div
          class="flex w-[50px] flex-col gap-2 border-r bg-muted/20 py-4 items-center"
        >
          <Tooltip.Provider>
            <Tooltip.Root>
              <Tooltip.Trigger
                class={cn(
                  buttonVariants({
                    variant: activeView === "chat" ? "secondary" : "ghost",
                    size: "icon",
                  }),
                  "rounded-md h-9 w-9",
                )}
                onclick={() => (activeView = "chat")}
              >
                <MessageSquare class="h-4 w-4" />
              </Tooltip.Trigger>
              <Tooltip.Content side="right">Chat</Tooltip.Content>
            </Tooltip.Root>

            <Tooltip.Root>
              <Tooltip.Trigger
                class={cn(
                  buttonVariants({
                    variant: activeView === "files" ? "secondary" : "ghost",
                    size: "icon",
                  }),
                  "rounded-md h-9 w-9",
                )}
                onclick={() => (activeView = "files")}
              >
                <Files class="h-4 w-4" />
              </Tooltip.Trigger>
              <Tooltip.Content side="right">Opened Files</Tooltip.Content>
            </Tooltip.Root>
          </Tooltip.Provider>
        </div>

        <!-- Content Area -->
        <div class="flex-1 overflow-hidden h-full">
          {#if activeView === "chat"}
            <div class="h-full flex flex-col p-4 w-full">
              <h3 class="font-semibold text-lg mb-4">Chat</h3>
              <div
                class="flex-1 border rounded-md p-4 text-muted-foreground flex items-center justify-center"
              >
                Chat functionality coming soon...
              </div>
            </div>
          {:else if activeView === "files"}
            <div class="h-full flex flex-col w-full">
              {#if activeFile}
                <div class="h-full w-full">
                  <!-- Integrated File Viewer will go here -->
                  <FileViewer filePath={activeFile} {workspaceName} />
                </div>
              {:else}
                <div
                  class="flex-1 flex flex-col items-center justify-center text-muted-foreground"
                >
                  <Files class="h-10 w-10 mb-4 opacity-50" />
                  <p>Select a file from the sidebar to view it.</p>
                </div>
              {/if}
            </div>
          {/if}
        </div>
      </div>
    </Resizable.Pane>
  </Resizable.PaneGroup>
</div>
