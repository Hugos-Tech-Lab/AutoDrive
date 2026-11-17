<script lang="ts">
  import VehiclesList from "./LiveVehiclesSubmenu.svelte";
  import LiveVehicleStatus from "./LiveVehicleStatus.svelte";

  let selectedVehicleNumberPlate = $state<string>("");

  type SideOption = "Status" | "Logs" | "Position" | "Telemetry" | "Control";
  let selectedSideOption = $state<SideOption>("Status");

  const sideOptions: SideOption[] = [
    "Status",
    "Logs",
    "Position",
    "Telemetry",
    "Control",
  ];
</script>

<VehiclesList bind:selectedVehicleNumberPlate />

{#if selectedVehicleNumberPlate !== ""}
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
    {#if selectedSideOption === "Status"}
      <LiveVehicleStatus vehicle_number_plate={selectedVehicleNumberPlate} />
      <!-- <h2 class="text-2xl mb-4">No leaderboard yet.</h2> -->
    {/if}
  </div>
{/if}

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
