<script lang="ts">
  import UserAccountButton from "../UserAccountButton.svelte";
  import type { AuthState } from "$lib/auth.svelte";
  import TimeTrials from "./TimeTrials/TimeTrials.svelte";
  import Vehicles from "./Vehicles/Vehicles.svelte";

  interface Props {
    session: AuthState;
  }

  let { session }: Props = $props();
  let userName = $state<string | undefined>();
  type View = "Time Trials" | "Scripts" | "Vehicles";
  let selectedView = $state<View>("Time Trials");
</script>

<div class="px-4 pt-4 flex items-start gap-8">
  <h1 class="text-4xl mb-4 min-w-0 flex-1">Dashboard</h1>

  <div class="flex flex-col items-end gap-2 shrink-0">
    <UserAccountButton authState={session} />
  </div>
</div>

<!-- <button type="button" class="button block">The Beginner Car</button> -->

<div class="row">
  <button
    type="button"
    class="button block"
    class:selected={selectedView === "Time Trials"}
    aria-pressed={selectedView === "Time Trials"}
    onclick={() => (selectedView = "Time Trials")}
  >
    Time Trials
  </button>

  <button
    type="button"
    class="button block"
    class:selected={selectedView === "Scripts"}
    aria-pressed={selectedView === "Scripts"}
    onclick={() => (selectedView = "Scripts")}
  >
    Scripts
  </button>

  <button
    type="button"
    class="button block"
    class:selected={selectedView === "Vehicles"}
    aria-pressed={selectedView === "Vehicles"}
    onclick={() => (selectedView = "Vehicles")}
  >
    Vehicles
  </button>
</div>

<!-- {#if selectedView === "Time Trials"}
 
{:else}
  <div class="button-container">
    {#each buildOptions as buildOption}
      <button
        type="button"
        class="button block"
        class:selected={selectedBuildOption === buildOption}
        aria-pressed={selectedBuildOption === buildOption}
        onclick={() => (selectedBuildOption = buildOption)}
      >
        {buildOption}
      </button>
    {/each}
  </div>
{/if} -->

<!-- {#if selectedView !== "Time Trials" && selectedBuildOption === "Control"}
  <h2 class="text-2xl mb-4 min-w-0 flex-1">Car Control: {selectedView}</h2>

  <div class="row">
    <button type="button" class="button block">STOP</button>
    <button type="button" class="button block">Run Script</button>
    <button type="button" class="button block">Navigate To Start</button>
  </div>
{/if} -->
<!-- 
{#if selectedView !== "Time Trials" && selectedBuildOption === "Status"}
  <Status />
{/if}

{#if selectedView !== "Time Trials" && selectedBuildOption === "Position"}
  <Position />
{/if}

{#if selectedView !== "Time Trials" && selectedBuildOption === "Telemetry"}
  <HardwareTelemetry />
{/if} -->

{#if selectedView === "Time Trials"}
  <TimeTrials />
{/if}

{#if selectedView === "Scripts"}
  <!-- <Scripts /> -->
{/if}

{#if selectedView === "Vehicles"}
  <Vehicles />
{/if}

<!-- {#if selectedView === "Time Trials" && selectedPlayOption === "Live"}
  <LiveTimeTrials />
{/if} -->

<style>
  .row {
    display: flex;
  }

  .row .button {
    width: auto;
  }

  .button.selected {
    background-color: #a4aeff;
  }
</style>
