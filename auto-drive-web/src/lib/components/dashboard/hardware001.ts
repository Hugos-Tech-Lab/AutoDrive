import * as PIXI from "pixi.js";

export function createHardware001(): PIXI.Container {
  const container = new PIXI.Container();

  const graphics = new PIXI.Graphics();

  graphics.rect(-10, 0, 2.5, 6.5);
  graphics.fill({ color: 0x000000 });

  graphics.rect(7.5, 0, 2.5, 6.5);
  graphics.fill({ color: 0x000000 });

  graphics.rect(-7, -7, 14, 14);
  graphics.fill({ color: 0x000000 });
  graphics.stroke({ color: 0xffffff, width: 2 });

  container.addChild(graphics);

  const markerContainer = new PIXI.Container();
  const markerGfx = new PIXI.Graphics();

  const markerSize = 8;
  const halfSize = markerSize / 2;
  
  markerGfx.rect(-halfSize, -halfSize, markerSize, markerSize);
  markerGfx.fill({ color: 0x000000 });

  const gridSize = 4;
  const cellSize = markerSize / gridSize;
  const gridStartX = -halfSize;
  const gridStartY = -halfSize;

  const matrix0 = [
    [1, 0, 1, 1],
    [0, 1, 0, 1],
    [0, 0, 1, 1],
    [0, 0, 1, 0]
  ];

  for (let row = 0; row < gridSize; row++) {
    for (let col = 0; col < gridSize; col++) {
      if (matrix0[row][col] === 1) {
        markerGfx.rect(
          gridStartX + col * cellSize,
          gridStartY + row * cellSize,
          cellSize,
          cellSize
        );
        markerGfx.fill({ color: 0xffffff });
      }
    }
  }

  markerContainer.addChild(markerGfx);

  markerContainer.scale.y = -1;
  container.addChild(markerContainer);

  return container;
}