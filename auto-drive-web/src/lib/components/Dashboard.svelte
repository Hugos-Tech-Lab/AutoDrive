<script lang="ts">
  let {} = $props();
</script>

<div class="row">
  <button type="button" class="button block">The Beginner Car (dropdown)</button>
  <button type="button" class="button block">Build A</button>
  <button type="button" class="button block">Build B</button>
  <button type="button" class="button block">Build C</button>
  <!-- <button type="button" class="button block">Unit D</button> -->
  <button type="button" class="button block hugo">Logged Out</button>
</div>
<!-- <button type="button" class="button block">Calendar</button> -->
<button type="button" class="button block">Software Development Kit (SDK)</button>
<button type="button" class="button block">Leaderboard</button>
<button type="button" class="button block">My Algorithms</button>
<button type="button" class="button block">Live Debug</button>

<button type="button" class="button block">Status (shown in selected Build only) (shows firmware, last online, etc)</button>
<button type="button" class="button block">Sensors (shown in selected Build only)</button>
<button type="button" class="button block">Upcoming (shown in selected Build only)</button>
<button type="button" class="button block">Past (shown in selected Build only)</button>
<button type="button" class="button block">Logs (shown in selected Build only)</button>
<!-- <button type="button" class="button block">Live Control (shown in selected Build only)</button> -->

<style>
  .row {
    display: flex;
    gap: 0.5rem;
  }

  .row .button {
    width: auto;
  }

  .hugo {
    margin-left: auto;
  }
</style>