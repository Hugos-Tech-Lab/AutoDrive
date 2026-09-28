<script lang="ts">
  import InnerHeader from "./InnerHeader.svelte";

  let { username }: { username: string | null } = $props();
</script>

<InnerHeader {username}>THINK YOU CAN CODE? <u>Prove</u> it</InnerHeader>
<div class="px-4 py-4">
  <h1 class="text-2xl mb-4 min-w-0 flex-1">
    Control a real small car remotely
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
  <h1 class="text-2xl mb-4 min-w-0 flex-1">Leaderboard</h1>
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
