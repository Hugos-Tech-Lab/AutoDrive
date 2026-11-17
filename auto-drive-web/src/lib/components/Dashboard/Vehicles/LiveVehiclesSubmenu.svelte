<script lang="ts">
  import { onMount } from "svelte";
  import { endpointRoot } from "../../../../endpoint";

  let {
    selectedVehicleNumberPlate = $bindable(),
  }: { selectedVehicleNumberPlate: string } = $props();

  type HardwareData = {
    number_plate: string;
    category: string;
    is_connected: boolean;
  };

  let hardwareList = $state<HardwareData[]>([]);
  let backendConnected = $state(false);
  let isRetrying = $state(false);

  let sse: EventSource | null = null;

  function connect() {
    // Clean up any existing connection before starting a new one
    if (sse) {
      sse.close();
    }

    isRetrying = true;
    sse = new EventSource(`${endpointRoot()}/hardware/overview`);

    sse.addEventListener("overview", (event) => {
      try {
        const json = JSON.parse(event.data);
        const dataObj = json.data || json;
        hardwareList = Object.values(dataObj);
      } catch (err) {
        console.error("Failed to parse SSE overview data", err);
      }

      backendConnected = true;
      isRetrying = false;
    });

    sse.onopen = () => {
      backendConnected = true;
      isRetrying = false;
    };

    sse.onerror = (err) => {
      console.error("SSE Error on hardware overview", err);
      backendConnected = false;
      isRetrying = false;
    };
  }

  onMount(() => {
    connect();

    // Cleanup the connection when the component is destroyed
    return () => {
      sse?.close();
    };
  });
</script>

<div class="row">
  {#if backendConnected}
    {#each hardwareList as hw}
      <button
        type="button"
        class="button block"
        class:selected={selectedVehicleNumberPlate === hw.number_plate}
        aria-pressed={selectedVehicleNumberPlate === hw.number_plate}
        onclick={() => (selectedVehicleNumberPlate = hw.number_plate)}
      >
        {hw.number_plate}
        {hw.is_connected ? "(CONNECTED)" : "(NOT CONNECTED)"}
      </button>
    {/each}
  {:else}
    <button
      type="button"
      class="button block"
      disabled={isRetrying}
      onclick={connect}
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
