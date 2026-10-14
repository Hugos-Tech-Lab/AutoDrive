
<script lang="ts">
  import TheBeginnerCar from './TheBeginnerCarDrawing.svg?raw';

  let {
    IRAValue = 0
  }: {
    IRAValue?: number;
  } = $props();

  let windowColor = $state('#87ceeb');

  // Clamp the sensor value to the range 0–4095.
  let normalizedValue = $derived(
    Math.max(0, Math.min(2048, IRAValue)) / 2048
  );

  // 0 => white (#ffffff)
  // 4095 => black (#000000)
  let bodyColor = $derived(
    `rgb(${Math.round(255 * (1 - normalizedValue))}, ${Math.round(
      255 * (1 - normalizedValue)
    )}, ${Math.round(255 * (1 - normalizedValue))})`
  );

</script>

<div
  class="car"
  style:--body-color={bodyColor}
  style:--window-color={windowColor}
>
  {@html TheBeginnerCar}

  <div class="telemetry">
    <span>IR Sensor A: {IRAValue}</span>
    <span>{IRAValue} {Math.round(normalizedValue * 100)}% darkness</span>
  </div>
</div>

<style>
  .car :global(.car-body) {
    fill: var(--body-color);
  }

  .car :global(.car-windows) {
    fill: var(--window-color);
  }

  .car :global(#wheel-left) {
    fill: var(--body-color);
  }

  .telemetry {
    display: flex;
    justify-content: space-between;
    gap: 1rem;
    margin-top: 0.5rem;
    font-size: 0.875rem;
  }
</style>
