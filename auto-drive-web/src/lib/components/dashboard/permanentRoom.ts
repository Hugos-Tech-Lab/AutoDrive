import * as PIXI from "pixi.js";
import { createTileLayer } from "./tiles";
import { createMarker } from "./marker";

export function createPermanentRoom(): PIXI.Container {
  const container = new PIXI.Container();

  const roomWidthCm = 600;
  const roomHeightCm = 600;

  const tiles = createTileLayer({
    maxCmX: roomWidthCm,
    maxCmY: roomHeightCm,
  });
  tiles.x = 10.4;
  tiles.y = 12;

  container.addChild(tiles);

  const step = new PIXI.Graphics();
  step.rect(455, 254.5, 102, 100);
  step.fill({ color: 0x000000, alpha: 0.3 });
  container.addChild(step);

  const door = new PIXI.Graphics();
  door.rect(557, 254.5, 50, 100);
  door.fill({ color: 0x00, alpha: 0.8});
  container.addChild(door);

  const markerXCm = 455;
  const markerYCm = 254.5;
  const marker = createMarker({
    x: markerXCm,
    y: markerYCm,
    color: 0xff0000
  });
  container.addChild(marker);

  return container;
}
