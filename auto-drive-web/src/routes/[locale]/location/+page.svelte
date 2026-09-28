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
    <InnerHeader>Firmware</InnerHeader>
    <div class="px-4 py-4">
        <h2>Location</h2>
        <button>Hugo's Tech Lab</button>
        <h2>Device</h2>
        <button>Car 1</button>
        <button>Car 2</button>
        <h2>Overview</h2>
        <h2>Status: Online / Offline</h2>
        <h2>Model: </h2>
        <h2>Firmware Time: Online / Offline</h2>
        <h2>Firmware Hash: Online / Offline</h2>
        <h2>Firmware Name: Online / Offline</h2>

    </div>
  </div>
</section>
<Footer />
