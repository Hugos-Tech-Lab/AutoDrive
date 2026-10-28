<script lang="ts">
  import type { Locale } from "$lib/Locale";
  import Footer from "$lib/components/Footer.svelte";
  import Header from "$lib/components/Header.svelte";
  import Profile from "$lib/components/Profile.svelte";

  let { locale }: { locale: Locale } = $props();
</script>

<Header {locale} />
<section>
  <div class="section-inner main relative">
    <div class="px-4 py-4">
    <h1 class="text-4xl mb-4 min-w-0 flex-1">Dashboard</h1>
      <Profile />
    </div>
  </div>
</section>
<Footer />
