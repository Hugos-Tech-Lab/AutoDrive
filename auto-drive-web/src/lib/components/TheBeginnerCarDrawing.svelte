
<script lang="ts">
  import TheBeginnerCar from "./TheBeginnerCarDrawing.svg?raw";

  let {
    IRUtmostFrontUtmostLeft = 0,
    IRUtmostFrontUtmostRight = 0,
  }: {
    IRUtmostFrontUtmostLeft?: number;
    IRUtmostFrontUtmostRight?: number;
  } = $props();

  let windowColor = $state("#87ceeb");

  const SENSOR_MAX = 65536;

  function normalize(value: number): number {
    return Math.max(0, Math.min(SENSOR_MAX, value)) / SENSOR_MAX;
  }

  let leftNormalized = $derived(
    normalize(IRUtmostFrontUtmostLeft)
  );

  let rightNormalized = $derived(
    normalize(IRUtmostFrontUtmostRight)
  );

  let leftDarkness = $derived(
    Math.round(leftNormalized * 100)
  );

  let rightDarkness = $derived(
    Math.round(rightNormalized * 100)
  );

  // Average both sensors to determine the car body color.
  let normalizedValue = $derived(
    (leftNormalized + rightNormalized) / 2
  );

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
    <div class="sensor">
      <strong>Front Left IR</strong>
      <span>{IRUtmostFrontUtmostLeft}</span>
      <div class="meter">
        <div class="meter-fill" style:width="{leftDarkness}%"></div>
      </div>
      <span>{leftDarkness}% darkness</span>
    </div>

    <div class="sensor">
      <strong>Front Right IR</strong>
      <span>{IRUtmostFrontUtmostRight}</span>
      <div class="meter">
        <div class="meter-fill" style:width="{rightDarkness}%"></div>
      </div>
      <span>{rightDarkness}% darkness</span>
    </div>
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
    display: grid;
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 1rem;
    margin-top: 0.75rem;
    font-size: 0.875rem;
  }

  .sensor {
    display: flex;
    flex-direction: column;
    gap: 0.35rem;
    min-width: 0;
  }

  .meter {
    height: 8px;
    overflow: hidden;
    border-radius: 9999px;
    background: #e5e7eb;
  }

  .meter-fill {
    height: 100%;
    background: #22c55e;
    border-radius: inherit;
    transition: width 100ms linear;
  }

  @media (max-width: 400px) {
    .telemetry {
      grid-template-columns: 1fr;
    }
  }
</style>
