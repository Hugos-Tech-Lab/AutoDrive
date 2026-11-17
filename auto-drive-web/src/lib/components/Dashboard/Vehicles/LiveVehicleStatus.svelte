<script lang="ts">
  import { onMount } from "svelte";
  import TheBeginnerCar from "../../TheBeginnerCarDrawing.svelte";
  import { endpointRoot } from "../../../../endpoint";

  type Status = {
    utmost_front_utmost_left: number;
    utmost_front_utmost_right: number;
  };

  let status = $state<Status | null>(null);
  let connectionStatus = $state("Connecting...");
  let lastUpdate = $state<Date | null>(null);
  let telemetryError = $state("");

  interface Props {
    vehicle_number_plate: string;
  }

  let { vehicle_number_plate }: Props = $props();

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
</script>

<svelte:head>
  <title>Status</title>
</svelte:head>

<h2 class="text-2xl mb-4">The STATE of the MCUs controlling the vehicle</h2>
<div
  class="w-full flex flex-col md:flex-row gap-4 mb-6 bg-gray-100 p-4 border-2"
>
  <div class="flex-1 bg-green-100 p-4 border-2">
    <h3>Wifi MCU</h3>

    Uptime: 00:00:00. Firmware: #1233987.
  </div>

  <div class="flex-1 bg-green-100 p-4 border-2">
    <h3>WASM MCU</h3>
    Uptime: 00:00:00. Firmware: #abcde.
  </div>
</div>
