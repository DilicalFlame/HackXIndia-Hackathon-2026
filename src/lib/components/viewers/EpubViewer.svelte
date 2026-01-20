<script lang="ts">
  import { onMount } from "svelte";
  import ePub from "epubjs";
  import { invoke } from "@tauri-apps/api/core";

  export let filePath: string;
  export let workspaceName: string;

  let viewerContainer: HTMLDivElement;
  let book: any;
  let rendition: any;
  let error: string | null = null;

  $: if (filePath && viewerContainer) {
    loadEpub();
  }

  async function loadEpub() {
    if (book) {
      book.destroy();
    }
    error = null;
    try {
      const response: number[] = await invoke("read_workspace_file", {
        workspaceName,
        fileName: filePath,
      });
      const buffer = new Uint8Array(response).buffer;

      book = ePub(buffer);
      rendition = book.renderTo(viewerContainer, {
        width: "100%",
        height: "100%",
      });
      await rendition.display();
    } catch (e: any) {
      error = e.message || String(e);
      console.error("Failed to load epub", e);
    }
  }
</script>

<div
  class="h-full w-full flex flex-col relative bg-background border rounded-md overflow-hidden"
>
  {#if error}
    <div
      class="absolute inset-0 flex items-center justify-center text-red-500 z-10 bg-background/90"
    >
      Error: {error}
    </div>
  {/if}
  <div bind:this={viewerContainer} class="w-full h-full"></div>
</div>
