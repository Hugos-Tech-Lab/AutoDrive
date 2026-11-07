<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as PIXI from 'pixi.js';
  import { Compass } from './dashboard/compass';

  let container: HTMLDivElement;
  let app: PIXI.Application;

  onMount(async () => {
    app = new PIXI.Application();

    await app.init({
      width: 800,
      height: 600,
      background: '#ffffff',
      antialias: true
    });

    if (container) {
      container.appendChild(app.canvas);
    }

    const gridLayer = new PIXI.Graphics();
    
    const originX = 0;   
    const originY = 600;  
    const pixelsPerMeter = 200;
    const maxMetersX = 4;
    const maxMetersY = 3;

    gridLayer.moveTo(originX, originY);
    gridLayer.lineTo(originX + (maxMetersX * pixelsPerMeter), originY);
    gridLayer.stroke({ width: 2, color: 0x9ca3af });

    gridLayer.moveTo(originX, originY);
    gridLayer.lineTo(originX, originY - (maxMetersY * pixelsPerMeter));
    gridLayer.stroke({ width: 2, color: 0x9ca3af });

    for (let i = 1; i <= maxMetersX; i++) {
      const xPos = originX + (i * pixelsPerMeter);
      
      gridLayer.moveTo(xPos, originY);
      gridLayer.lineTo(xPos, originY - 8);
      gridLayer.stroke({ width: 1, color: 0x4b5563 });

      const label = new PIXI.Text({
        text: `${i}m`,
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
        text: `${i}m`,
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

    const rectangle = new PIXI.Graphics();
    const rectX = originX + (0.5 * pixelsPerMeter);
    const rectY = originY - (2.5 * pixelsPerMeter);
    const rectWidth = 1.8 * pixelsPerMeter;
    const rectHeight = 1.5 * pixelsPerMeter;

    rectangle.rect(rectX, rectY, rectWidth, rectHeight);
    rectangle.fill({ color: 0x4f46e5 }); 
    rectangle.stroke({ width: 4, color: 0x818cf8 });
    app.stage.addChild(rectangle);

    const compass = new Compass();
    compass.position.set(50, 550);
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