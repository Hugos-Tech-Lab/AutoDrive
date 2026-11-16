import * as PIXI from 'pixi.js';

interface GridOptions {
  widthPixels: number;
  heightPixels: number;
  pixelsPerMeter: number;
  bottomLeftXCm: number;
  bottomLeftYCm: number;
  worldContainer: PIXI.Container;
}

export function createGridLayer(options: GridOptions): PIXI.Graphics {
  const { widthPixels, heightPixels, pixelsPerMeter, bottomLeftXCm, bottomLeftYCm, worldContainer } = options;
  const scaleFactor = pixelsPerMeter / 100;
  
  const gridLayer = new PIXI.Graphics();
  
  gridLayer.moveTo(0, heightPixels);
  gridLayer.lineTo(0 + widthPixels, heightPixels);
  gridLayer.stroke({ width: 2, color: 0x4b5563 });

  gridLayer.moveTo(0, heightPixels);
  gridLayer.lineTo(0, heightPixels - heightPixels);
  gridLayer.stroke({ width: 2, color: 0x4b5563 });

  // --- Dynamic X-Axis Labels & Ticks ---
  const startMeterX = Math.ceil(bottomLeftXCm / 100);
  const endMeterX = (bottomLeftXCm / 100) + (widthPixels / pixelsPerMeter);

  for (let m = startMeterX; m <= endMeterX; m++) {
    const xPos = (m * 100 * scaleFactor) + worldContainer.x;
    
    if (xPos >= 0 && xPos <= widthPixels) {
      gridLayer.moveTo(xPos, heightPixels);
      gridLayer.lineTo(xPos, heightPixels - 8);
      gridLayer.stroke({ width: 1, color: 0x4b5563 });

      const label = new PIXI.Text({
        text: `${m}m`,
        style: {
          fontFamily: 'sans-serif',
          fontSize: 12,
          fill: 0x4b5563
        }
      });
      label.x = xPos - 10;
      label.y = heightPixels - 24;
      
      // Attach the label directly to the gridLayer so they group together
      gridLayer.addChild(label);
    }
  }

  // --- Dynamic Y-Axis Labels & Ticks ---
  const startMeterY = Math.ceil(bottomLeftYCm / 100);
  const endMeterY = (bottomLeftYCm / 100) + (heightPixels / pixelsPerMeter);

  for (let m = startMeterY; m <= endMeterY; m++) {
    const yPos = worldContainer.y - (m * 100 * scaleFactor);
    
    if (yPos >= 0 && yPos <= heightPixels) {
      gridLayer.moveTo(0, yPos);
      gridLayer.lineTo(0 + 8, yPos);
      gridLayer.stroke({ width: 1, color: 0x4b5563 });

      const label = new PIXI.Text({
        text: `${m}m`,
        style: {
          fontFamily: 'sans-serif',
          fontSize: 12,
          fill: 0x4b5563
        }
      });
      label.x = 12;
      label.y = yPos - 8;
      
      gridLayer.addChild(label);
    }
  }

  return gridLayer;
}