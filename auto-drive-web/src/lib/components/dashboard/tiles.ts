import * as PIXI from 'pixi.js';

export interface TileConfig {
  originX: number;
  originY: number;
  pixelsPerMeter: number;
  maxMetersX: number;
  maxMetersY: number;
}

export function createTileLayer(config: TileConfig): PIXI.Graphics {
  const { originX, originY, pixelsPerMeter, maxMetersX, maxMetersY } = config;

  const pxPerMm = pixelsPerMeter / 1000; // 0.2 px per mm
  const tileWidthPx = 196 * pxPerMm;    // 39.2 px
  const tileHeightPx = 297 * pxPerMm;   // 59.4 px
  const groutPx = 6 * pxPerMm;          // 1.2 px

  const stepX = tileWidthPx + groutPx;
  const stepY = tileHeightPx + groutPx;

  const tileLayer = new PIXI.Graphics();

  const totalWidthPx = maxMetersX * pixelsPerMeter;
  const totalHeightPx = maxMetersY * pixelsPerMeter;

  // Background (Grout color)
  tileLayer.rect(originX, originY - totalHeightPx, totalWidthPx, totalHeightPx);
  tileLayer.fill({ color: 0x9ca3af });

  const cols = Math.ceil(totalWidthPx / stepX);
  const rows = Math.ceil(totalHeightPx / stepY);

  // Individual tiles
  for (let r = 0; r < rows; r++) {
    for (let c = 0; c < cols; c++) {
      const x = originX + (c * stepX) + (groutPx / 2);
      const y = originY - totalHeightPx + (r * stepY) + (groutPx / 2);

      tileLayer.rect(x, y, tileWidthPx, tileHeightPx);
      tileLayer.fill({ color: 0xf9fafb });
    }
  }

  return tileLayer;
}
