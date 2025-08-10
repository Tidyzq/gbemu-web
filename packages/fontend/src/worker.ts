import { Emu } from "@gbemu-web/core";

export type WorkerRequest = {
  cartData: ArrayBuffer;
  mainBuffer: SharedArrayBuffer;
  debugBuffer: SharedArrayBuffer;
};

self.onmessage = (ev: MessageEvent<WorkerRequest>) => {
  console.log("onmessage", ev.data);
  const { cartData, mainBuffer, debugBuffer } = ev.data;

  const emu = new Emu(new Uint8Array(cartData), debugBuffer);

  emu.run()
};
