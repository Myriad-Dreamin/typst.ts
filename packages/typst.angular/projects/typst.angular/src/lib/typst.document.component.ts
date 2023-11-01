import { Component, ElementRef, Input, ViewChild } from '@angular/core';
import { withGlobalRenderer } from '@myriaddreamin/typst.ts/dist/esm/contrib/global-renderer.mjs';
import * as typst from '@myriaddreamin/typst.ts/dist/esm/main.mjs';

let moduleInitOptions: typst.InitOptions = {
  beforeBuild: [],
  getModule: () => '/assets/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm',
};

@Component({
  selector: 'typst-document',
  templateUrl: `./typst.document.component.html`,
  styles: [],
})
export class TypstDocumentComponent {
  _artifact: Uint8Array = new Uint8Array(0);
  @ViewChild('typst_app') typst_app?: ElementRef<HTMLDivElement>;

  @Input() fill: string = '#ffffff';

  @Input() format = 'vector' as const;

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
          renderer.render({
            artifactContent: this.artifact,
            format: 'vector',
            backgroundColor: this.fill,
            container: displayDiv,
            pixelPerPt: 8,
          });
          return;
        };

        /// render after init
        withGlobalRenderer(
          typst.createTypstRenderer,
          (window as unknown as any).pdfjsLib,
          moduleInitOptions,
          doRender,
        );
      } else {
        displayDiv.innerHTML = '';
      }
    }
  }

  static setWasmModuleInitOptions(opts: typst.InitOptions) {
    moduleInitOptions = opts;
  }
}
