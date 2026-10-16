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
    const source = new EventSource("http://localhost:3030/live_time_trials");

    source.onopen = () => {
      connectionStatus = "Connected";
      telemetryError = "";
    };

    source.addEventListener(
      "update",
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
      },
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

<h2 class="text-2xl mb-4">Live Time Trials - On Now</h2>

<div class="w-full flex flex-col gap-8">

  <!-- CURRENTLY IN PROGRESS SECTION -->
  <div class="w-full flex flex-col gap-3">
    <h2 class="text-xl font-bold text-gray-800 flex items-center gap-2">
      <span class="w-2.5 h-2.5 rounded-full bg-green-500 animate-pulse"></span>
      Driving
    </h2>

    <div class="w-full overflow-x-auto border-2 border-black shadow-sm bg-white">
      <table class="w-full text-left text-sm text-gray-700 border-collapse min-w-[700px]">
        <thead class="bg-gray-100 text-gray-700 uppercase text-xs font-semibold tracking-wider border-b-2 border-black">
          <tr>
            <th class="py-3 px-4">Name</th>
            <th class="py-3 px-4">Run Time</th>
            <th class="py-3 px-4">Upload Hash</th>
            <th class="py-3 px-4">Run #</th>
            <th class="py-3 px-4">Run UUID</th>
            <th class="py-3 px-4 text-center">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          <tr class="hover:bg-gray-50 transition-colors">
            <td class="py-3 px-4 font-semibold text-gray-900">Magnus</td>
            <td class="py-3 px-4 font-mono text-gray-600">5s</td>
            <td class="py-3 px-4 font-mono text-xs text-gray-500">a1b2c3d4e5</td>
            <td class="py-3 px-4 font-mono">#42</td>
            <td class="py-3 px-4 font-mono text-xs text-gray-500">8f3a9b12-4c...</td>
            <td class="py-3 px-4">
              <div class="flex items-center justify-center gap-2">
                <button 
                  type="button" 
                  class="px-3 py-1.5 text-xs font-medium text-amber-700 bg-amber-50 hover:bg-amber-100 border border-amber-300 transition"
                  onclick={restartRun}
                >
                  Restart
                </button>
                <button 
                  type="button" 
                  class="px-3 py-1.5 text-xs font-medium text-rose-700 bg-rose-50 hover:bg-rose-100 border border-rose-300 transition"
                  onclick={stopTelemetry}
                >
                  Cancel
                </button>
              </div>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>

  <!-- IN THE QUEUE SECTION -->
  <div class="w-full flex flex-col gap-3">
    <h2 class="text-xl font-bold text-gray-800 flex items-center gap-2">
      <span class="w-2.5 h-2.5 rounded-full bg-amber-400"></span>
      In The Queue
    </h2>

    <div class="w-full overflow-x-auto border-2 border-black shadow-sm bg-white">
      <table class="w-full text-left text-sm text-gray-700 border-collapse min-w-[700px]">
        <thead class="bg-gray-100 text-gray-700 uppercase text-xs font-semibold tracking-wider border-b-2 border-black">
          <tr>
            <th class="py-3 px-4">Name</th>
            <th class="py-3 px-4">Queued Time</th>
            <th class="py-3 px-4">Upload Hash</th>
            <th class="py-3 px-4">Run #</th>
            <th class="py-3 px-4">Run UUID</th>
            <th class="py-3 px-4 text-center">Action</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          <tr class="hover:bg-gray-50 transition-colors">
            <td class="py-3 px-4 font-semibold text-gray-900">Joe</td>
            <td class="py-3 px-4 font-mono text-gray-600">2s</td>
            <td class="py-3 px-4 font-mono text-xs text-gray-500">f9e8d7c6b5</td>
            <td class="py-3 px-4 font-mono">#43</td>
            <td class="py-3 px-4 font-mono text-xs text-gray-500">3d2e1f0a-9b...</td>
            <td class="py-3 px-4 text-center">
              <button 
                type="button" 
                class="px-3 py-1.5 text-xs font-medium text-blue-700 bg-blue-50 hover:bg-blue-100 border border-blue-300 rounded transition"
                onclick={skipQueue}
              >
                Skip The Queue
              </button>
            </td>
          </tr>
        </tbody>
      </table>
    </div>
  </div>

</div>
