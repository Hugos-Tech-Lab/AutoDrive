<script lang="ts">
  import { onMount } from "svelte";
  import TheBeginnerCar from "../TheBeginnerCarDrawing.svelte";
  import { endpointRoot } from "../../../endpoint";
  import Status from "../Vehicles/Vehicle/Status.svelte";

  type Status = {
    utmost_front_utmost_left: number;
    utmost_front_utmost_right: number;
  };

  let status = $state<Status | null>(null);
  let connectionStatus = $state("Connecting...");
  let lastUpdate = $state<Date | null>(null);
  let telemetryError = $state("");

  onMount(() => {
    const source = new EventSource(`${endpointRoot()}/status`);

    source.onopen = () => {
      connectionStatus = "Connected";
      telemetryError = "";
    };

    source.addEventListener("status", (event: MessageEvent<string>) => {
      try {
        const parsed: unknown = JSON.parse(event.data);

        lastUpdate = new Date();
        telemetryError = "";
      } catch (error) {
        telemetryError = "Failed to parse telemetry data";
        console.error("[Telemetry] Invalid event:", error);
      }
    });

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

<Status />

<div class="row"></div>
