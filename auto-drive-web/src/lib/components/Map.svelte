<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as PIXI from 'pixi.js';
  import { Compass } from './dashboard/compass';
  import { createPermanentRoom } from './dashboard/permanentRoom';
  import { createBoundaries001 } from './dashboard/boundaries001';

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

    const mapStartBottomLeftXMeters = 3;
    const mapStartBottomLeftYMeters = 1;

    const bottomLeftXCm = mapStartBottomLeftXMeters * 100;
    const bottomLeftYCm = mapStartBottomLeftYMeters * 100;

    
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

    worldContainer.addChild(permanentRoom);
    worldContainer.addChild(boundaries);

    app.stage.addChild(worldContainer);

    const gridLayer = new PIXI.Graphics();
    gridLayer.moveTo(0, heightPixels);
    gridLayer.lineTo(0 + widthPixels, heightPixels);
    gridLayer.stroke({ width: 2, color: 0x4b5563 });

    gridLayer.moveTo(0, heightPixels);
    gridLayer.lineTo(0, heightPixels - heightPixels);
    gridLayer.stroke({ width: 2, color: 0x4b5563 });

    // Dynamic X-Axis Labels & Ticks
    for (let i = 1; i <= 10; i++) {
      const xPos = 0 + (i * pixelsPerMeter);
      
      gridLayer.moveTo(xPos, heightPixels);
      gridLayer.lineTo(xPos, heightPixels - 8);
      gridLayer.stroke({ width: 1, color: 0x4b5563 });

      const label = new PIXI.Text({
        text: `${mapStartBottomLeftXMeters + i}m`,
        style: {
          fontFamily: 'sans-serif',
          fontSize: 12,
          fill: 0x4b5563
        }
      });
      label.x = xPos - 10;
      label.y = heightPixels - 24;
      app.stage.addChild(label);
    }

    // Dynamic Y-Axis Labels & Ticks
    for (let i = 1; i <= 10; i++) {
      const yPos = heightPixels - (i * pixelsPerMeter);
      
      gridLayer.moveTo(0, yPos);
      gridLayer.lineTo(0 + 8, yPos);
      gridLayer.stroke({ width: 1, color: 0x4b5563 });

      const label = new PIXI.Text({
        text: `${mapStartBottomLeftYMeters + i}m`,
        style: {
          fontFamily: 'sans-serif',
          fontSize: 12,
          fill: 0x4b5563
        }
      });
      label.x = 0 + 12;
      label.y = yPos - 8;
      app.stage.addChild(label);
    }

    app.stage.addChild(gridLayer);

    const step = new PIXI.Graphics();

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