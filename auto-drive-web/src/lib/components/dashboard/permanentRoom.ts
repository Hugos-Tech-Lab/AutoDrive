import * as PIXI from "pixi.js";
import { createTileLayer } from "./tiles";
import { createMarker } from "./marker";

export const doorStepCornerX = 455; 
export const doorStepCornerY = 254.5; 

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

  const bottomRightTableLeg = new PIXI.Graphics();
  bottomRightTableLeg.rect(doorStepCornerX - 54.2, doorStepCornerY + 40, 3.5, 3.5);
  bottomRightTableLeg.fill({ color: 0x000000, alpha: 0.8 });
  container.addChild(bottomRightTableLeg);

  const step = new PIXI.Graphics();
  step.rect(455, 254.5, 102, 100);
  step.fill({ color: 0x000000, alpha: 0.3 });
  container.addChild(step);

  const entryYellow = new PIXI.Graphics();
  entryYellow.rect(442.2, 254.5, 12.8, 100);
  entryYellow.fill({ color: 0xffbf00, alpha: 0.8 });
  entryYellow.rect(442.2, 254.5 - 12.8, 102 + 12.8, 12.8);
  entryYellow.fill({ color: 0xffbf00, alpha: 0.8 });
  container.addChild(entryYellow);

  const door = new PIXI.Graphics();
  door.rect(557, 254.5, 50, 100);
  door.fill({ color: 0x00, alpha: 0.8});
  container.addChild(door);

  const marker = createMarker({
    x: doorStepCornerX,
    y: doorStepCornerY,
    color: 0x0000ff
  });
  container.addChild(marker);

  return container;
}
