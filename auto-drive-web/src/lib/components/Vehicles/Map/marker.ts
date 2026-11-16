import * as PIXI from 'pixi.js';

export interface MarkerConfig {
  x: number;
  y: number;
  color?: number;
}

export function createMarker(config: MarkerConfig): PIXI.Container {
  const {
    x,
    y,
    color = 0x000000,

  } = config;

  let radius = 2;

  const container = new PIXI.Container();

  const markerGraphic = new PIXI.Graphics();
  markerGraphic.circle(0, 0, radius);
  markerGraphic.fill({ color });
  markerGraphic.stroke({ width: 0.4, color: 0xffffff });
  container.addChild(markerGraphic);

  const text = new PIXI.Text({
    text: `(${x}cm, ${y}cm)`,
    style: {
      fontFamily: 'sans-serif',
      fontSize: 14,
      fill: 0x000000,
      fontWeight: 'bold'
    }
  });
  text.x = radius;
  text.y = 0;
  const scale = 0.3;
  text.scale.x = scale;
  text.scale.y = -scale;
  container.addChild(text);
  

  container.position.set(x, y);
  return container;
}