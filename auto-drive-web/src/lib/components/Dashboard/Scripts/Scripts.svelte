<script lang="ts">
  import LiveScripts from "./LiveScripts.svelte";

  type SideOption = "Live";
  let selectedSideOption = $state<SideOption>("Live");

  const sideOptions: SideOption[] = ["Live"];
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
  {#if selectedSideOption === "Live"}
    <LiveScripts />
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
