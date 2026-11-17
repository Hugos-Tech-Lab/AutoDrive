<script lang="ts">
  type SideOption = "Leaderboard" | "Algorithms";
  let selectedSideOption = $state<SideOption>("Leaderboard");

  const sideOptions: SideOption[] = ["Leaderboard", "Algorithms"];
</script>

<div class="side-options-container">
  {#each sideOptions as sideOption}
    <button
      type="button"
      class="button block"
      class:selected={selectedSideOption === sideOption}
      aria-pressed={selectedSideOption === sideOption}
      onclick={() => (selectedSideOption = sideOption)}
    >
      {sideOption}
    </button>
  {/each}
</div>

<div class="min-w-0 flex-1 p-4">
  {#if selectedSideOption === "Leaderboard"}
    <h2 class="text-2xl mb-4">No leaderboard yet.</h2>
  {/if}

  {#if selectedSideOption === "Algorithms"}
    <h2 class="text-2xl mb-4 min-w-0 flex-1">No algorithms yet.</h2>
  {/if}
</div>

<style>
  .side-options-container {
    display: flex;
    flex-direction: column;
    align-items: flex-end;
    position: absolute;
    /* THIS MUST MATCH: THE MAIN WIDTH */
    right: calc(50% + 30rem);
  }

  .button.selected {
    background-color: #a4aeff;
  }
</style>
