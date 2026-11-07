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
  container.addChild(tiles);

  const step = new PIXI.Graphics();
  step.rect(400, 200, 100, 100);
  step.fill({ color: 0x636363, alpha: 0.4 });
  container.addChild(step);

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
