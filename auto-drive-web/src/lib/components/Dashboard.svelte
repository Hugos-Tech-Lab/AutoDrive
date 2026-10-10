<script lang="ts">
  import { goto } from "$app/navigation";

  let {} = $props();
  type Build = "Any" | "Build A" | "Build B";

  let selectedBuild = $state<Build>("Any");

  const builds: Build[] = ["Any", "Build A", "Build B"];
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

{#if selectedBuild === "Any"}
<div class="button-container">
    <button type="button" class="button block">Leaderboard</button>
    <button type="button" class="button block">Algorithms</button>
    <button type="button" class="button block">Calendar</button>
    <button type="button" class="button block">Play</button>
</div>
{:else}
<div class="button-container">
  <button type="button" class="button block"
    >Status</button
  >
  <button type="button" class="button block"
    >Sensors</button
  >
  <button type="button" class="button block"
    >Upcoming Runs</button
  >
  <button type="button" class="button block"
    >Past Runs</button
  >
  <button type="button" class="button block"
    >Logs</button 
  >
</div>

{/if}


<!-- <button type="button" class="button block">Live Control (shown in selected Build only)</button> -->

<style>
  /* TODO: this is a bit dangerous if width is small. Do something else if width is small  */
  .button-container {
      display: flex;
      flex-direction: column;
      align-items: flex-end;
    position: fixed;
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

  .row .button.selected {
    background-color: #a4aeff;
  }

  /* .hugo {
    margin-left: auto;
  } */
</style>
