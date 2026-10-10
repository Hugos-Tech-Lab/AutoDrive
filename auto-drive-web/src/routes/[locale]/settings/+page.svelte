<script lang="ts">
  import { onMount } from 'svelte'
  import { goto } from "$app/navigation";
  import { type Locale } from "$lib/Locale";
  import { supabase } from '$lib/supabaseClient'
  import Header from "$lib/components/Header.svelte";
  import Footer from "$lib/components/Footer.svelte";
  import type { PageData } from "./$types";
  import type { AuthSession } from '@supabase/supabase-js'
  import Translations from "$lib/components/Translations.svelte";
  import InnerHeader from "$lib/components//InnerHeader.svelte";
  import Account from "$lib/components/Account.svelte";

  let session = $state<AuthSession | null>(null)
  onMount(() => {
    supabase.auth.getSession().then(({ data }) => {
      session = data.session
    })
    supabase.auth.onAuthStateChange((_event, _session) => {
      session = _session
    })
  })

  let { data }: { data: PageData } = $props();
</script>

<Header locale={data.locale as Locale} />

<section>
  <div class="section-inner main relative">
    <InnerHeader>Settings</InnerHeader>
    <Translations locale = {data.locale as Locale} />

    {#if !session}
     <h1>Sign In / Sign up</h1>
    {:else}
    <Account {session} />
    {/if}
  </div>
</section>

<Footer />
