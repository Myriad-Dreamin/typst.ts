// import MyWorker from './contrib/canvas-worker.mjs?worker&inline';
// import MyWorker from './contrib/canvas-worker.mjs?worker&inline';
import { CanvasCommand } from './contrib/canvas-worker-common.mjs';

// s.set_attribute("crossorigin", "anonymous").unwrap();

const data = fetch('http://localhost:20810/core/dist/canvas-worker.js');

async function main() {
  const workerScript = await (await data).text();
  const workerUrl = URL.createObjectURL(
    new Blob([workerScript], { type: 'application/javascript' }),
  );

  const worker = new Worker(workerUrl, { type: 'module' });

  // console.log('hello world');

  const glyphT =
    'M 391 172 Q 391 117 433 91.5 Q 475 66 614 66 Q 731 66 821 89.5 Q 911 113 989 168 Q 1067 223 1109 327.5 Q 1151 432 1151 582 Q 1151 707 1120.5 821.5 Q 1090 936 1024 1036.5 Q 958 1137 839.5 1196 Q 721 1255 563 1255 Q 391 1255 391 1155 L 391 172 Z M 303 1321 Q 336 1321 447.5 1323 Q 559 1325 641 1325 Q 936 1325 1142 1121.5 Q 1348 918 1348 631 Q 1348 451 1285.5 320.5 Q 1223 190 1119.5 122.5 Q 1016 55 906.5 25.5 Q 797 -4 678 -4 Q 555 -4 440.5 -2 Q 326 0 305 0 Q 199 0 39 -4 Q 29 4 29 28.5 Q 29 53 39 63 Q 154 67 185.5 97 Q 217 127 217 250 L 217 1071 Q 217 1194 185.5 1223.5 Q 154 1253 39 1257 Q 29 1265 29 1290 Q 29 1315 39 1325 Q 203 1321 303 1321 Z ';
  // const glyph = new TextEncoder().encode(glyphT);

  worker.postMessage([
    CanvasCommand.Init,
    {
      mainScript: 'http://localhost:20810/core/dist/main.mjs',
    },
  ]);

  const doRender = () => {
    worker.postMessage([CanvasCommand.Render, { glyph: glyphT }]);
  };

  let rendered = 0;

  worker.onmessage = event => {
    // console.log('onmessage', event.data);
    const { data } = event;
    const [ty, opts] = data;
    switch (ty) {
      case CanvasCommand.Init: {
        // console.log('worker init');
        doRender();
        break;
      }
      case CanvasCommand.Render: {
        const canvas = document.createElement('canvas');
        canvas.width = 16;
        canvas.height = 16;
        canvas.getContext('2d')!.drawImage(opts.result, 0, 0);
        // console.log('worker render', opts);
        // const c = canvasPool[opts];
        rendered += 1;
        if (rendered == 10) {
          document.body.appendChild(document.createElement('div'));
        }
        if (rendered < 18) {
          doRender();
        }
        document.body.appendChild(canvas);
        break;
      }
    }
  };
}

main();
