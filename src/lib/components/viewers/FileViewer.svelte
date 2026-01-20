<script lang="ts">
  import PDFViewer from "./PDFViewer.svelte";
  import EpubViewer from "./EpubViewer.svelte";
  import TextViewer from "./TextViewer.svelte";

  export let filePath: string;
  export let workspaceName: string;

  let fileType: "pdf" | "epub" | "text" | "unknown" = "unknown";

  $: {
    if (filePath) {
      const lower = filePath.toLowerCase();
      if (lower.endsWith(".pdf")) fileType = "pdf";
      else if (lower.endsWith(".epub")) fileType = "epub";
      else if (
        lower.endsWith(".txt") ||
        lower.endsWith(".md") ||
        lower.endsWith(".rs") ||
        lower.endsWith(".ts") ||
        lower.endsWith(".js") ||
        lower.endsWith(".json") ||
        lower.endsWith(".css") ||
        lower.endsWith(".html") ||
        lower.endsWith(".svelte") ||
        lower.endsWith(".toml")
      )
        fileType = "text";
      else fileType = "unknown";
    }
  }
</script>

<div class="w-full h-full flex flex-col">
  {#if fileType === "pdf"}
    <PDFViewer {filePath} {workspaceName} />
  {:else if fileType === "epub"}
    <EpubViewer {filePath} {workspaceName} />
  {:else if fileType === "text"}
    <TextViewer {filePath} {workspaceName} />
  {:else}
    <div class="flex items-center justify-center h-full text-muted-foreground">
      Unsupported file type for preview: {filePath}
    </div>
  {/if}
</div>
