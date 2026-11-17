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

  const views: View[] = ["Time Trials", "Scripts", "Vehicles"];
</script>

<div class="px-4 pt-4 flex items-start gap-8">
  <h1 class="text-4xl mb-4 min-w-0 flex-1">Dashboard</h1>

  <div class="flex flex-col items-end gap-2 shrink-0">
    <UserAccountButton authState={session} />
  </div>
</div>

<div class="row">
  {#each views as view}
    <button
      type="button"
      class="button block"
      class:selected={selectedView === view}
      aria-pressed={selectedView === view}
      onclick={() => (selectedView = view)}
    >
      {view}
    </button>
  {/each}
</div>

{#if selectedView === "Time Trials"}
  <TimeTrials />
{/if}

{#if selectedView === "Scripts"}
  <!-- <Scripts /> -->
{/if}

{#if selectedView === "Vehicles"}
  <Vehicles />
{/if}

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
