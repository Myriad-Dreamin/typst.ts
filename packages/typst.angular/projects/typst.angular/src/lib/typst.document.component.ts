import { Component, ElementRef, Input, ViewChild } from '@angular/core';
import { withGlobalRenderer } from '@myriaddreamin/typst.ts/dist/contrib/global-renderer';
import * as typst from '@myriaddreamin/typst.ts';

@Component({
  selector: 'typst-document',
  templateUrl: `./typst.document.component.html`,
  styles: [],
})
export class TypstDocumentComponent {
  _artifact: Uint8Array = new Uint8Array(0);
  @ViewChild('typst_app') typst_app?: ElementRef<HTMLDivElement>;

  @Input() fill: string = '#ffffff';

  @Input()
  set artifact(artifact: Uint8Array) {
    this._artifact = artifact;
    this.applyArtifact();
  }

  get artifact(): Uint8Array {
    return this._artifact;
  }

  constructor() {}

  applyArtifact() {
    if (this.typst_app?.nativeElement) {
      const displayDiv = this.typst_app?.nativeElement;
      if (this.artifact?.length) {
        const doRender = (renderer: typst.TypstRenderer) => {
          console.log(renderer);
          return renderer.render({
            artifactContent: this.artifact,
            backgroundColor: this.fill,
            container: displayDiv,
            pixelPerPt: 8,
          });
        };

        /// render after init
        withGlobalRenderer(
          (window as unknown as any).pdfjsLib,
          {
            beforeBuild: [
              typst.preloadRemoteFonts([
                'http://localhost:20811/fonts/LinLibertine_R.ttf',
                'http://localhost:20811/fonts/LinLibertine_RB.ttf',
                'http://localhost:20811/fonts/LinLibertine_RBI.ttf',
                'http://localhost:20811/fonts/LinLibertine_RI.ttf',
                'http://localhost:20811/fonts/NewCMMath-Book.otf',
                'http://localhost:20811/fonts/NewCMMath-Regular.otf',
              ]),
              typst.preloadSystemFonts({
                byFamily: ['Segoe UI Symbol'],
              }),
            ],
            getModule: () =>
              'node_modules/@myriaddreamin/typst-ts-renderer/typst_ts_renderer_bg.wasm',
          },
          doRender,
        );
      } else {
        displayDiv.innerHTML = '';
      }
    }
  }
}
