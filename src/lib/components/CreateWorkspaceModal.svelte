<script lang="ts">
  import { workspaces } from "../stores/workspace";
  import { Button } from "$lib/components/ui/button/index.js";
  import { Input } from "$lib/components/ui/input/index.js";
  import { Label } from "$lib/components/ui/label/index.js";
  import * as Dialog from "$lib/components/ui/dialog/index.js";

  export let isOpen = false;
  export let onClose: () => void;

  let workspaceName = "";
  let error = "";
  let isLoading = false;

  async function handleSubmit() {
    if (!workspaceName.trim()) {
      error = "Workspace name is required";
      return;
    }

    // Basic validation
    if (!/^[a-zA-Z0-9_\-]+$/.test(workspaceName)) {
      error = "Only letters, numbers, underscores and hyphens allowed";
      return;
    }

    isLoading = true;
    error = "";

    const success = await workspaces.createWorkspace(workspaceName);

    isLoading = false;

    if (success) {
      workspaceName = "";
      onClose();
    } else {
      error = "Failed to create workspace. It might already exist.";
    }
  }

  // Handle open state change to sync with parent
  $: if (!isOpen) {
    // Logic if needed when closed externally
  }
</script>

<Dialog.Root
  bind:open={isOpen}
  onOpenChange={(open: boolean) => !open && onClose()}
>
  <Dialog.Content class="sm:max-w-[425px]">
    <Dialog.Header>
      <Dialog.Title>Create New Workspace</Dialog.Title>
      <Dialog.Description>
        Enter a name for your new workspace. Click create when you're done.
      </Dialog.Description>
    </Dialog.Header>
    <div class="grid gap-4 py-4">
      <div class="grid gap-2">
        <Label for="name">Workspace Name</Label>
        <Input
          id="name"
          bind:value={workspaceName}
          placeholder="my-awesome-project"
          onkeydown={(e) => e.key === "Enter" && handleSubmit()}
        />
        {#if error}
          <p class="text-sm text-red-500">{error}</p>
        {/if}
      </div>
    </div>
    <Dialog.Footer>
      <Button variant="outline" onclick={onClose} disabled={isLoading}
        >Cancel</Button
      >
      <Button onclick={handleSubmit} disabled={isLoading}>
        {isLoading ? "Creating..." : "Create Workspace"}
      </Button>
    </Dialog.Footer>
  </Dialog.Content>
</Dialog.Root>
