import * as PIXI from 'pixi.js';

export interface MarkerConfig {
  xCm: number;
  yCm: number;
  originX: number;
  originY: number;
  pixelsPerMeter: number;
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
    color = 0xef4444, // Default red
    radius = 6,
    label
  } = config;

  const container = new PIXI.Container();

  // Convert cm to pixels relative to the origin
  const pixPerCm = pixelsPerMeter / 100;
  const pixelX = originX + (xCm * pixPerCm);
  const pixelY = originY - (yCm * pixPerCm);

  // Draw the marker circle
  const markerGraphic = new PIXI.Graphics();
  markerGraphic.circle(0, 0, radius);
  markerGraphic.fill({ color });
  markerGraphic.stroke({ width: 2, color: 0xffffff });
  container.addChild(markerGraphic);

  // Optional text label near the marker
  if (label) {
    const text = new PIXI.Text({
      text: label,
      style: {
        fontFamily: 'sans-serif',
        fontSize: 11,
        fill: 0x1f2937,
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