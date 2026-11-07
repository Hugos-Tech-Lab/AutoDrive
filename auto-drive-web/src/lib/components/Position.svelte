<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as PIXI from 'pixi.js';
  import { Compass } from './dashboard/compass';
  import { createTileLayer } from './dashboard/tiles';

  let container: HTMLDivElement;
  let app: PIXI.Application;

  onMount(async () => {
    app = new PIXI.Application();

    let widthPixels = 800;
    let heightPixels = 533;

    await app.init({
      width: widthPixels,
      height: heightPixels,
      background: '#ffffff',
      antialias: true
    });

    if (container) {
      container.appendChild(app.canvas);
    }

    const originX = 0;   
    const originY = heightPixels;  
    const pixelsPerMeter = 266;
    const maxMetersX = 3;
    const maxMetersY = 2;
    const totalWidthPx = maxMetersX * pixelsPerMeter;
    const totalHeightPx = maxMetersY * pixelsPerMeter;

    // Use the separated tile layer generator
    const tileLayer = createTileLayer({
      originX,
      originY,
      pixelsPerMeter,
      maxMetersX,
      maxMetersY
    });
    app.stage.addChild(tileLayer);

    const gridLayer = new PIXI.Graphics();
    gridLayer.moveTo(originX, originY);
    gridLayer.lineTo(originX + totalWidthPx, originY);
    gridLayer.stroke({ width: 2, color: 0x4b5563 });

    gridLayer.moveTo(originX, originY);
    gridLayer.lineTo(originX, originY - totalHeightPx);
    gridLayer.stroke({ width: 2, color: 0x4b5563 });

    let yStartM = 1;
    let xStartM = 3; 
    for (let i = 1; i <= maxMetersX; i++) {
      const xPos = originX + (i * pixelsPerMeter);
      
      gridLayer.moveTo(xPos, originY);
      gridLayer.lineTo(xPos, originY - 8);
      gridLayer.stroke({ width: 1, color: 0x4b5563 });

      const label = new PIXI.Text({
        text: `${xStartM + i}m`,
        style: {
          fontFamily: 'sans-serif',
          fontSize: 12,
          fill: 0x4b5563
        }
      });
      label.x = xPos - 10;
      label.y = originY - 24;
      app.stage.addChild(label);
    }

    for (let i = 1; i <= maxMetersY; i++) {
      const yPos = originY - (i * pixelsPerMeter);
      
      gridLayer.moveTo(originX, yPos);
      gridLayer.lineTo(originX + 8, yPos);
      gridLayer.stroke({ width: 1, color: 0x4b5563 });

      const label = new PIXI.Text({
        text: `${yStartM + i}m`,
        style: {
          fontFamily: 'sans-serif',
          fontSize: 12,
          fill: 0x4b5563
        }
      });
      label.x = originX + 12;
      label.y = yPos - 8;
      app.stage.addChild(label);
    }

    app.stage.addChild(gridLayer);

    const step = new PIXI.Graphics();
    const rectX = widthPixels - (pixelsPerMeter);
    const rectY = heightPixels - (2 * pixelsPerMeter);
    const rectWidth = 1 * pixelsPerMeter;
    const rectHeight = 1 * pixelsPerMeter;

    step.rect(rectX, rectY, rectWidth, rectHeight);
    step.fill({ color: 0x636363, alpha: 0.4 });
    step.stroke({ width: 4, color: 0x818cf8 });
    app.stage.addChild(step);

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