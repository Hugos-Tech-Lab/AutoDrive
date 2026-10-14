
<script lang="ts">
  import { onMount } from "svelte";
  import TheBeginnerCar from "./TheBeginnerCarDrawing.svelte";

  type InfraredLedReadings = {
    utmost_front_utmost_left: number;
    utmost_front_utmost_right: number;
  };

  let irSensorReadings = $state<InfraredLedReadings | null>(null);
  let connectionStatus = $state("Connecting...");
  let lastUpdate = $state<Date | null>(null);
  let telemetryError = $state("");

  onMount(() => {
    const source = new EventSource(
      "http://localhost:3030/hardware_telemetry"
    );

    source.onopen = () => {
      connectionStatus = "Connected";
      telemetryError = "";
    };

    source.addEventListener(
      "infrared_led_reading",
      (event: MessageEvent<string>) => {
        try {
          const parsed: unknown = JSON.parse(event.data);

          if (typeof parsed !== "object" || parsed === null) {
            throw new Error("Invalid telemetry payload");
          }

          const data = parsed as Record<string, unknown>;
          const left = data["utmost_front_utmost_left"];
          const right = data["utmost_front_utmost_right"];

          if (typeof left !== "number" || typeof right !== "number") {
            throw new Error("Missing or invalid sensor readings");
          }

          irSensorReadings = {
            utmost_front_utmost_left: left,
            utmost_front_utmost_right: right,
          };

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
  <title>Hardware Telemetry</title>
</svelte:head>

<h2 class="text-2xl mb-4">Telemetry</h2>

<TheBeginnerCar
  IRUtmostFrontUtmostLeft={irSensorReadings?.utmost_front_utmost_left}
  IRUtmostFrontUtmostRight={irSensorReadings?.utmost_front_utmost_right}
/>

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

  <p class="text-sm">
    Connection: {connectionStatus}
  </p>

  <div class="grid grid-cols-1 sm:grid-cols-2 gap-4 w-full">
    <div class="rounded border p-4 text-center">
      <p class="text-sm text-gray-500">Front Left IR</p>
      <p class="text-2xl font-bold">
        {irSensorReadings?.utmost_front_utmost_left ?? "Waiting for data..."}
      </p>
      <p class="text-xs text-gray-500">Raw sensor value</p>
    </div>

    <div class="rounded border p-4 text-center">
      <p class="text-sm text-gray-500">Front Right IR</p>
      <p class="text-2xl font-bold">
        {irSensorReadings?.utmost_front_utmost_right ?? "Waiting for data..."}
      </p>
      <p class="text-xs text-gray-500">Raw sensor value</p>
    </div>
  </div>

  {#if lastUpdate}
    <p class="text-xs text-gray-500">
      Last update: {lastUpdate.toLocaleTimeString()}
    </p>
  {/if}

  {#if telemetryError}
    <p class="text-sm text-red-500">{telemetryError}</p>
  {/if}
</div>

<div class="row">
  <button type="button" class="button block" onclick={stopTelemetry}>
    STOP
  </button>

  <button type="button" class="button block" onclick={runScript}>
    Run Script
  </button>

  <button
    type="button"
    class="button block"
    onclick={navigateToStart}
  >
    Navigate To Start
  </button>
</div>
