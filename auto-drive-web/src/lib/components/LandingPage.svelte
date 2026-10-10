<script lang="ts">
  import { goto } from "$app/navigation";
  import InnerHeader from "./InnerHeader.svelte";

  // let { username }: { username: string | null } = $props();
</script>

<div class="px-4 pt-4 flex items-start gap-8">
  <h1 class="text-4xl mb-4 min-w-0 flex-1">
    THINK YOU CAN CODE? <u>Prove</u> it
  </h1>

  <div class="flex flex-col items-end gap-2 shrink-0">
    <button onclick={() => goto("/dashboard")}>DASHBOARD</button>
  </div>
</div>

<div class="px-4 py-4">
  <h1 class="text-2xl mb-4 min-w-0 flex-1">
    Control a real small machine remotely
  </h1>
  <div class="video">
    <iframe
      src="https://www.youtube.com/embed/rFZHOHl-L8A?autoplay=1&mute=1&playsinline=1&controls=0&disablekb=1&modestbranding=1"
      title="YouTube video player"
      allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
      referrerpolicy="strict-origin-when-cross-origin"
      allowfullscreen
    ></iframe>
  </div>
  <br />
  <h1 class="text-2xl mb-4 min-w-0 flex-1">Leaderboard</h1>

  <div class="grid grid-cols-2 sm:grid-cols-3 md:grid-cols-4 gap-3">
    <div class="border-2 border-black p-[10px] rounded-sm">
      <div class="text-2xl font-bold">1</div>
      <div>Hugo</div>
      <div>5.233 seconds</div>
    </div>
  </div>
  <br />

  <h1 class="text-2xl mb-4 min-w-0 flex-1">The Machine</h1>
  - Top Speed: 15cm / second (placeholder)<br />
  - 0 - 100(cm): 5 seconds (placeholder)<br />
  - Power: 200 mA (placeholder)<br />
  - Weight: 500 grams (placeholder)<br />
  - Steering: Skid <br />
  - Wheels: 2 powered, 1 non-powered <br />
  - Inputs: 4 light sensors (placeholder)<br />
  - Outputs: 2 motor speeds (placeholder)<br /><br />
  <h1 class="text-2xl mb-4 min-w-0 flex-1">The Track</h1>
  <!-- <h2 class="mb-6">
    Manage software on real physical hardware devices in the lab.
  </h2>

  <h2>
    Source code of this site here: <a
      href="https://github.com/Hugos-Tech-Lab/Remote-Firmware-Manager"
      >https://github.com/Hugos-Tech-Lab/Remote-Firmware-Manager
    </a>
  </h2> -->
</div>

<style>
  .video {
    width: 100%;
    aspect-ratio: 16 / 9;
  }

  .video iframe {
    width: 100%;
    height: 100%;
    border: 0;
  }
</style>
