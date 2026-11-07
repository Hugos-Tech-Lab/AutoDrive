import * as PIXI from 'pixi.js';

export interface TileConfig {
  maxMetersX: number;
  maxMetersY: number;
}

export function createTileLayer(config: TileConfig): PIXI.Graphics {
  const { maxMetersX, maxMetersY } = config;

  const tileWidth = 0.196;  // 196 mm = 0.196 m
  const tileHeight = 0.297; // 297 mm = 0.297 m
  const grout = 0.006;      // 6 mm = 0.006 m

  const stepX = tileWidth + grout;
  const stepY = tileHeight + grout;

  const tileLayer = new PIXI.Graphics();

  // Background (Grout color)
  tileLayer.rect(0, 0, maxMetersX, maxMetersY);
  tileLayer.fill({ color: 0x9ca3af });

  const cols = Math.ceil(maxMetersX / stepX);
  const rows = Math.ceil(maxMetersY / stepY);

  // Individual tiles
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      const x = (c * stepX) + (grout / 2);
      const y = (r * stepY) + (grout / 2);

      tileLayer.rect(x, y, tileWidth, tileHeight);
      tileLayer.fill({ color: 0xf9fafb });
    }
  }

  return tileLayer;
}