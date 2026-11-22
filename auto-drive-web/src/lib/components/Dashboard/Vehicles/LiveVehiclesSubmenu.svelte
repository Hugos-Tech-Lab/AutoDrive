<script lang="ts">
  import { onMount } from "svelte";
  import { endpointRoot } from "../../../../endpoint";

  let {
    selectedVehicleNumberPlate = $bindable(),
  }: { selectedVehicleNumberPlate: string } = $props();

  let numberPlates = $state<string[]>([]);
  let backendConnected = $state(false);
  let isRetrying = $state(false);

  async function loadVehicles() {
    isRetrying = true;
    try {
      const res = await fetch(`${endpointRoot()}/vehicle/overview`);
      if (!res.ok) throw new Error(`HTTP ${res.status}`);

      const data: { number_plates: string[] } = await res.json();
      numberPlates = data.number_plates;
      backendConnected = true;
    } catch (err) {
      console.error("Failed to fetch vehicle overview", err);
      backendConnected = false;
    } finally {
      isRetrying = false;
    }
  }

  onMount(loadVehicles);
</script>

<div class="row">
  {#if backendConnected}
    {#each numberPlates as plate}
      <button
        type="button"
        class="button block"
        class:selected={selectedVehicleNumberPlate === plate}
        aria-pressed={selectedVehicleNumberPlate === plate}
        onclick={() => (selectedVehicleNumberPlate = plate)}
      >
        {plate}
      </button>
    {/each}
  {:else}
    <button
      type="button"
      class="button block"
      disabled={isRetrying}
      onclick={loadVehicles}
    >
      {isRetrying ? "Retrying..." : "NETWORK ERROR. Press to retry."}
    </button>
  {/if}
</div>

<style>
  .row {
    display: flex;
  }

  .row .button {
    width: auto;
  }

  .button.selected {
    background-color: #a4aeff;
  }
</style>
