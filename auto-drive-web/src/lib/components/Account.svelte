<script lang="ts">
  import { onMount } from "svelte";
  import type { AuthSession } from "@supabase/supabase-js";
  import { supabase } from "$lib/supabaseClient";

  interface Props {
    session: AuthSession;
  }

  // let { session }: Props = $props();

  let loading = $state(false);
  let username = $state<string | null>(null);
  let website = $state<string | null>(null);
  let avatarUrl = $state<string | null>(null);

  const updateProfile = async () => {
    try {
      loading = true;
      // const { user } = session;

      const updates = {
        // id: user.id,
        username,
        website,
        avatar_url: avatarUrl,
        updated_at: new Date().toISOString(),
      };

      const { error } = await supabase.from("profiles").upsert(updates);

      if (error) {
        throw error;
      }
    } catch (error) {
      if (error instanceof Error) {
        alert(error.message);
      }
    } finally {
      loading = false;
    }
  };
</script>

<form
  onsubmit={(e) => {
    e.preventDefault();
    updateProfile();
  }}
  class="form-widget"
>
  <!-- <div>Email: {session.user.email}</div> -->
  <div>
    // ...

    <label for="username">Name</label>
    <input id="username" type="text" bind:value={username} />
  </div>
  <div>
    <label for="website">Website</label>
    <input id="website" type="text" bind:value={website} />
  </div>
  <div>
    <button type="submit" class="button primary block" disabled={loading}>
      {loading ? "Saving ..." : "Update profile"}
    </button>
  </div>
  <button type="button" class="button block">Leaderboard</button>
  <button type="button" class="button block">My Algorithms</button>
  <button type="button" class="button block">Real Time Play</button>
  <button type="button" class="button block">Live Debug</button>

  <button
    type="button"
    class="button block"
    onclick={() => supabase.auth.signOut()}
  >
    Sign Out
  </button>
</form>
