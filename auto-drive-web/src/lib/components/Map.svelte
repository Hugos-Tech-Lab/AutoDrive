<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as PIXI from 'pixi.js';
  import { Compass } from './dashboard/compass';
  import { createPermanentRoom } from './dashboard/permanentRoom';
  import { createBoundaries001 } from './dashboard/boundaries001';
  import { createHardware001 } from './dashboard/hardware001';
  import { createGridLayer } from './dashboard/uiGridLayer'; // <-- Import the new module

  let container: HTMLDivElement;
  let app: PIXI.Application;

  onMount(async () => {
    app = new PIXI.Application();

    const widthPixels = 800;
    const heightPixels = 533;

    await app.init({
      width: widthPixels,
      height: heightPixels,
      background: '#ffffff',
      antialias: true
    });

    if (container) {
      container.appendChild(app.canvas);
    }

    const pixelsPerMeter = 266;
    const bottomLeftXCm = 270;
    const bottomLeftYCm = 100;
    
    const worldContainer = new PIXI.Container();
    const scaleFactor = pixelsPerMeter / 100;

    worldContainer.x = -bottomLeftXCm * scaleFactor;
    worldContainer.y = heightPixels + (bottomLeftYCm * scaleFactor);

    worldContainer.scale.set(
        scaleFactor, 
        -scaleFactor
    );

    const permanentRoom = createPermanentRoom();
    const boundaries = createBoundaries001();
    const hardware = createHardware001();
    hardware.x = 400;
    hardware.y = 200;

    worldContainer.addChild(permanentRoom);
    worldContainer.addChild(boundaries);
    worldContainer.addChild(hardware);

    app.stage.addChild(worldContainer);

    const gridLayer = createGridLayer({
      widthPixels,
      heightPixels,
      pixelsPerMeter,
      bottomLeftXCm,
      bottomLeftYCm,
      worldContainer
    });

    app.stage.addChild(gridLayer);

    const compass = new Compass();
    compass.position.set(50, heightPixels - 50);
    compass.scale.set(0.5);
    app.stage.addChild(compass);
  });

  onDestroy(() => {
    if (app) {
      app.destroy(true, { children: true });
    }
  });
</script>

<div class="pixi-wrapper">
  <div bind:this={container} class="canvas-container"></div>
</div>

<style>
  .pixi-wrapper {
    font-family: sans-serif;
    color: #e5e7eb;
    border: 3px black solid;
    width: fit-content;
  }

  .canvas-container {
    overflow: hidden;
  }
</style>