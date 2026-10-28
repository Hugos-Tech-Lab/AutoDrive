<script lang="ts">
  import { goto } from "$app/navigation";
  import HardwareTelemetry from "./HardwareTelemetry.svelte";
  import LiveTimeTrials from "./LiveTimeTrials.svelte";
  import LiveHardwareBanner from "./LiveHardwareBanner.svelte";
  import TheBeginnerCar from "./TheBeginnerCarDrawing.svelte";
  import UserAccountButton from "./UserAccountButton.svelte";
  import type { AuthState } from "$lib/auth.svelte";

  interface Props {
    session: AuthState;
  }

  let { session }: Props = $props();
  let userName = $state<string | undefined>();
  let selectedBuild = $state<string>("Time Trials");

  type PlayOption = "Leaderboard" | "Algorithms" | "Live";
  let selectedPlayOption = $state<PlayOption>("Leaderboard");

  const playOptions: PlayOption[] = [
    "Leaderboard",
    "Algorithms",
    "Live"
  ];

  type BuildOption = "Status" | "Telemetry" | "Control";
  let selectedBuildOption = $state<BuildOption>("Status");

  const buildOptions: BuildOption[] = [
    "Status",
    "Telemetry",
    "Control",
  ];
</script>

<div class="px-4 pt-4 flex items-start gap-8">
  <h1 class="text-4xl mb-4 min-w-0 flex-1">Dashboard</h1>

  <div class="flex flex-col items-end gap-2 shrink-0">
    <UserAccountButton authState={session} />
  </div>
</div>

<!-- <button type="button" class="button block">The Beginner Car</button> -->

<LiveHardwareBanner bind:selectedBuild />

{#if selectedBuild === "Time Trials"}
  <div class="button-container">
    {#each playOptions as playOption}
      <button
        type="button"
        class="button block"
        class:selected={selectedPlayOption === playOption}
        aria-pressed={selectedPlayOption === playOption}
        onclick={() => (selectedPlayOption = playOption)}
      >
        {playOption}
      </button>
    {/each}
  </div>
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
{/if}

{#if selectedBuild !== "Time Trials" && selectedBuildOption === "Control"}
  <h2 class="text-2xl mb-4 min-w-0 flex-1">Car Control: {selectedBuild}</h2>

  <div class="row">
    <button type="button" class="button block">STOP</button>
    <button type="button" class="button block">Run Script</button>
    <button type="button" class="button block">Navigate To Start</button>
  </div>
{/if}

{#if selectedBuild === "Time Trials" && selectedPlayOption === "Leaderboard"}
  <h2 class="text-2xl mb-4 min-w-0 flex-1">No leaderboard yet.</h2>
{/if}

{#if selectedBuild === "Time Trials" && selectedPlayOption === "Algorithms"}
  <h2 class="text-2xl mb-4 min-w-0 flex-1">No algorithms yet.</h2>
{/if}

{#if selectedBuild !== "Time Trials" && selectedBuildOption === "Telemetry"}
  <HardwareTelemetry />
{/if}

{#if selectedBuild === "Time Trials" && selectedPlayOption === "Live"}
  <LiveTimeTrials />
{/if}

<style>
  .button-container {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    position: absolute;
    /* THIS MUST MATCH: THE MAIN WIDTH */
    right: calc(50% + 30rem);
  }

  .row {
    display: flex;
    gap: 0.5rem;
  }

  .row .button {
    width: auto;
  }

  .button.selected {
    background-color: #a4aeff;
  }
</style>