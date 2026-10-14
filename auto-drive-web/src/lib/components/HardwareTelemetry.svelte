
<script lang="ts">
  import { onMount } from "svelte";
  import TheBeginnerCar from "./TheBeginnerCarDrawing.svelte";

  // type HardwareTelemetry = {
  //   IrSensorReading?: {
  //     value: number;
  //   };
  // };

  // let telemetry: HardwareTelemetry = {};
  let irSensorValue = $state<number | undefined>(undefined);
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

    source.addEventListener("ir_sensor_a_reading", (event: MessageEvent) => {
      irSensorValue = event.data;

      try {
        // const data: HardwareTelemetry = JSON.parse(event.data);

        // telemetry = data;

        // if (data.IrSensorReading) {
        //   irSensorValue = data.IrSensorReading.value;
        // }

        lastUpdate = new Date();
        telemetryError = "";

        // console.log("[Telemetry]", data);
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
  <title>Hardware Telemetry</title>
</svelte:head>

<h2 class="text-2xl mb-4">Telemetry</h2>

<TheBeginnerCar IRAValue={irSensorValue} />

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

  <div class="rounded border p-4 text-center">
    <p class="text-sm text-gray-500">IR Sensor A</p>
    <p class="text-2xl font-bold">
      {irSensorValue ?? "Waiting for data..."}
    </p>
    <p class="text-xs text-gray-500">Raw sensor value</p>
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
