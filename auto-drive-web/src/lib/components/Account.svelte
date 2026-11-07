<script lang="ts">
  import { onMount } from "svelte";
  import type { AuthSession } from "@supabase/supabase-js";
  import { supabase } from "$lib/supabaseClient";
  import { goto } from "$app/navigation";

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

  const handleSignOut = async () => {
    await supabase.auth.signOut();
    goto("/dashboard");
  };
</script>
