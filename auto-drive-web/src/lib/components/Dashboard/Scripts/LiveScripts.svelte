<script lang="ts">
  import { onMount } from "svelte";
  import { endpointRoot } from "../../../../endpoint";
  import { type AutoScriptMetadata } from "../../../../../../auto-drive-edge/api/auto-drive-edge-protocol/ts/AutoScriptMetadata";

  // Define your fleet's number plates here (or fetch them from an endpoint)
  const AVAILABLE_CARS = ["TBC 001"];

  type AutoScriptOverviewStreamedResponse = {
    auto_scripts: Array<AutoScriptMetadata>;
  };

  type AutoScripts = {
    driving: Array<AutoScriptMetadata>;
    queued: Array<AutoScriptMetadata>;
    stopped: Array<AutoScriptMetadata>;
  };

  let timeTrialsData = $state<AutoScripts | null>(null);
  let connectionStatus = $state("Connecting...");
  let telemetryError = $state("");
  let pendingRuns = $state<Record<string, boolean>>({});

  // Variables for the client-side time offset ticker
  let lastUpdateMs = $state(Date.now());
  let localTickOffset = $state(0);

  // Derive a nicely formatted string for the last refresh time
  let lastRefreshTime = $derived(
    timeTrialsData ? new Date(lastUpdateMs).toLocaleTimeString() : "Never",
  );

  // Track which number plates are currently driving so we can disable their buttons
  let busyPlates = $derived(
    new Set(
      (timeTrialsData?.driving ?? [])
        .map((s) => s.vehicle_number_plate)
        .filter((p): p is string => Boolean(p)),
    ),
  );

  onMount(() => {
    const ticker = setInterval(() => {
      if (timeTrialsData) {
        localTickOffset = Math.floor((Date.now() - lastUpdateMs) / 1000);
      }
    }, 1000);

    // 2. Setup the SSE connection
    const source = new EventSource(`${endpointRoot()}/autoscript/overview`);

    source.onopen = () => {
      connectionStatus = "Connected";
      telemetryError = "";
    };

    source.addEventListener("overview", (event: MessageEvent<string>) => {
      try {
        const parsed = JSON.parse(
          event.data,
        ) as AutoScriptOverviewStreamedResponse;

        if (!parsed || !Array.isArray(parsed.auto_scripts)) {
          throw new Error("Invalid telemetry payload");
        }

        console.log(parsed.auto_scripts);

        const grouped: AutoScripts = {
          driving: [],
          queued: [],
          stopped: [],
        };

        for (const script of parsed.auto_scripts) {
          if (script.lifecycle === "Idle") {
            grouped.queued.push(script);
          } else if (
            script.lifecycle === "StoppedByDisconnect" ||
            script.lifecycle === "StoppedByHttpCancel" ||
            script.lifecycle === "StoppedBySafety" ||
            script.lifecycle === "StoppedByScriptEnd"
          ) {
            grouped.stopped.push(script);
          } else {
            grouped.driving.push(script);
          }
        }

        // Sort all arrays by run_number descending (biggest first)
        const sortDesc = (a: AutoScriptMetadata, b: AutoScriptMetadata) =>
          (b.run_number ?? 0) - (a.run_number ?? 0);
        grouped.driving.sort(sortDesc);
        grouped.queued.sort(sortDesc);
        grouped.stopped.sort(sortDesc);

        // Store data and reset the local ticker
        timeTrialsData = grouped;
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
  function displayTime(
    serverTimeStr: string | undefined,
    offset: number,
  ): string {
    if (!serverTimeStr) return `${offset}s`;
    const baseTime = parseInt(serverTimeStr.replace(/\D/g, ""), 10) || 0;
    return `${baseTime + offset}s`;
  }

  async function runScript(run_uuid: string | undefined, number_plate: string) {
    if (!run_uuid) return;
    const key = `${run_uuid}-${number_plate}`;

    try {
      const response = await fetch(`${endpointRoot()}/autoscript/run`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ run_uuid, number_plate }),
      });

      const data = await response.json();
      if (!response.ok || !data.success) {
        console.error(
          "Failed to run auto script:",
          data.message || response.statusText,
        );
        telemetryError = data.message || "Failed to start script on car";
      }
    } catch (error) {
      console.error("Error executing run request:", error);
      telemetryError = "Network error starting script";
    }
  }

  function restart(run_uuid: string | undefined) {
    if (!run_uuid) return;
    console.log("Restarting", run_uuid);
    // TODO: HTTP request to restart.
  }

  async function cancel(run_uuid: string | undefined) {
    if (!run_uuid) return;
    console.log("Canceling", run_uuid);
    try {
      const response = await fetch(`${endpointRoot()}/autoscript/cancel`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ run_uuid }),
      });
      if (!response.ok) {
        console.error("Failed to cancel run:", response.statusText);
      }
    } catch (error) {
      console.error("Error executing cancel request:", error);
    }
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

    <div
      class="w-full overflow-x-auto border-2 border-black shadow-sm bg-white"
    >
      <table
        class="w-full text-left text-sm text-gray-700 border-collapse min-w-[700px]"
      >
        <thead
          class="bg-gray-100 text-gray-700 uppercase text-xs font-semibold tracking-wider border-b-2 border-black"
        >
          <tr>
            <th class="py-3 px-4">Owner</th>
            <th class="py-3 px-4">Script Start</th>
            <th class="py-3 px-4">Run #</th>
            <th class="py-3 px-4">Run UUID</th>
            <th class="py-3 px-4">Upload Hash</th>
            <th class="py-3 px-4">Upload Comment</th>
            <th class="py-3 px-4">Number Plate</th>
            <th class="py-3 px-4">Lifecycle</th>
            <th class="py-3 px-4 text-center">Actions</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          {#if timeTrialsData?.driving && timeTrialsData.driving.length > 0}
            {#each timeTrialsData.driving as entry, i (entry.run_uuid || i)}
              <tr class="hover:bg-gray-50 transition-colors">
                <td class="py-3 px-4 font-semibold text-gray-900"
                  >{entry.owner_name || "Unknown"}</td
                >
                <td class="py-3 px-4 font-mono text-gray-600"
                  >{entry.script_start_time}</td
                >
                <td class="py-3 px-4 font-mono">#{entry.run_number ?? "-"}</td>
                <td class="py-3 px-4 font-mono text-xs text-gray-500"
                  >{(entry.run_uuid || "").substring(0, 13)}...</td
                >
                <td class="py-3 px-4 font-mono text-xs text-gray-500"
                  >{(entry.uploaded_script_hash || "").substring(0, 10)}</td
                >
                <td class="py-3 px-4 font-mono text-xs text-gray-500"
                  >{(entry.owner_comment || "").substring(0, 10)}</td
                >
                <td class="py-3 px-4 font-mono text-xs text-gray-500"
                  >{(entry.vehicle_number_plate || "").substring(0, 10)}</td
                >
                <td class="py-3 px-4 font-mono text-xs text-gray-500"
                  >{(entry.lifecycle || "").substring(0, 10)}</td
                >
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
              <td colspan="8" class="py-6 px-4 text-center text-gray-500 italic"
                >No cars currently driving.</td
              >
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

    <div
      class="w-full overflow-x-auto border-2 border-black shadow-sm bg-white"
    >
      <table
        class="w-full text-left text-sm text-gray-700 border-collapse min-w-[700px]"
      >
        <thead
          class="bg-gray-100 text-gray-700 uppercase text-xs font-semibold tracking-wider border-b-2 border-black"
        >
          <tr>
            <th class="py-3 px-4">Owner</th>
            <th class="py-3 px-4">Queued Time</th>
            <th class="py-3 px-4">Upload Hash</th>
            <th class="py-3 px-4">Run #</th>
            <th class="py-3 px-4">Run UUID</th>
            <th class="py-3 px-4 text-center">Run on Car</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          {#if timeTrialsData?.queued && timeTrialsData.queued.length > 0}
            {#each timeTrialsData.queued as entry, i (entry.run_uuid || i)}
              <tr class="hover:bg-gray-50 transition-colors">
                <td class="py-3 px-4 font-semibold text-gray-900"
                  >{entry.owner_name || "Unknown"}</td
                >
                <td class="py-3 px-4 font-mono text-gray-600"
                  >{displayTime(entry.script_start_time, localTickOffset)}</td
                >
                <td class="py-3 px-4 font-mono text-xs text-gray-500"
                  >{(entry.uploaded_script_hash || "").substring(0, 10)}</td
                >
                <td class="py-3 px-4 font-mono">#{entry.run_number ?? "-"}</td>
                <td class="py-3 px-4 font-mono text-xs text-gray-500"
                  >{(entry.run_uuid || "").substring(0, 13)}...</td
                >
                <td class="py-3 px-4">
                  <div class="flex flex-wrap items-center justify-center gap-2">
                    {#each AVAILABLE_CARS as plate (plate)}
                      {@const isBusy = busyPlates.has(plate)}
                      {@const isPending =
                        pendingRuns[`${entry.run_uuid}-${plate}`]}
                      <button
                        type="button"
                        disabled={isBusy || isPending}
                        title={isBusy
                          ? `${plate} is currently driving`
                          : `Run on ${plate}`}
                        class="px-2.5 py-1.5 text-xs font-mono font-medium rounded border transition
                          {isBusy
                          ? 'text-gray-400 bg-gray-100 border-gray-200 cursor-not-allowed'
                          : 'text-blue-700 bg-blue-50 hover:bg-blue-100 border-blue-300'}"
                        onclick={() => runScript(entry.run_uuid, plate)}
                      >
                        {isPending ? "Starting..." : `Run ${plate}`}
                      </button>
                    {/each}
                  </div>
                </td>
              </tr>
            {/each}
          {:else}
            <tr>
              <td colspan="6" class="py-6 px-4 text-center text-gray-500 italic"
                >Queue is currently empty.</td
              >
            </tr>
          {/if}
        </tbody>
      </table>
    </div>
  </div>

  <!-- STOPPED SECTION -->
  <div class="w-full flex flex-col gap-3">
    <h2 class="text-xl font-bold text-gray-800 flex items-center gap-2">
      <span class="w-2.5 h-2.5 rounded-full bg-red-500"></span>
      Stopped
    </h2>

    <div
      class="w-full overflow-x-auto border-2 border-black shadow-sm bg-white"
    >
      <table
        class="w-full text-left text-sm text-gray-700 border-collapse min-w-[700px]"
      >
        <thead
          class="bg-gray-100 text-gray-700 uppercase text-xs font-semibold tracking-wider border-b-2 border-black"
        >
          <tr>
            <th class="py-3 px-4">Owner</th>
            <th class="py-3 px-4">Run #</th>
            <th class="py-3 px-4">Run UUID</th>
            <th class="py-3 px-4">Upload Hash</th>
            <th class="py-3 px-4">Upload Comment</th>
            <th class="py-3 px-4">Number Plate</th>
            <th class="py-3 px-4">Lifecycle</th>
          </tr>
        </thead>
        <tbody class="divide-y divide-gray-200">
          {#if timeTrialsData?.stopped && timeTrialsData.stopped.length > 0}
            {#each timeTrialsData.stopped as entry, i (entry.run_uuid || i)}
              <tr class="hover:bg-gray-50 transition-colors">
                <td class="py-3 px-4 font-semibold text-gray-900"
                  >{entry.owner_name || "Unknown"}</td
                >
                <td class="py-3 px-4 font-mono">#{entry.run_number ?? "-"}</td>
                <td class="py-3 px-4 font-mono text-xs text-gray-500"
                  >{(entry.run_uuid || "").substring(0, 13)}...</td
                >
                <td class="py-3 px-4 font-mono text-xs text-gray-500"
                  >{(entry.uploaded_script_hash || "").substring(0, 10)}</td
                >
                <td class="py-3 px-4 font-mono text-xs text-gray-500"
                  >{(entry.owner_comment || "").substring(0, 10)}</td
                >
                <td class="py-3 px-4 font-mono text-xs text-gray-500"
                  >{(entry.vehicle_number_plate || "").substring(0, 10)}</td
                >
                <td class="py-3 px-4 font-mono text-xs text-gray-500"
                  >{(entry.lifecycle || "").substring(0, 10)}</td
                >
              </tr>
            {/each}
          {:else}
            <tr>
              <td colspan="6" class="py-6 px-4 text-center text-gray-500 italic"
                >No recently stopped runs.</td
              >
            </tr>
          {/if}
        </tbody>
      </table>
    </div>
  </div>
</div>

<!-- BOTTOM STATUS BAR -->
<div class="mt-6 text-sm flex items-center justify-end gap-4 w-full">
  <span class="text-gray-500">
    Last Refresh: {lastRefreshTime}
  </span>
  <div>
    <span
      class="font-medium {connectionStatus === 'Connected'
        ? 'text-green-600'
        : 'text-amber-600'}"
    >
      {connectionStatus}
    </span>
    {#if telemetryError}
      <span class="text-red-500 ml-1">({telemetryError})</span>
    {/if}
  </div>
</div>
