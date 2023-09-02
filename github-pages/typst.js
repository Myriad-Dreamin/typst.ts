(function () {
  document.ready(() => {
    let plugin = window.TypstRenderModule.createTypstRenderer(pdfjsLib);
    plugin
      .init({
        beforeBuild: [
          window.TypstRenderModule.preloadRemoteFonts([
            // use glyphs.json
            // '/typst.ts/assets/fonts/LinLibertine_R.ttf',
            // '/typst.ts/assets/fonts/LinLibertine_RB.ttf',
            // '/typst.ts/assets/fonts/LinLibertine_RBI.ttf',
            // '/typst.ts/assets/fonts/LinLibertine_RI.ttf',
            // '/typst.ts/assets/fonts/NewCMMath-Regular.otf',
            // '/typst.ts/assets/fonts/DejaVuSansMono.ttf',
            // not used
            // '/typst.ts/assets/fonts/NewCMMath-Book.otf',
            // '/typst.ts/assets/fonts/DejaVuSansMono-Bold.ttf',
            // '/typst.ts/assets/fonts/DejaVuSansMono-BoldOblique.ttf',
            // '/typst.ts/assets/fonts/DejaVuSansMono-Oblique.ttf',
          ]),
        ],
        getModule: () => '/typst.ts/renderer/typst_ts_renderer_bg.wasm',
      })
      .then(() => {
        let artifactContent = undefined;

        fetch('/typst.ts/docs/readme.artifact.sir.in')
          .then(response => response.arrayBuffer())
          .then(buffer => new Uint8Array(buffer))
          .then(buffer => {
            artifactContent = buffer;
          })
          .then(() => {
            const appContainer = document.getElementById('typst-app');

            plugin
              .render({
                artifactContent,
                container: appContainer,
                backgroundColor: '#343541',
                pixelPerPt: 4.5,
              })
              .then(renderResult => {
                console.log('render done');
              });
          });
      });
  });
})();
