<script lang="ts">
  import { onMount } from "svelte";
  import { workspaces } from "$lib/stores/workspace";
  import CreateWorkspaceModal from "$lib/components/CreateWorkspaceModal.svelte";
  import ModeToggle from "$lib/components/ModeToggle.svelte";
  import { Folder, Plus } from "@lucide/svelte";
  import { Button } from "$lib/components/ui/button/index.js";
  import * as Card from "$lib/components/ui/card/index.js";

  let isModalOpen = false;

  onMount(() => {
    workspaces.loadWorkspaces();
  });

  function openModal() {
    isModalOpen = true;
  }

  function closeModal() {
    isModalOpen = false;
  }
</script>

<div class="min-h-screen bg-background p-8 font-sans text-foreground relative">
  <div class="max-w-6xl mx-auto">
    <header class="mb-8 flex items-center justify-between border-b pb-6">
      <div>
        <h1 class="text-3xl font-bold tracking-tight text-foreground">
          Workspaces
        </h1>
      </div>

      <Button onclick={openModal}>
        <Plus class="mr-2 h-4 w-4" />
        New Workspace
      </Button>
    </header>

    <div
      class="grid grid-cols-1 gap-6 sm:grid-cols-2 lg:grid-cols-3 xl:grid-cols-4"
    >
      {#each $workspaces as workspace}
        <!-- svelte-ignore a11y_click_events_have_key_events -->
        <!-- svelte-ignore a11y_no_static_element_interactions -->
        <div
          onclick={() => {
            /* Navigate or open workspace */
          }}
        >
          <Card.Root
            class="group cursor-pointer transition-colors hover:bg-muted/50 h-full"
          >
            <Card.Header>
              <div class="flex items-center gap-4">
                <div
                  class="flex h-10 w-10 items-center justify-center rounded-lg bg-muted text-foreground group-hover:bg-background transition-colors border"
                >
                  <Folder class="h-5 w-5" />
                </div>
                <div>
                  <Card.Title class="text-base font-semibold">
                    {workspace.name}
                  </Card.Title>
                </div>
              </div>
            </Card.Header>
          </Card.Root>
        </div>
      {:else}
        <div
          class="col-span-full py-20 text-center border-2 border-dashed rounded-lg"
        >
          <div
            class="inline-flex h-16 w-16 items-center justify-center rounded-full bg-muted mb-4"
          >
            <Folder class="h-8 w-8 text-muted-foreground" />
          </div>
          <h3 class="text-lg font-medium">No workspaces found</h3>
          <p class="mt-1 text-muted-foreground">
            Create your first workspace to get started.
          </p>
          <div class="mt-6">
            <Button variant="outline" onclick={openModal}>
              Create new workspace
            </Button>
          </div>
        </div>
      {/each}
    </div>
  </div>

  <div class="fixed bottom-8 right-8">
    <ModeToggle />
  </div>
</div>

<CreateWorkspaceModal isOpen={isModalOpen} onClose={closeModal} />
