import * as PIXI from "pixi.js";

export function createHardware001(): PIXI.Container {
  const container = new PIXI.Container();

  const graphics = new PIXI.Graphics();

  graphics.rect(-10, -6, 2.5, 6.5);
  graphics.fill({ color: 0x1a202c });

  graphics.rect(7.5, -6, 2.5, 6.5);
  graphics.fill({ color: 0x1a202c });

  graphics.rect(-7, -7, 14, 14);
  graphics.fill({ color: 0x4a5568 });
  graphics.stroke({ color: 0xffffff, width: 2 });

  container.addChild(graphics);

  return container;
}