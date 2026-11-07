import * as PIXI from "pixi.js";
import { doorStepCornerX, doorStepCornerY } from "./permanentRoom";

export function createBoundaries001(): PIXI.Container {
  const container = new PIXI.Container();

  const path = new PIXI.Graphics();
  path.moveTo(doorStepCornerX - 30.1, doorStepCornerY);
  path.lineTo(doorStepCornerX - 163, doorStepCornerY);
  path.stroke({ color: 0xff0000, width: 1.7 });
  path.alpha = 0.6;

  container.addChild(path);

  return container;
}