<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as PIXI from 'pixi.js';
  import { Compass } from './dashboard/compass';
  import { createTileLayer } from './dashboard/tiles';
  import { createMarker } from './dashboard/marker';
  import { createPermanentRoom } from './dashboard/permanentRoom';

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

    const markerXCm = 455;
    const markerYCm = 254.5;

    const originX = 0;   
    const originY = heightPixels;  
    const pixelsPerMeter = 266;
    
    const maxMetersX = 3;
    const maxMetersY = 2;

    const mapStartBottomLeftXMeters = 3;
    const mapStartBottomLeftYMeters = 1;
    
    const mapStartBottomLeftXCm = mapStartBottomLeftXMeters * 100;
    const mapStartBottomLeftYCm = mapStartBottomLeftYMeters * 100;

    const totalWidthPx = maxMetersX * pixelsPerMeter;
    const totalHeightPx = maxMetersY * pixelsPerMeter;

    const permanentRoom = createPermanentRoom();
    permanentRoom.scale.set(pixelsPerMeter);

    app.stage.addChild(permanentRoom);

    // 2. Grid & Axis Lines
    const gridLayer = new PIXI.Graphics();
    gridLayer.moveTo(originX, originY);
    gridLayer.lineTo(originX + totalWidthPx, originY);
    gridLayer.stroke({ width: 2, color: 0x4b5563 });

    gridLayer.moveTo(originX, originY);
    gridLayer.lineTo(originX, originY - totalHeightPx);
    gridLayer.stroke({ width: 2, color: 0x4b5563 });

    // Dynamic X-Axis Labels & Ticks
    for (let i = 1; i <= maxMetersX; i++) {
      const xPos = originX + (i * pixelsPerMeter);
      
      gridLayer.moveTo(xPos, originY);
      gridLayer.lineTo(xPos, originY - 8);
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
      label.y = originY - 24;
      app.stage.addChild(label);
    }

    // Dynamic Y-Axis Labels & Ticks
    for (let i = 1; i <= maxMetersY; i++) {
      const yPos = originY - (i * pixelsPerMeter);
      
      gridLayer.moveTo(originX, yPos);
      gridLayer.lineTo(originX + 8, yPos);
      gridLayer.stroke({ width: 1, color: 0x4b5563 });

      const label = new PIXI.Text({
        text: `${mapStartBottomLeftYMeters + i}m`,
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
    
    const stepXCm = markerXCm; // Bottom-left X in cm
    const stepYCm = markerYCm; // Bottom-left Y in cm
    const stepWidthCm = 112.2;   // 1 meter wide
    const stepHeightCm = 114;

    const stepWidthPx = (stepWidthCm / 100) * pixelsPerMeter;
    const stepHeightPx = (stepHeightCm / 100) * pixelsPerMeter;

    const blScreenX = originX + ((stepXCm - mapStartBottomLeftXCm) / 100) * pixelsPerMeter;
    const blScreenY = originY - ((stepYCm - mapStartBottomLeftYCm) / 100) * pixelsPerMeter;

    // Pixi rect() takes the top-left corner:
    const rectX = blScreenX;
    const rectY = blScreenY - stepHeightPx; 

    const step = new PIXI.Graphics();
    
    step.rect(rectX, rectY, stepWidthPx, stepHeightPx);
    step.fill({ color: 0x636363, alpha: 0.4 });
    step.moveTo(rectX, rectY);                                            
    step.lineTo(rectX, rectY + stepHeightPx);                 
    step.lineTo(rectX + stepWidthPx, rectY + stepHeightPx);         
    step.stroke({ 
      width: 0.128 * pixelsPerMeter, 
      color: 0xfcba03, 
      alignment: 0,
      alpha: 0.7
    });

    app.stage.addChild(step);

    const marker = createMarker({
      xCm: markerXCm,
      yCm: markerYCm,
      originX,
      originY,
      pixelsPerMeter,
      bottomLeftXCm: mapStartBottomLeftXCm,
      bottomLeftYCm: mapStartBottomLeftYCm,
      color: 0x000000,
      label: `(${markerXCm}cm, ${markerYCm}cm)`
    });
    app.stage.addChild(marker);

    // 5. Compass Widget
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