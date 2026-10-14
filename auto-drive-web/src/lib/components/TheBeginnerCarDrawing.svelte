<script lang="ts">
  import TheBeginnerCar from './TheBeginnerCarDrawing.svg?raw';

  let bodyColor = $state('#ff0000');
  let windowColor = $state('#87ceeb');
</script>

<div
  class="car"
  style:--body-color={bodyColor}
  style:--window-color={windowColor}
>
  {@html TheBeginnerCar}
</div>

<style>
  .car :global(.car-body) {
    fill: var(--body-color);
  }

  .car :global(.car-windows) {
    fill: var(--window-color);
  }

  .car :global(#wheel-left) {
    fill: red;
  }
</style>
