<script lang="ts">
  import { invoke } from "@tauri-apps/api/core";
  import { ScrollArea } from "$lib/components/ui/scroll-area/index.js";

  export let filePath: string;
  export let workspaceName: string;

  let content = "";
  let error: string | null = null;
  let loading = false;

  $: if (filePath) {
    loadContent();
  }

  async function loadContent() {
    loading = true;
    error = null;
    content = "";
    try {
      const response: number[] = await invoke("read_workspace_file", {
        workspaceName,
        fileName: filePath,
      });

      const uint8Array = new Uint8Array(response);
      content = new TextDecoder().decode(uint8Array);
    } catch (e: any) {
      error = e.message || e;
    } finally {
      loading = false;
    }
  }
</script>

<div
  class="h-full w-full p-4 font-mono text-sm overflow-auto whitespace-pre-wrap bg-background text-foreground border rounded-md"
>
  {#if loading}
    <div class="flex items-center justify-center h-full">Loading...</div>
  {:else if error}
    <div class="text-red-500">Error: {error}</div>
  {:else}
    {content}
  {/if}
</div>
