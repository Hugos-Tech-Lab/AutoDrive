<script lang="ts">
  import type { Locale } from "$lib/Locale";
  import Logo from "$lib/components/Logo.svelte";

  let { locale }: { locale: Locale } = $props();
</script>

<header>
  <div class="flex items-center justify-between section-inner py-6">
    <Logo />
    <a href="/">
      <h1 class="text-right">Remote Firmware Manager</h1>
    </a>
  </div>
</header>
