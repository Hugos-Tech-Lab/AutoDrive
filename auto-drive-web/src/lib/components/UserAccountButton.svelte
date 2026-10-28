<script lang="ts">
  import { goto } from "$app/navigation";
  import type { AuthState } from "$lib/auth.svelte";

  let { authState }: { authState: AuthState } = $props();
</script>

{#if authState.status === 'LOADING'}
  <button disabled class="opacity-50 cursor-not-allowed">Loading...</button>
{:else if authState.status === 'AUTHENTICATED'}
  <button onclick={() => goto("/profile")}>
    {authState.username ?? "Profile"}
  </button>
{:else}
  <button onclick={() => goto("/create-account")}>Create Account</button>
  <button onclick={() => goto("/sign-in")}>Sign In</button>
{/if}
