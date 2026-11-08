<script lang="ts">
  import { onMount, onDestroy } from 'svelte';
  import * as PIXI from 'pixi.js';
  import { Compass } from './dashboard/compass';
  import { createPermanentRoom } from './dashboard/permanentRoom';
  import { createBoundaries001 } from './dashboard/boundaries001';
  import { createHardware001 } from './dashboard/hardware001';
  import { createGridLayer } from './dashboard/uiGridLayer';
  import { endpointRoot } from '../../endpoint';

  let container: HTMLDivElement;
  let app: PIXI.Application;
  
  type Pose = { number_plate: string; x: number; y: number; rotation: number };
  let hardwareList = $state<Pose[]>([]);

  let hardwareContainer: PIXI.Container;

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
    
    // Initialize the hardware container that will hold all dynamic hardware items
    hardwareContainer = new PIXI.Container();

    worldContainer.addChild(permanentRoom);
    worldContainer.addChild(boundaries);
    worldContainer.addChild(hardwareContainer);

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

  $effect(() => {
    console.log("hi");
    if (!hardwareContainer) return;

    // TODO: map by ID
    hardwareContainer.removeChildren();
    // console.log(hardwareList.length)
    hardwareList.forEach((pose) => {
      // console.log(pose);


      const hwItem = createHardware001();
      hwItem.x = pose.x;
      hwItem.y = pose.y;
      hwItem.rotation = pose.rotation;
      
      hardwareContainer.addChild(hwItem);
    });
  });

  const sse = new EventSource(`${endpointRoot()}/hardware/positions`);

  sse.addEventListener("positions", (event) => {
    try {

      const json = JSON.parse(event.data);
      const dataObj = json.data || json; 
      const poses: Pose[] = Object.values(dataObj);
      console.log(poses);
      poses.forEach((pose) => { 
        const hwItem = createHardware001();
        hwItem.x = pose.x;
        hwItem.y = pose.y;
        hwItem.rotation = pose.rotation;
        
        hardwareContainer.addChild(hwItem);
      });
    } catch (err) {
      console.error("Failed to parse SSE overview data", err);
    }
  });

  sse.onerror = (err) => {
    console.error("SSE Error on hardware overview", err);
  };

  onDestroy(() => {
    if (app) {
      sse.close();
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