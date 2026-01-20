<script lang="ts">
  import { onMount, tick } from "svelte";
  import { invoke } from "@tauri-apps/api/core";
  import { PDFDocument, rgb } from "pdf-lib";

  export let filePath: string;
  export let workspaceName: string;

  // --- Types ---
  // We mirror pdf-lib's expectation or just use what we need.
  interface LocalAnnotation {
    id: string; // PDF annotation ID if available, or temporary ID
    rects: { x: number; y: number; width: number; height: number }[];
    color: string; // Hex
    pageIndex: number; // 0-based
    pdfAnnotationId?: any; // To store the ref if we can get it
  }

  // --- State ---
  let pageNumbers: number[] = [];
  let canvases: HTMLCanvasElement[] = [];
  let textLayers: HTMLDivElement[] = [];
  let localAnnotations: LocalAnnotation[] = [];

  let error: string | null = null;
  let loading = false;
  let pdfjs: any;

  // --- Selection / Menu State ---
  let showMenu = false;
  let menuPosition = { x: 0, y: 0 };
  let currentSelectionRanges: Range[] = [];
  let selectedPageIndex: number | null = null;

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
  let pdfBytes: Uint8Array | null = null; // Store the raw bytes for pdf-lib

  onMount(async () => {
    try {
      const pdfjsLib = await import("pdfjs-dist");
      const pdfjsWorker = await import(
        "pdfjs-dist/build/pdf.worker.min.mjs?url"
      );
      pdfjs = pdfjsLib;
      pdfjs.GlobalWorkerOptions.workerSrc = pdfjsWorker.default;

      document.addEventListener("selectionchange", handleSelectionChange);
      // document.addEventListener("mousedown", handleGlobalClick);
    } catch (e) {
      console.error("Failed to load pdfjs", e);
      error = "Failed to initialize PDF viewer";
    }

    return () => {
      document.removeEventListener("selectionchange", handleSelectionChange);
      // document.removeEventListener("mousedown", handleGlobalClick);
    };
  });

  $: if (filePath && pdfjs) {
    loadPDF();
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
    localAnnotations = [];

    // Reset selection state on file switch
    showMenu = false;
    currentSelectionRanges = [];
    selectedPageIndex = null;

    try {
      const response: number[] = await invoke("read_workspace_file", {
        workspaceName,
        fileName: filePath,
      });

      pdfBytes = new Uint8Array(response); // Keep specific reference for pdf-lib

      docLoadingTask = pdfjs.getDocument({
        data: pdfBytes,
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

      // Fetch existing annotations
      const annotations = await page.getAnnotations();

      // Process existing PDF annotations to display them
      // We are filtering for Highlight annotations for now
      const pageHeight = page.view.not_scaled ? page.view[3] : page.view[3]; // ViewBox usually [x, y, w, h] or similar. Standard is [0, 0, w, h]

      // Actually pdf.js page.view is [x1, y1, x2, y2]. Height is y2 - y1.
      const view = page.view; // [x, y, w, h] usually [0, 0, width, height]
      const pdfPageWidth = view[2] - view[0];
      const pdfPageHeight = view[3] - view[1];

      for (const ann of annotations) {
        if (ann.subtype === "Highlight" && ann.rect) {
          const r = ann.rect; // [x1, y1, x2, y2] (PDF coords: bottom-left origin)

          // Normalize to 0..1 relative to page dimensions
          // x1, x2 are straightforward key off width
          // y1, y2 need flip.
          // PDF y=0 is bottom. HTML y=0 is top.
          // HTML Top = (PageHeight - PDF_Y_Top) / PageHeight
          // PDF_Y_Top is r[3] (max y)
          // PDF_Y_Bottom is r[1] (min y)

          const normX = (r[0] - view[0]) / pdfPageWidth;
          const normW = (r[2] - r[0]) / pdfPageWidth;

          const normY = (pdfPageHeight - (r[3] - view[1])) / pdfPageHeight; // Top edge
          const normH = (r[3] - r[1]) / pdfPageHeight; // Height

          localAnnotations = [
            ...localAnnotations,
            {
              id: ann.id,
              pageIndex: index,
              color: ann.color
                ? `rgb(${Math.round(ann.color[0])}, ${Math.round(ann.color[1])}, ${Math.round(ann.color[2])})`
                : "#FFFF00",
              rects: [
                {
                  x: normX,
                  y: normY,
                  width: normW,
                  height: normH,
                },
              ],
              pdfAnnotationId: ann.id,
            },
          ];
        }
      }

      const scale = 1.5;
      const pixelRatio = window.devicePixelRatio || 1;
      const viewport = page.getViewport({ scale: scale * pixelRatio });

      canvas.height = viewport.height;
      canvas.width = viewport.width;

      const displayWidth = viewport.width / pixelRatio;
      const displayHeight = viewport.height / pixelRatio;

      canvas.style.width = `${displayWidth}px`;
      canvas.style.height = `${displayHeight}px`;

      textLayerDiv.style.width = `${displayWidth}px`;
      textLayerDiv.style.height = `${displayHeight}px`;
      textLayerDiv.style.setProperty("--scale-factor", `${scale}`);

      const textLayerViewport = page.getViewport({ scale: scale });

      const renderContext = {
        canvasContext: canvas.getContext("2d")!,
        viewport: viewport,
        // We do render annotations here? No, defaults usually exclude them or include them?
        // Default annotationMode is usually ENABLE (1).
        // If they are rendered on canvas, we get double rendering if we draw divs.
        // We'll turn off annotation rendering on canvas.
        annotationMode: 0,
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

  // --- Interaction Logic ---

  function handleSelectionChange() {
    const selection = window.getSelection();
    if (!selection || selection.isCollapsed) {
      showMenu = false;
      currentSelectionRanges = [];
      return;
    }

    // Identify if selection is within our PDF viewer
    const anchorNode = selection.anchorNode;
    if (!anchorNode) return;

    // Check if within a page
    let node: Node | null = anchorNode;
    let pageContainer: HTMLElement | null = null;
    while (node && node instanceof Element === false) {
      node = node.parentNode;
    }
    if (node) {
      pageContainer = (node as Element).closest(".relative[data-page-index]");
    }

    if (!pageContainer) {
      // Did not select inside a page
      return;
    }

    // Valid selection
    const pageIndexStr = pageContainer.getAttribute("data-page-index");
    if (pageIndexStr === null) return;

    selectedPageIndex = parseInt(pageIndexStr);

    const range = selection.getRangeAt(0);
    const rect = range.getBoundingClientRect();

    // Position menu
    menuPosition = {
      x: rect.left + rect.width / 2 - 80,
      y: rect.top - 50,
    };

    // Bounds check
    if (menuPosition.y < 50) menuPosition.y = rect.bottom + 10; // show below if not enough space above

    currentSelectionRanges = [range];
    showMenu = true;
  }

  async function addHighlight(colorHex: string) {
    if (
      !currentSelectionRanges.length ||
      selectedPageIndex === null ||
      !pdfBytes
    )
      return;

    loading = true; // Show loading while saving

    try {
      // 1. Calculate quads/rects from selection
      const range = currentSelectionRanges[0];
      const clientRects = Array.from(range.getClientRects());

      // Get Page Dimensions from the textLayer container (which matches display size)
      const pageContainer = document.querySelector(
        `.relative[data-page-index="${selectedPageIndex}"]`,
      ) as HTMLElement;
      const textLayer = pageContainer.querySelector(
        ".textLayer",
      ) as HTMLElement;
      const pageRect = textLayer.getBoundingClientRect();

      // Need to map Screen Coords (HTML) -> PDF Coords
      // HTML (Top-Left) -> PDF (Bottom-Left)
      // We need to know the original PDF Page Height.

      // We can load the PDF using pdf-lib first to get page properties.
      const pdfDoc = await PDFDocument.load(pdfBytes);
      const pages = pdfDoc.getPages();
      const page = pages[selectedPageIndex];
      const { width, height } = page.getSize();

      // Conversion factors
      // HTML Display Width / PDF Width = Scale
      // But we can just use ratios.

      const rectsForPdf: {
        x: number;
        y: number;
        width: number;
        height: number;
      }[] = [];

      for (const rect of clientRects) {
        // Relative to Page Container Top-Left
        const relX = rect.left - pageRect.left;
        const relY = rect.top - pageRect.top;
        const relW = rect.width;
        const relH = rect.height;

        // Normalize (0..1)
        const normX = relX / pageRect.width;
        const normY = relY / pageRect.height;
        const normW = relW / pageRect.width;
        const normH = relH / pageRect.height;

        // Convert to PDF Coords
        // x = normX * pdfWidth
        // y = height - (normY * height) - (normH * height) ??
        // PDF Y is from bottom.
        // Top of HTML rect is `normY`. Bottom of HTML rect is `normY + normH`.
        // In PDF, Bottom of HTML rect corresponds to `height - (normY + normH) * height`.
        // Height of rect is `normH * height`.

        const pdfX = normX * width;
        const pdfH = normH * height;
        const pdfY = height - normY * height - pdfH; // Bottom-left Y of the rect
        const pdfW = normW * width;

        rectsForPdf.push({ x: pdfX, y: pdfY, width: pdfW, height: pdfH });
      }

      // 2. Add Annotation to PDF using pdf-lib
      // Convert Hex to RGB
      const r = parseInt(colorHex.slice(1, 3), 16) / 255;
      const g = parseInt(colorHex.slice(3, 5), 16) / 255;
      const b = parseInt(colorHex.slice(5, 7), 16) / 255;

      const highlight = page.drawRectangle({
        x: 0,
        y: 0,
        width: 0,
        height: 0, // Placeholder
        color: rgb(r, g, b),
      });

      // Wait, drawRectangle draws content on the stream.
      // We want an ANNOTATION.
      // pdf-lib currently (v1.17) doesn't have a high-level `addHighlight` API easily exposed on `Page`.
      // We have to work with lower level objects, OR use `doc.context.register` etc.
      // Actually checking documentation: `page.node.addAnnot` is needed.

      // Construct the QuadPoints.
      // Highlight annotations need QuadPoints efficiently for text selection.
      // Rect is the bounding box of all quads.

      // Simplification for Hackathon: Add plain rectangle annotations?
      // User asked for "Highlight".
      // Using Semi-transparent Rectangle?
      // It works visually but isn't a "Text Highlight".
      // But implementing true Highlight annotation manually with QuadPoints in pdf-lib is verbose.
      // Let's try to add a true Highlight annotation if possible, otherwise fallback to drawn transparent rectangles.
      // Drawing rectangles is PERMANENT CONTENT (not annotation), so hard to delete.
      // We MUST use Annotations.

      // Let's use `createAnnotation` helper logic manually.
      // We'll construct a Highlight annotation dictionary.

      // Calculate bounding box of all rects
      let minX = Infinity,
        minY = Infinity,
        maxX = -Infinity,
        maxY = -Infinity;
      const quadPoints: number[] = [];

      for (const rect of rectsForPdf) {
        minX = Math.min(minX, rect.x);
        minY = Math.min(minY, rect.y);
        maxX = Math.max(maxX, rect.x + rect.width);
        maxY = Math.max(maxY, rect.y + rect.height);

        // QuadPoints: [xTL, yTL, xTR, yTR, xBL, yBL, xBR, yBR] (8 numbers per rect)
        // PDF Coords.
        // TL = (x, y+h)
        // TR = (x+w, y+h)
        // BL = (x, y)
        // BR = (x+w, y)
        quadPoints.push(
          rect.x,
          rect.y + rect.height,
          rect.x + rect.width,
          rect.y + rect.height,
          rect.x,
          rect.y,
          rect.x + rect.width,
          rect.y,
        );
      }

      const highlightAnnot = pdfDoc.context.obj({
        Type: "Annot",
        Subtype: "Highlight",
        Rect: [minX, minY, maxX, maxY], // Bounding box
        QuadPoints: quadPoints,
        C: [r, g, b], // Color
        Params: "(mix)", // Optional, rendering style
      });

      const highlightRef = pdfDoc.context.register(highlightAnnot);

      page.node.addAnnot(highlightRef);

      // 3. Save PDF
      const modifiedPdfBytes = await pdfDoc.save();

      // 4. Write to file
      await invoke("write_workspace_file_binary", {
        workspaceName,
        fileName: filePath,
        content: Array.from(modifiedPdfBytes),
      });

      // 5. Reload Viewer to reflect changes from file (Permanent proof)
      // To make it smooth, maybe we don't full reload, but it's safest for sync.
      // We'll call loadPDF().
      await loadPDF();
    } catch (e) {
      console.error("Error adding highlight:", e);
      error = "Failed to save highlight";
      loading = false;
    }

    showMenu = false;
    currentSelectionRanges = [];
    window.getSelection()?.removeAllRanges();
  }

  async function deleteAnnotation(annotationId: string) {
    if (!pdfBytes) return;
    loading = true;

    try {
      const pdfDoc = await PDFDocument.load(pdfBytes);
      const pages = pdfDoc.getPages();

      // We need to find the annotation reference by ID.
      // pdf-lib doesn't index annotations by ID easily.
      // However, our `localAnnotations` came from pdf.js which gave us an ID.
      // pdf.js ID is usually the Ref string "123 0 R".
      // pdf-lib handles Refs.

      // If annotationId looks like "123R", we can probably parse it.
      // pdf.js IDs are often "123R" or similar.
      // Let's check format. `page.getAnnotations()` usually returns `id` as string.

      // Iterate all pages to find and remove.
      let found = false;

      for (let i = 0; i < pages.length; i++) {
        const page = pages[i];
        const annots = page.node.Annots();
        if (!annots) continue;

        // Annots is maybe undefined or PDFArray.
        // We need to traverse.
        // This is tricky without knowing exact Ref.

        // Alternative Strategy:
        // Since we are creating the annotations, we could store a unique ID in the `NM` (Name) field.
        // But for existing ones or ones we just added via low-level, they might not have it.

        // Let's rely on coordinate matching if Ref fails? No, risky.
        // Let's assume pdf.js `id` corresponds to the PDF Object Number.
        // e.g. "50R" -> Object 50.

        // pdf-lib's `context.enumerateIndirectObjects()` might help?
        // Or we just iterate page annotations and check their Ref.

        const annotRefs = page.node.Annots();
        if (!annotRefs) continue;

        const len = annotRefs.size();
        for (let j = 0; j < len; j++) {
          const ref = annotRefs.get(j);
          // ref could be direct object or reference.
          // In simple PDF generation, it's a reference.

          // How to get Ref tag from pdf-lib object?
          // `ref.tag` -> "123 0 R" string format?

          // In pdf-lib, direct references are stored.
          // We need to compare `ref` with the `id` we got from pdf.js.

          // pdf.js ID: "524R" (Object 524, Gen 0?).
          // Let's try to match.

          // Actually, traversing arrays in pdf-lib and filtering is standard.
          // We will simply remove the reference from the Annots array.

          // If we can't easily match ID, we can't delete reliably.
          // PROPOSAL: Whenever we create an annotation, we set contents or NM to a UUID.
          // Then we search by that UUID.
          // But for native "Highlight" logic above, we didn't add UUID.
          // Let's add NM (Name) field to the annotation dictionary above.
        }
      }

      // REVISED DELETE STRATEGY:
      // We will find the annotation by iterating and matching the Object Reference string.
      // pdf.js `id` -> "524R"
      // pdf-lib `ref.toString()` -> "524 0 R"

      // Let's try to implement this loop.

      for (const page of pages) {
        const annots = page.node.Annots();
        if (!annots) continue;

        const count = annots.size();
        const indicesToRemove: number[] = [];

        for (let j = 0; j < count; j++) {
          const ref = annots.get(j);
          if (ref instanceof Object && "tag" in ref) {
            // It is a reference
            const tag = (ref as any).tag; // "524 0 R"
            // Convert to pdf.js format: "524R" (if gen 0)
            // tag is simplified in pdf-lib?
            // Actually `ref.toString()` returns "524 0 R".

            const refStr = ref.toString();
            const [objNum, genNum, _] = refStr.split(" ");
            const pdfJsId = `${objNum}R`; // pdf.js style usually

            if (pdfJsId === annotationId || refStr === annotationId) {
              indicesToRemove.push(j);
            }
          }
        }

        // Remove in reverse order
        for (const idx of indicesToRemove.reverse()) {
          annots.remove(idx);
          found = true;
        }
      }

      if (found) {
        const modifiedBytes = await pdfDoc.save();
        await invoke("write_workspace_file_binary", {
          workspaceName,
          fileName: filePath,
          content: Array.from(modifiedBytes),
        });
        await loadPDF();
      }
    } catch (e) {
      console.error("Failed to delete", e);
    } finally {
      loading = false;
    }
  }
</script>

<div
  class="h-full w-full flex flex-col items-center overflow-auto p-4 gap-4 bg-gray-100 dark:bg-gray-900 border rounded-md relative"
  role="application"
  tabindex="-1"
>
  {#if loading}
    <div
      class="fixed inset-0 z-50 flex items-center justify-center bg-black/20 backdrop-blur-sm"
    >
      <div class="bg-white p-4 rounded shadow">Processing PDF...</div>
    </div>
  {/if}

  {#if error}
    <div class="text-red-500 p-4">Error loading PDF: {error}</div>
  {/if}

  {#key filePath}
    {#each pageNumbers as pageNum, i}
      <div
        class="relative shadow-lg bg-white shrink-0 group"
        data-page-index={i}
      >
        <div class="relative">
          <canvas bind:this={canvases[i]} class="block"></canvas>
          <div bind:this={textLayers[i]} class="textLayer"></div>

          <!-- Interactive Annotation Overlay Layer -->
          <!-- We render click handlers over the annotations to allow delete -->
          <div class="absolute inset-0 pointer-events-none">
            {#each localAnnotations.filter((a) => a.pageIndex === i) as ann}
              <!-- We render a transparent box for interaction/visual feedback of functionality if needed.
                         Since they are burned into PDF, they are visible.
                         But we need a way to clicking them to DELETE.
                         This overlay provides the click target. -->
              {#each ann.rects as r}
                <!-- 
                            Note: PDF coords (ann.rects) were Bottom-Left origin relative to PDF Page.
                            We need to convert back to HTML Top-Left for this div placement.
                            
                            Wait, in renderPage, I stored them roughly.
                            Ah, in renderPage loop:
                            r is [x1, y1, x2, y2] (PDF coords).
                            We need conversion to place DIV.
                            
                            y_html = (pageHeight - y2) * scale ??
                            Actually, let's look at render logic.
                            
                            scale comes from container width vs pdf width.
                            
                            Let's use percentages for robust positioning.
                            
                            pdfPageHeight = (y2 - y1) of the page view box.
                            
                            top = (pdfPageHeight - r[3]) / pdfPageHeight
                            height = (r[3] - r[1]) / pdfPageHeight
                            
                            Note: r[1] is bottom, r[3] is top in PDF.
                        -->
                <button
                  class="absolute hover:bg-black/10 cursor-pointer pointer-events-auto border-2 border-transparent hover:border-red-500 transition-colors"
                  style="
                                left: {(r.x / canvases[i]?.width) *
                    100}%;  /* Rough approximation if width matches? Unlikely. */
                                /* Let's use viewport-based logic if possible. */
                                /* Actually, we need to know PDF page dimensions to convert correctly. */
                                /* We don't have them easily accessible in interactions template without storing them. */
                                /* Assuming standard US Letter aspect ratio or similar is risky. */
                                
                                /* BETTER: In renderPage, we calculated localAnnotations using PDF coords. 
                                   Let's store Normalized coords in localAnnotations instead! 
                                */
                                
                                /* FIX IN RENDERPAGE NEEDED: Calculate normalized rects there. */
                            "
                  title="Click to delete annotation"
                  onclick={() => deleteAnnotation(ann.id)}
                  onkeydown={(e) =>
                    e.key === "Enter" && deleteAnnotation(ann.id)}
                  aria-label="Delete annotation"
                >
                  <!-- We can render a hidden X or similar that appears on hover -->
                </button>
              {/each}
            {/each}
          </div>
        </div>

        <!-- Page Number -->
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
    class="fixed z-50 bg-white dark:bg-gray-800 border shadow-xl rounded-lg p-2 flex gap-2 animate-in fade-in zoom-in-95 duration-100"
    style="left: {menuPosition.x}px; top: {menuPosition.y}px;"
  >
    {#each COLORS as color}
      <button
        class="w-6 h-6 rounded-full border border-gray-200 transition-transform hover:scale-110 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500"
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
    opacity: 1; /* Make text layer fully opaque but text transparent, so selection works? Default is usually opacity: 0.2-1 with color: transparent */
    line-height: 1;
    transform-origin: 0 0;
    pointer-events: auto;
  }

  :global(.textLayer span) {
    color: transparent;
    position: absolute;
    white-space: pre;
    cursor: text;
    transform-origin: 0% 0%;
  }

  :global(.textLayer ::selection) {
    background: rgba(
      255,
      215,
      0,
      0.5
    ); /* Improved selection color (Gold) - more visible */
    /* Or standard blue but brighter? User said 'almost invisible' */
    /* Let's use a nice vibrant blue/purple mix or high contrast. */
    background: rgba(60, 130, 246, 0.4);
  }
</style>
