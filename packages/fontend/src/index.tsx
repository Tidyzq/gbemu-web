import { render } from "solid-js/web";

const worker = new Worker(new URL("./worker.ts", import.meta.url));

const initCanvas = (canvas: HTMLCanvasElement, width: number, height: number) => {
  const buffer = new SharedArrayBuffer(width * height * 4)
  const bufferView = new Uint8ClampedArray(buffer)

  const imageBuffer = new Uint8ClampedArray(width * height * 4)
  const imageData = new ImageData(imageBuffer, width, height);

  canvas.width = width;
  canvas.height = height;

  const ctx = canvas.getContext("2d")!;

  const render = () => {
    try {
      ctx.clearRect(0, 0, width, height);
      imageBuffer.set(bufferView)
      ctx.putImageData(imageData, 0, 0);

      requestAnimationFrame(render);
    } catch (e) {
      console.log(e);
    }
  };

  render();

  return buffer;
};

const Emu = () => {
  const handleFileSelection = async (
    event: Event & { target: HTMLInputElement }
  ) => {
    const file = event.target.files?.[0];
    if (!file) return;
    const reader = await new Promise<FileReader>((resolve, reject) => {
      const reader = new FileReader();
      reader.onload = () => {
        resolve(reader);
      };
      reader.onerror = (ev) => {
        console.error("read file error", ev);
        reject(ev);
      };
      reader.readAsArrayBuffer(file);
    });
    const arrayBuffer = reader.result as ArrayBuffer;

    const mainBuffer = initCanvas(mainScreenCanvas!, 300, 300)
    const debugBuffer = initCanvas(debugScreenCanvas!, 16 * 8, 32 * 8)

    worker.postMessage(
      {
        cartData: arrayBuffer,
        mainBuffer: mainBuffer,
        debugBuffer: debugBuffer,
      },
      [arrayBuffer]
    );

    worker.onmessage = (e) => {
      console.log(e);
    };
  };

  let mainScreenCanvas: HTMLCanvasElement | undefined;
  let debugScreenCanvas: HTMLCanvasElement | undefined;

  return (
    <>
      <canvas
        style="width: 300px; height: 300px; border: 1px solid black;"
        ref={mainScreenCanvas}
      />
      <canvas
        style={`width: ${16 * 8 * 2}px; height: ${32 * 8 * 2}px; border: 1px solid black;`}
        ref={debugScreenCanvas}
      />
      <input type="file" onChange={handleFileSelection}></input>
    </>
  );
};

const App = () => <Emu />;

render(App, document.body);
