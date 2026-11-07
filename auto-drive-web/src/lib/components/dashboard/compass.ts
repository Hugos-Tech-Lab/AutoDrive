
import * as PIXI from 'pixi.js';

export class Compass extends PIXI.Container {
    constructor() {
      super();

      const circle = new PIXI.Graphics();
      circle.circle(0, 0, 40);
      circle.stroke({ width: 2, color: 0x334155 });
      this.addChild(circle);

      const northPointer = new PIXI.Graphics();
      northPointer.moveTo(0, -32);
      northPointer.lineTo(-8, 0);
      northPointer.lineTo(0, -4);
      northPointer.fill({ color: 0x1e293b });

      northPointer.moveTo(0, -32);
      northPointer.lineTo(8, 0);
      northPointer.lineTo(0, -4);
      northPointer.fill({ color: 0x475569 });
      this.addChild(northPointer);

      const southPointer = new PIXI.Graphics();
      southPointer.moveTo(0, 32);
      southPointer.lineTo(-8, 0);
      southPointer.lineTo(0, 4);
      southPointer.fill({ color: 0x94a3b8 });
      this.addChild(southPointer);

      const textStyle = new PIXI.TextStyle({
        fontFamily: 'sans-serif',
        fontSize: 22,
        fontWeight: 'bold',
        fill: 0x1e293b
      });

      const northText = new PIXI.Text({ text: 'N', style: textStyle });
      northText.anchor.set(0.5, 1);
      northText.position.set(0, -48);

      const southText = new PIXI.Text({ text: 'S', style: textStyle });
      southText.anchor.set(0.5, 0);
      southText.position.set(0, 48);

      const westText = new PIXI.Text({ text: 'W', style: textStyle });
      westText.anchor.set(1, 0.5);
      westText.position.set(-48, 0);

      const eastText = new PIXI.Text({ text: 'E', style: textStyle });
      eastText.anchor.set(0, 0.5);
      eastText.position.set(48, 0);

      this.addChild(northText);
      this.addChild(southText);
      this.addChild(westText);
      this.addChild(eastText);
    }
  }