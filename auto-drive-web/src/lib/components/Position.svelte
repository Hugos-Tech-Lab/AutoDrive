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

    const rectangle = new PIXI.Graphics();
    rectangle.rect(200, 150, 400, 300);
    rectangle.fill({ color: 0x4f46e5 }); 
    rectangle.stroke({ width: 4, color: 0x818cf8 });
    app.stage.addChild(rectangle);

    const compass = new Compass();
    compass.position.set(50, 550);
    compass.scale.set(0.5);
    app.stage.addChild(compass);
  });

  // Clean up Pixi resources when component is destroyed
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

  h2 {
    margin-bottom: 1rem;
  }

  .canvas-container {
    overflow: hidden;
  }
</style>