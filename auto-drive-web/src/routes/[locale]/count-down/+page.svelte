<script lang="ts">
  import { onMount, onDestroy } from 'svelte';

  let remainingSeconds = $state(0);
  let isLive = $state(false);
  let timer: ReturnType<typeof setInterval>;

  function updateCountdown() {
    const now = new Date();

    const formatter = new Intl.DateTimeFormat('en-US', {
      timeZone: 'Europe/Paris',
      year: 'numeric',
      month: 'numeric',
      day: 'numeric',
      hour: 'numeric',
      minute: 'numeric',
      second: 'numeric',
      hour12: false
    });

    const parts = formatter.formatToParts(now);
    const p: Record<string, number> = {};
    parts.forEach(({ type, value }) => {
      if (type !== 'literal') p[type] = parseInt(value, 10);
    });

    const currentCET = new Date(p.year, p.month - 1, p.day, p.hour === 24 ? 0 : p.hour, p.minute, p.second);
    const targetCET = new Date(p.year, p.month - 1, p.day, 20, 27, 0, 0);

    const diffInSeconds = Math.floor((targetCET.getTime() - currentCET.getTime()) / 1000);

    if (diffInSeconds <= 0) {
      isLive = true;
      remainingSeconds = 0;
    } else {
      isLive = false;
      remainingSeconds = diffInSeconds;
    }
  }

  onMount(() => {
    updateCountdown();
    timer = setInterval(updateCountdown, 1000);
  });

  onDestroy(() => {
    if (timer) clearInterval(timer);
  });
</script>

<section class="h-screen w-screen flex items-center justify-center bg-[rgb(37,34,131)] text-white">
  <div class="text-center">
    {#if isLive}
      <h1 class="text-6xl md:text-8xl font-black uppercase tracking-wider">
        Stream Live
      </h1>
    {:else}
      <div class="text-8xl md:text-[16rem] font-bold font-mono tracking-tight leading-none">
        LIVE IN
        {remainingSeconds}
        SECONDS
      </div>
    {/if}
  </div>
</section>