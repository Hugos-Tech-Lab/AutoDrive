<script lang="ts">
  import { onMount } from 'svelte';
  import Header from "$lib/components/Header.svelte";
  import Footer from "$lib/components/Footer.svelte";
  import Dashboard from '$lib/components/Dashboard.svelte';
  import { createAuthState } from '$lib/auth.svelte';
  import type { Locale } from "$lib/Locale";
  import type { PageData } from "./$types";

  let { data }: { data: PageData } = $props();

  const auth = createAuthState();

  onMount(() => auth.destroy);
</script>

<Header locale={data.locale as Locale} />

<section>
  <div class="section-inner main relative">
    <Dashboard session={auth.current} />
  </div>
</section>

<Footer />
