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

  let hardwareList = $state<HardwareData[]>([
    { number_plate: "ABC 123", category: "", is_connected: false },
    { number_plate: "ABC 256", category: "", is_connected: false },
    { number_plate: "ABC 788", category: "", is_connected: false },
    { number_plate: "ABC 788", category: "", is_connected: false },
    { number_plate: "ABC 788", category: "", is_connected: false },
    { number_plate: "ABC 788", category: "", is_connected: false },
    { number_plate: "ABC 788", category: "", is_connected: false },
  ]);
  let backendConnected = $state(false);

  onMount(() => {
    const sse = new EventSource(`${endpointRoot()}/hardware/overview`);

    sse.addEventListener("overview", (event) => {
      try {
        const json = JSON.parse(event.data);
        const dataObj = json.data || json;
        hardwareList = Object.values(dataObj);
      } catch (err) {
        console.error("Failed to parse SSE overview data", err);
      }

      backendConnected = true;
    });

    sse.onopen = () => {
      backendConnected = true;
    };

    sse.onerror = (err) => {
      console.error("SSE Error on hardware overview", err);
      backendConnected = false;
    };

    // Cleanup the connection when the component is destroyed
    return () => {
      sse.close();
    };
  });
</script>

<div class="row">
  {#each hardwareList as hw}
    <button
      type="button"
      class="button block"
      class:selected={selectedVehicleNumberPlate === hw.number_plate}
      aria-pressed={selectedVehicleNumberPlate === hw.number_plate}
      onclick={() => (selectedVehicleNumberPlate = hw.number_plate)}
    >
      <!-- TODO: it shows connected even when connection to backend not made? Fix that -->
      {hw.number_plate}
      <!-- {backendConnected && hw.is_connected ? "(CONNECTED)" : "(NOT CONNECTED)"} -->
    </button>
  {/each}
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
