
<script lang="ts">
  import { onMount } from "svelte";
  import TheBeginnerCar from "../TheBeginnerCarDrawing.svelte";
  import { endpointRoot } from "../../../endpoint";

  type Status = {
    utmost_front_utmost_left: number;
    utmost_front_utmost_right: number;
  };

  let status = $state<Status | null>(null);
  let connectionStatus = $state("Connecting...");
  let lastUpdate = $state<Date | null>(null);
  let telemetryError = $state("");

  onMount(() => {
    const source = new EventSource(
      `${endpointRoot()}/status`
    );

    source.onopen = () => {
      connectionStatus = "Connected";
      telemetryError = "";
    };

    source.addEventListener(
      "status",
      (event: MessageEvent<string>) => {
        try {
          const parsed: unknown = JSON.parse(event.data);

    
          lastUpdate = new Date();
          telemetryError = "";
        } catch (error) {
          telemetryError = "Failed to parse telemetry data";
          console.error("[Telemetry] Invalid event:", error);
        }
      }
    );

    source.onerror = () => {
      connectionStatus =
        source.readyState === EventSource.CLOSED
          ? "Disconnected"
          : "Reconnecting...";
    };

    return () => {
      source.close();
    };
  });

  function stopTelemetry() {
    // TODO: Call your hardware stop endpoint.
  }

  function runScript() {
    // TODO: Call your script execution endpoint.
  }

  function navigateToStart() {
    // TODO: Call your navigation endpoint.
  }
</script>

<svelte:head>
  <title>Status</title>
</svelte:head>

<h2 class="text-2xl mb-4">Status</h2>


<div class="flex flex-col gap-4 mb-6">
  Wifi MCU. Uptime: 00:00:00. Software: #1233987. 
</div>

<div class="flex flex-col gap-4 mb-6">
  Wasm MCU. Uptime: 00:00:00. Software: #abcde. 
</div>

<div class="row">

</div>
