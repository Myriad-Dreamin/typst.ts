(function () {
  document.ready(() => {
    let plugin = window.TypstRenderModule.createTypstRenderer();
    plugin
      .init({
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
