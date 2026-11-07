import * as PIXI from 'pixi.js';

export interface MarkerConfig {
  xCm: number;
  yCm: number;
  originX: number;
  originY: number;
  pixelsPerMeter: number;
  bottomLeftXCm: number; // Bottom-left X coordinate in cm
  bottomLeftYCm: number; // Bottom-left Y coordinate in cm
  color?: number;
  radius?: number;
  label?: string;
}

export function createMarker(config: MarkerConfig): PIXI.Container {
  const {
    xCm,
    yCm,
    originX,
    originY,
    pixelsPerMeter,
    bottomLeftXCm,
    bottomLeftYCm,
    color = 0x3b82f6,
    radius = 6,
    label
  } = config;

  const container = new PIXI.Container();
  const pixPerCm = pixelsPerMeter / 100;

  const pixelX = originX + ((xCm - bottomLeftXCm) * pixPerCm);
  const pixelY = originY - ((yCm - bottomLeftYCm) * pixPerCm);

  const markerGraphic = new PIXI.Graphics();
  markerGraphic.circle(0, 0, radius);
  markerGraphic.fill({ color });
  markerGraphic.stroke({ width: 2, color: 0xffffff });
  container.addChild(markerGraphic);

  if (label) {
    const text = new PIXI.Text({
      text: label,
      style: {
        fontFamily: 'sans-serif',
        fontSize: 11,
        fill: 0x000000,
        fontWeight: 'bold'
      }
    });
    text.x = radius + 4;
    text.y = -8;
    container.addChild(text);
  }

  container.position.set(pixelX, pixelY);
  return container;
}