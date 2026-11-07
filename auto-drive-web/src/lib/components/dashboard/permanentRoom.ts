import * as PIXI from "pixi.js";
import { createTileLayer } from "./tiles";

export function createPermanentRoom(): PIXI.Container {
  const container = new PIXI.Container();
  const tiles = createTileLayer({
    maxMetersX: 5,
    maxMetersY: 5,
  });

  container.addChild(tiles);


  

  return container;
}
