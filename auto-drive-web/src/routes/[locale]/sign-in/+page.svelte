<script lang="ts">
  import type { Locale } from "$lib/Locale";
  import Footer from "$lib/components/Footer.svelte";
  import Header from "$lib/components/Header.svelte";
  import InnerHeader from "$lib/components/InnerHeader.svelte";
  import Auth from "$lib/components/Auth.svelte";

  let { locale }: { locale: Locale } = $props();
</script>

<Header {locale} />
<section>
  <div class="section-inner main relative">
    <InnerHeader>Sign in</InnerHeader>
    <div class="px-4 py-4">
      <Auth />
    </div>
  </div>
</section>
<Footer />
