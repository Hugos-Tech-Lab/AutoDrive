import * as PIXI from 'pixi.js';

export interface TileConfig {
  maxCmX: number;
  maxCmY: number;
}

export function createTileLayer(config: TileConfig): PIXI.Graphics {
  const { maxCmX, maxCmY } = config;

  // Convert dimensions from mm/m to centimeters
  const tileWidth = 19.6;  // 196 mm = 19.6 cm
  const tileHeight = 29.7; // 297 mm = 29.7 cm
  const grout = 0.6;       // 6 mm = 0.6 cm

  const stepX = tileWidth + grout;
  const stepY = tileHeight + grout;

  const tileLayer = new PIXI.Graphics();

  // Background (Grout color)
  tileLayer.rect(0, 0, maxCmX, maxCmY);
  tileLayer.fill({ color: 0x9ca3af });

  const cols = Math.ceil(maxCmX / stepX);
  const rows = Math.ceil(maxCmY / stepY);

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
