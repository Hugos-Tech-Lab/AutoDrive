
<script lang="ts">
  import { onMount } from "svelte";
  import TheBeginnerCarDrawing from "./TheBeginnerCarDrawing.svelte";

  type HardwareTelemetry = Record<string, unknown>;

  let telemetry: HardwareTelemetry = {};
  let connectionStatus = "Connecting...";
  let lastUpdate: Date | null = null;
  let telemetryError = "";

  onMount(() => {
    const source = new EventSource(
      "http://localhost:3030/hardware_telemetry"
    );

    source.onopen = () => {
      connectionStatus = "Connected";
      telemetryError = "";
    };

    source.onmessage = (event: MessageEvent<string>) => {
      console.log(event);

      // try {
      //   const data: HardwareTelemetry = JSON.parse(event.data);

      //   telemetry = data;
      //   lastUpdate = new Date();
      //   telemetryError = "";
      // } catch (error) {
      //   telemetryError = "Failed to parse telemetry data";
      //   console.error("Invalid telemetry event:", error);
      // }
    };

    source.onerror = () => {
      connectionStatus = source.readyState === EventSource.CLOSED
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
  <title>Hardware Telemetry</title>
</svelte:head>

<h2 class="text-2xl mb-4">Telemetry</h2>

<!-- <TheBeginnerCarDrawing {telemetry} /> -->

<div class="flex flex-col items-center gap-4 mb-6">
  <div class="flex gap-4 text-sm">
    <span class="flex items-center gap-2">
      <span class="w-2.5 h-2.5 rounded-full bg-cyan-500"></span>
      GPS
    </span>

    <span class="flex items-center gap-2">
      <span class="w-2.5 h-2.5 rounded-full bg-green-500"></span>
      Sensors
    </span>
  </div>

  <div class="text-sm">
    Connection:
    <span
      class:text-green-600={connectionStatus === "Connected"}
      class:text-yellow-600={connectionStatus === "Reconnecting..."}
      class:text-red-600={connectionStatus === "Disconnected"}
    >
      {connectionStatus}
    </span>
  </div>

  {#if lastUpdate}
    <p class="text-xs text-gray-500">
      <!-- Last update: {lastUpdate.toLocaleTimeString()} -->
    </p>
  {/if}

  {#if telemetryError}
    <p class="text-sm text-red-500">{telemetryError}</p>
  {/if}

  <details class="w-full">
    <summary class="cursor-pointer text-sm">
      Raw telemetry
    </summary>
    <pre class="mt-2 overflow-auto rounded bg-gray-100 p-3 text-xs text-gray-800">{JSON.stringify(telemetry, null, 2)}</pre>
  </details>
</div>

<div class="row">

</div>
