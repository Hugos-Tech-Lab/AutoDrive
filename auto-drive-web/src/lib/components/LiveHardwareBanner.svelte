<script lang="ts">
  import { onMount } from "svelte";

  let { selectedBuild = $bindable() }: { selectedBuild: string } = $props();

  type HardwareData = { id: string; category: string; is_connected: boolean };
  let hardwareList = $state<HardwareData[]>([]);

  onMount(() => {
    const sse = new EventSource("http://localhost:3030/hardware/overview");

    sse.addEventListener("overview", (event) => {
      try {
        const json = JSON.parse(event.data);
        const dataObj = json.data || json; 
        hardwareList = Object.values(dataObj);
      } catch (err) {
        console.error("Failed to parse SSE overview data", err);
      }
    });

    sse.onerror = (err) => {
      console.error("SSE Error on hardware overview", err);
    };

    // Cleanup the connection when the component is destroyed
    return () => {
      sse.close();
    };
  });
</script>

<div class="row">
  <!-- Static Option -->
  <button
    type="button"
    class="button block"
    class:selected={selectedBuild === "Time Trials"}
    aria-pressed={selectedBuild === "Time Trials"}
    onclick={() => (selectedBuild = "Time Trials")}
  >
    Time Trials
  </button>

  {#each hardwareList as hw}
    <button
      type="button"
      class="button block"
      class:selected={selectedBuild === hw.id}
      aria-pressed={selectedBuild === hw.id}
      onclick={() => (selectedBuild = hw.id)}
    >
      {hw.is_connected ? "(CONNECTED)" : "(NOT CONNECTED)"} {hw.id}
    </button>
  {/each}
</div>

<style>
  .row {
    display: flex;
    gap: 0.5rem;
  }

  .row .button {
    width: auto;
  }

  .button.selected {
    background-color: #a4aeff;
  }
</style>