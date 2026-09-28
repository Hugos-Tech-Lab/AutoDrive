<script lang="ts">
  import { type Locale } from "$lib/Locale";
  import Header from "$lib/components/Header.svelte";
  import Footer from "$lib/components/Footer.svelte";
  import type { PageData } from "./$types";
  import InnerHeader from "$lib/components/InnerHeader.svelte";
  import { supabase } from "$lib/supabaseClient";
  import { onMount } from "svelte";
  import type { AuthSession } from "@supabase/supabase-js";
  import LandingPage from "$lib/components/LandingPage.svelte";
  import { extractErrorFromURL, type SomeError } from "$lib/getErrorFromURL";
  import { getUsername } from "$lib/getUsername";

  let { data }: { data: PageData } = $props();

  let loading = $state(true);
  let username = $state<string | null>(null);
  let error = $state<SomeError | null>(null);
  let session = $state<AuthSession | null>(null);
  onMount(() => {
    error = extractErrorFromURL();

    supabase.auth.getSession().then(({ data }) => {
      session = data.session;
      loading = false;
    });

    const {
      data: { subscription },
    } = supabase.auth.onAuthStateChange((_event, newSession) => {
      session = newSession;
      loading = false;
    });

    return () => subscription.unsubscribe();
  });
  
  $effect(() => {
    if (session && username === null) {
      (async () => {
        loading = true;
        username = await getUsername(session.user);
        loading = false;
      })();
    }
  });
</script>

<Header locale={data.locale as Locale} />

<section>
  <div class="section-inner main relative">
    {#if session}
      <InnerHeader username={username}>Dashboard</InnerHeader>
      <div class="px-4 py-4">
          <h2>Location</h2>
          <button>Hugo's Tech Lab</button>
      </div>
    {:else if !loading}
      <LandingPage {username} />
    {/if}

  </div>
</section>

<Footer />
