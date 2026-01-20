<script lang="ts">
  import { onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";

  export let filePath: string;
  export let workspaceName: string;

  // --- Types ---
  interface NormalizedRect {
    left: number;
    top: number;
    right: number;
    bottom: number;
  }

  interface Annotation {
    id: string;
    annotation_type: number; // 4 = Highlight
    author: string;
    contents: string;
    unique_name: string;
    creation_date: string;
    modification_date: string;
    flags: number;
    bounding_rect: NormalizedRect;
    color: string;
    opacity: number;
    page_num: number;
  }

  // --- State ---
  let pageNumbers: number[] = [];
  let canvases: HTMLCanvasElement[] = [];
  let textLayers: HTMLDivElement[] = [];
  let annotations: Annotation[] = [];

  let error: string | null = null;
  let loading = false;
  let pdfjs: any;

  // --- Selection / Menu State ---
  let showMenu = false;
  let menuPosition = { x: 0, y: 0 };
  let currentSelection: Selection | null = null;
  let selectedPageNum: number | null = null;

  // Color palette
  const COLORS = [
    "#FFFF00", // Yellow
    "#00FF00", // Green
    "#00FFFF", // Cyan
    "#FF00FF", // Magenta
    "#FF0000", // Red
  ];

  // --- Tracking ---
  let docLoadingTask: any = null;
  let activeRenderTasks: Set<any> = new Set();

  onMount(async () => {
    try {
      const pdfjsLib = await import("pdfjs-dist");
      const pdfjsWorker = await import(
        "pdfjs-dist/build/pdf.worker.min.mjs?url"
      );
      pdfjs = pdfjsLib;
      pdfjs.GlobalWorkerOptions.workerSrc = pdfjsWorker.default;
    } catch (e) {
      console.error("Failed to load pdfjs", e);
      error = "Failed to initialize PDF viewer";
    }
  });

  $: if (filePath && pdfjs) {
    loadPDF();
    loadAnnotations();
  }

  async function cancelAllTasks() {
    if (docLoadingTask) {
      try {
        await docLoadingTask.destroy();
      } catch (e) {}
      docLoadingTask = null;
    }
    for (const task of activeRenderTasks) {
      try {
        task.cancel();
      } catch (e) {}
    }
    activeRenderTasks.clear();
  }

  async function loadPDF() {
    if (!pdfjs) return;

    await cancelAllTasks();

    loading = true;
    error = null;
    pageNumbers = [];
    canvases = [];
    textLayers = [];

    // Reset selection state on file switch
    showMenu = false;
    currentSelection = null;

    try {
      const response: number[] = await invoke("read_workspace_file", {
        workspaceName,
        fileName: filePath,
      });

      const data = new Uint8Array(response);

      docLoadingTask = pdfjs.getDocument({
        data,
        cMapUrl: "https://unpkg.com/pdfjs-dist@5.4.530/cmaps/",
        cMapPacked: true,
      });
      const pdf = await docLoadingTask.promise;

      pageNumbers = Array.from({ length: pdf.numPages }, (_, i) => i + 1);

      await tick();

      const currentTask = docLoadingTask;
      for (let i = 0; i < pageNumbers.length; i++) {
        if (docLoadingTask !== currentTask) break;
        await renderPage(pdf, i + 1, i);
      }
    } catch (e: any) {
      if (
        e?.name === "RenderingCancelledException" ||
        e?.message?.includes("cancelled")
      ) {
        return;
      }
      error = e.message || e;
      console.error(e);
    } finally {
      if (docLoadingTask) {
        loading = false;
      }
    }
  }

  async function renderPage(pdf: any, pageNum: number, index: number) {
    const canvas = canvases[index];
    const textLayerDiv = textLayers[index];
    if (!canvas || !textLayerDiv) return;

    try {
      const page = await pdf.getPage(pageNum);

      const scale = 1.5;
      const pixelRatio = window.devicePixelRatio || 1;
      const viewport = page.getViewport({ scale: scale * pixelRatio });

      canvas.height = viewport.height;
      canvas.width = viewport.width;

      // CSS scale to match desired display size
      const displayWidth = viewport.width / pixelRatio;
      const displayHeight = viewport.height / pixelRatio;

      canvas.style.width = `${displayWidth}px`;
      canvas.style.height = `${displayHeight}px`;

      textLayerDiv.style.width = `${displayWidth}px`;
      textLayerDiv.style.height = `${displayHeight}px`;

      // Ensure text layer matches the CSS dimensions 1:1
      textLayerDiv.style.setProperty("--scale-factor", `${scale}`);

      const textLayerViewport = page.getViewport({ scale: scale });

      const renderContext = {
        canvasContext: canvas.getContext("2d")!,
        viewport: viewport,
      };

      const task = page.render(renderContext);
      activeRenderTasks.add(task);

      await task.promise;
      activeRenderTasks.delete(task);

      const textContent = await page.getTextContent();
      textLayerDiv.innerHTML = "";

      const textLayer = new pdfjs.TextLayer({
        textContentSource: textContent,
        container: textLayerDiv,
        viewport: textLayerViewport,
      });

      await textLayer.render();
    } catch (e: any) {
      if (e?.name === "RenderingCancelledException") return;
      console.error(`Error rendering page ${pageNum}:`, e);
    }
  }

  // --- Annotation Logic ---

  async function loadAnnotations() {
    try {
      const doc: any = await invoke("load_annotations", {
        workspaceName,
        fileName: filePath,
      });

      const flat: Annotation[] = [];
      for (const [pageNumStr, page] of Object.entries(doc.pages || {})) {
        const pNum = parseInt(pageNumStr);
        if ((page as any).annotations) {
          for (const ann of (page as any).annotations) {
            flat.push({ ...ann, page_num: pNum });
          }
        }
      }
      annotations = flat;
    } catch (e) {
      console.error("Failed to load annotations", e);
    }
  }

  function handleSelection(event: MouseEvent) {
    const selection = window.getSelection();
    if (!selection || selection.isCollapsed) {
      showMenu = false;
      return;
    }

    // We need to find which page the selection is on.
    // The anchorNode might be deep inside the textLayer spans.
    const anchorNode = selection.anchorNode;
    if (!anchorNode) return;

    // Find the closest page container
    // textLayer is a child of the .relative container which has data-page-index
    let node: Node | null = anchorNode;
    let pageContainer: HTMLElement | null = null;

    while (node && node instanceof Element === false) {
      node = node.parentNode;
    }

    if (node) {
      pageContainer = (node as Element).closest(".relative[data-page-index]");
    }

    if (!pageContainer) {
      showMenu = false;
      return;
    }

    const pageIndexAttr = pageContainer.getAttribute("data-page-index");
    if (pageIndexAttr === null) return;

    selectedPageNum = pageNumbers[parseInt(pageIndexAttr)];

    if (selection.rangeCount > 0) {
      const range = selection.getRangeAt(0);
      const rect = range.getBoundingClientRect();

      // Ensure menu is within viewport
      // Add offset to avoid covering text
      menuPosition = {
        x: rect.left + rect.width / 2 - 100, // center menu roughly
        y: rect.top - 50,
      };

      // Keep menu in bounds
      if (menuPosition.x < 10) menuPosition.x = 10;
      if (menuPosition.y < 10) menuPosition.y = 10;

      currentSelection = selection;
      showMenu = true;
    }
  }

  async function addHighlight(color: string) {
    if (!currentSelection || selectedPageNum === null) return;

    const range = currentSelection.getRangeAt(0);
    // Use getBoundingClientRect for the whole range for now.
    // Ideally we should use getClientRects() to support multi-line highlights better,
    // but the backend only supports a single bounding_rect per annotation.
    // To support multi-line, we would need to generate multiple annotations.

    const clientRects = Array.from(range.getClientRects());

    // Find the page container again to calculate relative coordinates
    const anchorNode = currentSelection.anchorNode;
    let node: Node | null = anchorNode;
    while (node && !(node instanceof Element)) {
      node = node.parentNode;
    }
    const pageContainer = (node as Element)?.closest(
      ".relative[data-page-index]",
    ) as HTMLElement;

    if (!pageContainer) return;

    const pageRect = pageContainer.getBoundingClientRect();

    // We will create one annotation per client rect (one per line usually)
    // giving them the same ID would be ideal if we had grouping, but we'll just give unique IDs.

    const newAnnotations: Annotation[] = [];
    const groupId = crypto.randomUUID(); // could be used for grouping later
    const now = new Date().toISOString();

    for (const rect of clientRects) {
      // Calculate normalized coordinates relative to the page
      const normalizedRect: NormalizedRect = {
        left: (rect.left - pageRect.left) / pageRect.width,
        top: (rect.top - pageRect.top) / pageRect.height,
        right: (rect.right - pageRect.left) / pageRect.width,
        bottom: (rect.bottom - pageRect.top) / pageRect.height,
      };

      // Skip invalid/empty rects
      if (
        normalizedRect.left >= normalizedRect.right ||
        normalizedRect.top >= normalizedRect.bottom
      )
        continue;

      newAnnotations.push({
        id: crypto.randomUUID(),
        annotation_type: 4,
        author: "User",
        contents: currentSelection.toString(),
        unique_name: `ann-${Date.now()}-${Math.random()}`,
        creation_date: now,
        modification_date: now,
        flags: 0,
        bounding_rect: normalizedRect,
        color: color,
        opacity: 0.4,
        page_num: selectedPageNum,
      });
    }

    annotations = [...annotations, ...newAnnotations];
    await saveAnnotationsBackend();

    currentSelection.removeAllRanges();
    showMenu = false;
  }

  async function saveAnnotationsBackend() {
    const pagesMap: Record<number, any> = {};

    for (const ann of annotations) {
      if (!pagesMap[ann.page_num]) {
        pagesMap[ann.page_num] = { width: 0, height: 0, annotations: [] };
      }
      pagesMap[ann.page_num].annotations.push(ann);
    }

    const document = {
      pages: pagesMap,
      metadata: {},
    };

    try {
      await invoke("save_annotations", {
        workspaceName,
        file_name: filePath,
        document,
      });
    } catch (e) {
      console.error("Failed to save annotations", e);
    }
  }
</script>

<div
  class="h-full w-full flex flex-col items-center overflow-auto p-4 gap-4 bg-gray-100 dark:bg-gray-900 border rounded-md relative"
  onmouseup={handleSelection}
  role="application"
  tabindex="-1"
>
  {#if loading && pageNumbers.length === 0}
    <div class="text-muted-foreground p-4">Loading PDF...</div>
  {:else if error}
    <div class="text-red-500 p-4">Error loading PDF: {error}</div>
  {/if}

  {#key filePath}
    {#each pageNumbers as pageNum, i}
      <!-- Page Container -->
      <div class="relative shadow-lg bg-white shrink-0" data-page-index={i}>
        <div class="relative">
          <canvas bind:this={canvases[i]} class="block"></canvas>
          <div bind:this={textLayers[i]} class="textLayer"></div>

          <!-- Annotations Layer -->
          <div class="annotation-layer">
            {#each annotations.filter((a) => a.page_num === pageNum) as ann (ann.id)}
              <div
                class="highlight-annotation"
                style="
                            left: {ann.bounding_rect.left * 100}%;
                            top: {ann.bounding_rect.top * 100}%;
                            width: {(ann.bounding_rect.right -
                  ann.bounding_rect.left) *
                  100}%;
                            height: {(ann.bounding_rect.bottom -
                  ann.bounding_rect.top) *
                  100}%;
                            background-color: {ann.color};
                            opacity: {ann.opacity};
                        "
                title={ann.contents}
              ></div>
            {/each}
          </div>
        </div>

        <div
          class="absolute bottom-2 right-2 bg-black/50 text-white text-xs px-2 py-1 rounded pointer-events-none z-10"
        >
          Page {pageNum}
        </div>
      </div>
    {/each}
  {/key}
</div>

{#if showMenu}
  <div
    class="fixed z-50 bg-white dark:bg-gray-800 border shadow-xl rounded-lg p-2 flex gap-2"
    style="left: {menuPosition.x}px; top: {menuPosition.y}px;"
  >
    {#each COLORS as color}
      <button
        class="w-6 h-6 rounded-full border border-gray-200 transition-transform hover:scale-110"
        style="background-color: {color};"
        onclick={() => addHighlight(color)}
        aria-label="Highlight with {color}"
      ></button>
    {/each}
  </div>
{/if}

<style>
  :global(.textLayer) {
    position: absolute;
    text-align: initial;
    left: 0;
    top: 0;
    right: 0;
    bottom: 0;
    overflow: hidden;
    opacity: 0.2;
    line-height: 1;
    transform-origin: 0 0;
    pointer-events: auto; /* Required for text selection */
  }

  :global(.textLayer span) {
    color: transparent;
    position: absolute;
    white-space: pre;
    cursor: text;
    transform-origin: 0% 0%;
  }

  :global(.textLayer ::selection) {
    background: rgba(0, 0, 255, 0.2);
  }

  .annotation-layer {
    position: absolute;
    top: 0;
    left: 0;
    right: 0;
    bottom: 0;
    pointer-events: none; /* Allows clicks to pass through to text layer */
    z-index: 10;
  }

  .highlight-annotation {
    position: absolute;
    mix-blend-mode: multiply;
    pointer-events: auto; /* Allows interaction with highlights if needed */
  }
</style>
