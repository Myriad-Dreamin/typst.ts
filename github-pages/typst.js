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
        getModule: () => '/typst.ts/renderer/pkg/typst_ts_renderer_bg.wasm',
      })
      .then(() => {
        let artifactContent = undefined;

        const getGlyphs = fetch('/typst.ts/docs/readme.glyphs.json')
          .then(response => response.text())
          .then(content => JSON.parse(content))
          .catch(e => {
            console.log(e);
            return undefined;
          });

        fetch('/typst.ts/docs/readme.artifact.json')
          .then(response => response.arrayBuffer())
          .then(buffer => new Uint8Array(buffer))
          .then(buffer => {
            artifactContent = buffer;
          })
          .then(() => getGlyphs)
          .then(glyphPack => plugin.loadGlyphPack(glyphPack))
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
