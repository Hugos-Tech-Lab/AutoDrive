<script lang="ts">
  import { onMount } from "svelte";

  type TimeTrialDrivingEntry = {
    name: string;
    run_time: string;
    upload_hash: string;
    run_number: number;
    run_uuid: string;
    uploader_comment: string;
  };

  type TimeTrialQueueEntry = {
    name: string;
    waiting_time: string;
    upload_hash: string;
    run_number: number;
    run_uuid: string;
    uploader_comment: string;
  };

  type TimeTrialData = {
    driving_now: Array<TimeTrialDrivingEntry>;
    queue: Array<TimeTrialQueueEntry>;
  };

  let timeTrialsData = $state<TimeTrialData | null>(null);
  let connectionStatus = $state("Connecting...");
  let telemetryError = $state("");

  // Variables for the client-side time offset ticker
  let lastUpdateMs = $state(Date.now());
  let localTickOffset = $state(0);

  onMount(() => {
    // 1. Setup the interval to tick the seconds up locally between server updates
    const ticker = setInterval(() => {
      if (timeTrialsData) {
        localTickOffset = Math.floor((Date.now() - lastUpdateMs) / 1000);
      }
    }, 1000);

    // 2. Setup the SSE connection
    const source = new EventSource("http://localhost:3030/autoscript/watch");

    source.onopen = () => {
      connectionStatus = "Connected";
      telemetryError = "";
    };

    source.addEventListener("update", (event: MessageEvent<string>) => {
      try {
        const parsed = JSON.parse(event.data) as TimeTrialData;

        if (!parsed || typeof parsed !== "object") {
          throw new Error("Invalid telemetry payload");
        }

        // Store data and reset the local ticker
        timeTrialsData = parsed;
        lastUpdateMs = Date.now();
        localTickOffset = 0;
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
      clearInterval(ticker);
      source.close();
    };
  });

  // Helper to parse the server's time string (e.g. "5s" or "5") and add the local offset
  function displayTime(serverTimeStr: string, offset: number): string {
    const baseTime = parseInt(serverTimeStr.replace(/\D/g, ""), 10) || 0;
    return `${baseTime + offset}s`;
  }

  function skipTheQueue(run_uuid: string) {
    console.log("Skipping queue for", run_uuid);
    // TODO: Call your hardware stop endpoint.
  }

  function restart(run_uuid: string) {
    console.log("Restarting", run_uuid);
    // TODO: HTTP request to restart.
  }

  function cancel(run_uuid: string) {
    console.log("Canceling", run_uuid);
    // TODO: HTTP request to cancel.
  }
</script>

<svelte:head>
  <title>Hardware Telemetry</title>
</svelte:head>

<div class="flex items-center justify-between mb-4">
  <h2 class="text-2xl">Live Time Trials - On Now</h2>
</div>

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
            <th class="py-3 px-4">Run #</th>
            <th class="py-3 px-4">Run UUID</th>
            <th class="py-3 px-4">Upload Hash</th>
            <th class="py-3 px-4">Upload Comment</th>
            <th class="py-3 px-4 text-center">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          {#if timeTrialsData?.driving_now && timeTrialsData.driving_now.length > 0}
            {#each timeTrialsData.driving_now as entry (entry.run_uuid)}
              <tr class="hover:bg-gray-50 transition-colors">
                <td class="py-3 px-4 font-semibold text-gray-900">{entry.name}</td>
                <td class="py-3 px-4 font-mono text-gray-600">{displayTime(entry.run_time, localTickOffset)}</td>
                <td class="py-3 px-4 font-mono">#{entry.run_number}</td>
                <td class="py-3 px-4 font-mono text-xs text-gray-500">{entry.run_uuid.substring(0, 13)}...</td>
                <td class="py-3 px-4 font-mono text-xs text-gray-500">{entry.upload_hash.substring(0, 10)}</td>
                <td class="py-3 px-4 font-mono text-xs text-gray-500">{entry.uploader_comment.substring(0, 10)}</td>
                <td class="py-3 px-4">
                  <div class="flex items-center justify-center gap-2">
                    <button 
                      type="button" 
                      class="px-3 py-1.5 text-xs font-medium text-amber-700 bg-amber-50 hover:bg-amber-100 border border-amber-300 transition rounded"
                      onclick={() => restart(entry.run_uuid)}
                    >
                      Restart
                    </button>
                    <button 
                      type="button" 
                      class="px-3 py-1.5 text-xs font-medium text-rose-700 bg-rose-50 hover:bg-rose-100 border border-rose-300 transition rounded"
                      onclick={() => cancel(entry.run_uuid)}
                    >
                      Cancel
                    </button>
                  </div>
                </td>
              </tr>
            {/each}
          {:else}
            <tr>
              <td colspan="6" class="py-6 px-4 text-center text-gray-500 italic">No cars currently driving.</td>
            </tr>
          {/if}
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
          {#if timeTrialsData?.queue && timeTrialsData.queue.length > 0}
            {#each timeTrialsData.queue as entry (entry.run_uuid)}
              <tr class="hover:bg-gray-50 transition-colors">
                <td class="py-3 px-4 font-semibold text-gray-900">{entry.name}</td>
                <td class="py-3 px-4 font-mono text-gray-600">{displayTime(entry.waiting_time, localTickOffset)}</td>
                <td class="py-3 px-4 font-mono text-xs text-gray-500">{entry.upload_hash.substring(0, 10)}</td>
                <td class="py-3 px-4 font-mono">#{entry.run_number}</td>
                <td class="py-3 px-4 font-mono text-xs text-gray-500">{entry.run_uuid.substring(0, 13)}...</td>
                <td class="py-3 px-4 text-center">
                  <button 
                    type="button" 
                    class="px-3 py-1.5 text-xs font-medium text-blue-700 bg-blue-50 hover:bg-blue-100 border border-blue-300 rounded transition"
                    onclick={() => skipTheQueue(entry.run_uuid)}
                  >
                    Skip The Queue
                  </button>
                </td>
              </tr>
            {/each}
          {:else}
            <tr>
              <td colspan="6" class="py-6 px-4 text-center text-gray-500 italic">Queue is currently empty.</td>
            </tr>
          {/if}
        </tbody>
      </table>
    </div>
  </div>

</div>

<div class="text-sm">
  <span class={connectionStatus === 'Connected' ? 'text-green-600' : 'text-amber-600'}>
    {connectionStatus}
  </span>
  {#if telemetryError}
    <span class="text-red-500 ml-2">({telemetryError})</span>
  {/if}
</div>
