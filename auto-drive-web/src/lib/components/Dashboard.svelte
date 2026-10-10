<script lang="ts">
  import { goto } from "$app/navigation";

  let {} = $props();
  type Build = "A" | "B" | "C";

  let selectedBuild = $state<Build>("A");

  const builds: Build[] = ["A", "B", "C"];
</script>

<div class="px-4 pt-4 flex items-start gap-8">
  <h1 class="text-4xl mb-4 min-w-0 flex-1">Dashboard</h1>

  <div class="flex flex-col items-end gap-2 shrink-0">
    <button onclick={() => goto("/dashboard")}>Sign In / Sign up</button>
  </div>
</div>

<div class="row">
  <button type="button" class="button block">The Beginner Car (dropdown)</button
  >
  {#each builds as build}
    <button
      type="button"
      class="button block"
      class:selected={selectedBuild === build}
      aria-pressed={selectedBuild === build}
      onclick={() => (selectedBuild = build)}
    >
      Build {build}
    </button>
  {/each}
  <!-- <button type="button" class="button block">Unit D</button> -->
  <!-- <button type="button" class="button block hugo">Sign In / Sign up</button> -->
</div>
<!-- <button type="button" class="button block">Calendar</button> -->
<button type="button" class="button block"
  >Software Development Kit (SDK)</button
>
<button type="button" class="button block">Leaderboard</button>
<button type="button" class="button block">My Algorithms</button>
<button type="button" class="button block">Live Debug</button>

<button type="button" class="button block"
  >Status (shown in selected Build only) (shows firmware, last online, etc)</button
>
<button type="button" class="button block"
  >Sensors (shown in selected Build only)</button
>
<button type="button" class="button block"
  >Upcoming (shown in selected Build only)</button
>
<button type="button" class="button block"
  >Past (shown in selected Build only)</button
>
<button type="button" class="button block"
  >Logs (shown in selected Build only)</button
>

<!-- <button type="button" class="button block">Live Control (shown in selected Build only)</button> -->

<style>
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
