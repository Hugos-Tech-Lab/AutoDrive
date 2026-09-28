<script lang="ts">
  import InnerHeader from "./InnerHeader.svelte";

  let {
    username,
  }: { username: string | null} = $props();
</script>

<InnerHeader {username}>Remote Firmware Manager</InnerHeader>
<div class="px-4 py-4">
  <h2 class="mb-6">
    Manage software on real physical hardware devices in the lab.
  </h2>

  <h2>
    Source code of this site here: <a
      href="https://github.com/Hugos-Tech-Lab/Remote-Firmware-Manager"
      >https://github.com/Hugos-Tech-Lab/Remote-Firmware-Manager
    </a>
  </h2>
</div>
