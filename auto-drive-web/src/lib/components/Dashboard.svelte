<script lang="ts">
  import { goto } from "$app/navigation";

  let {} = $props();
  type Build = "Overview" | "Build A" | "Build B";
  let selectedBuild = $state<Build>("Overview");

  const builds: Build[] = ["Overview", "Build A", "Build B"];

  type PlayOption = "Leaderboard" | "Algorithms";
  let selectedPlayOption = $state<PlayOption>("Leaderboard");

  const playOptions: PlayOption[] = [
    "Leaderboard",
    "Algorithms",
  ];

  type BuildOption = "Status" | "Sensors" | "Control" | "Cameras";
  let selectedBuildOption = $state<BuildOption>("Status");

  const buildOptions: BuildOption[] = [
    "Status",
    "Sensors",
    "Control",
    "Cameras"
  ];
</script>

<div class="px-4 pt-4 flex items-start gap-8">
  <h1 class="text-4xl mb-4 min-w-0 flex-1">Dashboard</h1>

  <div class="flex flex-col items-end gap-2 shrink-0">
    <button onclick={() => goto("/dashboard")}>Sign In / Sign up</button>
  </div>
</div>
<button type="button" class="button block">The Beginner Car</button>
<div class="row">
  {#each builds as build}
    <button
      type="button"
      class="button block"
      class:selected={selectedBuild === build}
      aria-pressed={selectedBuild === build}
      onclick={() => (selectedBuild = build)}
    >
      {build}
    </button>
  {/each}
  <!-- <button type="button" class="button block">Unit D</button> -->
  <!-- <button type="button" class="button block hugo">Sign In / Sign up</button> -->
</div>
<!-- <button type="button" class="button block">Calendar</button> -->
<!-- <button type="button" class="button block"
  >Software Development Kit (SDK)</button
> -->

{#if selectedBuild === "Overview"}
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
    <!-- 
    <button type="button" class="button block">Leaderboard</button>
    <button type="button" class="button block">Algorithms</button>
    <button type="button" class="button block">Time Trials</button> -->
    <!-- <button type="button" class="button block">Experiment</button> -->
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

    <!-- <button type="button" class="button block">Status</button>
    <button type="button" class="button block">Sensors</button>
    <button type="button" class="button block">Upcoming Runs</button>
    <button type="button" class="button block">Past Runs</button>
    <button type="button" class="button block">Logs</button> -->
  </div>
{/if}

{#if selectedBuild.startsWith("Build") && selectedBuildOption === "Control"}
  <h2 class="text-2xl mb-4 min-w-0 flex-1">Car Control</h2>
  <!-- <button type="button" class="button block">                      ___</button> -->

  <div class="row">
    <button type="button" class="button block">STOP</button>
    <button type="button" class="button block">Run Script</button>
    <button type="button" class="button block">Navigate To Start</button>
  </div>
{/if}

<!-- <h2 class="text-2xl mb-4 min-w-0 flex-1">Software Development Kit</h2>
<h2 class="mb-4 min-w-0 flex-1">
  The SDK allows you to control the car from a programming language
</h2>

Rust:<button type="button" class="button block">Rust Example</button>
<button type="button" class="button block">Rust Library</button>

API Token
<h2 class="text-2xl mb-4 min-w-0 flex-1">Create new one</h2> -->

<!-- <div class="row">
  <button type="button" class="button block">Yesterday</button>
  <button type="button" class="button block">Today (10.10.2026)</button>
  <button type="button" class="button block">Tomorrow</button>
</div>

<button type="button" class="button block">8am - 8:05am: Sem</button>
<button type="button" class="button block">8.05am - 8:10am: Sem</button>
<button type="button" class="button block">8.10am - 8:15am: Sem</button> -->

<!-- <button type="button" class="button block">Live Control (shown in selected Build only)</button> -->

<style>
  /* TODO: this is a bit dangerous if width is small. Do something else if width is small  */
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

  /* .hugo {
    margin-left: auto;
  } */
</style>
