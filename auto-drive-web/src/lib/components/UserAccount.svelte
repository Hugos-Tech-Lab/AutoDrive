<script lang="ts">
  import { goto } from "$app/navigation";

  let {
    username,
  }: { username: string | null } = $props();
</script>

{#if username}
  <button onclick={() => goto("/profile")}>{username}</button>
{:else}
  <button onclick={() => goto("/sign-in")}>DASHBOARD</button>
{/if}
