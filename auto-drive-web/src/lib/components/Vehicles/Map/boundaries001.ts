import * as PIXI from "pixi.js";
import { doorStepCornerX, doorStepCornerY } from "./permanentRoom";

export function createBoundaries001(): PIXI.Container {
  const container = new PIXI.Container();

  const drawRoomPath = (g: PIXI.Graphics) => {
    g.moveTo(doorStepCornerX - 30.1, doorStepCornerY);
    g.lineTo(doorStepCornerX - 162, doorStepCornerY);
    g.lineTo(doorStepCornerX - 162, doorStepCornerY - 121.2);
    g.lineTo(doorStepCornerX - 162, doorStepCornerY - 121.2);
    g.lineTo(doorStepCornerX + 69.3, doorStepCornerY - 121.2);
    g.lineTo(doorStepCornerX + 69.3, doorStepCornerY - 30.1);
    g.lineTo(doorStepCornerX - 30.1, doorStepCornerY - 30.1);
    g.lineTo(doorStepCornerX - 30.1, doorStepCornerY);
  };

  const bgShading = new PIXI.Graphics();
  bgShading.rect(-5000, -5000, 10000, 10000);
  bgShading.fill({ color: 0xff0000, alpha: 0.22 });
  drawRoomPath(bgShading);
  bgShading.cut();
  container.addChild(bgShading);

  const strokeOutline = new PIXI.Graphics();
  drawRoomPath(strokeOutline);
  strokeOutline.stroke({ color: 0xff0000, width: 1.7 });
  strokeOutline.alpha = 0.6;
  container.addChild(strokeOutline);

  return container;
}