import { withGlobalRenderer } from "@myriaddreamin/typst.ts/dist/esm/contrib/global-renderer.mjs";
import * as typst from "@myriaddreamin/typst.ts";
import { createEffect, createSignal } from "solid-js";
import injectedCss from  "./html-export.css?raw";

export interface TypstDocumentProps {
	fill?: string;
	artifact?: Uint8Array;
	format?: "vector";
}

let moduleInitOptions: typst.InitOptions = {
	beforeBuild: [],
	getModule: () =>
		"https://cdn.jsdelivr.net/npm/@myriaddreamin/typst-ts-renderer/pkg/typst_ts_renderer_bg.wasm",
};

export const TypstDocument = ({
	fill,
	artifact,
	format,
}: TypstDocumentProps) => {
	/// --- beg: manipulate permission --- ///

	// todo: acquire permission.
	const [permission, setPermissionInternal] = createSignal(false);
	const setPermissionAndOk = (status: PermissionStatus) => {
		if (status.state === "granted") {
			setPermissionInternal(true);
			return true;
		}
		setPermissionInternal(false);
		return false;
	};
	createEffect(() => {
		/// only works in chromium
		navigator.permissions.query({ name: 'local-fonts' as PermissionName }).then(status => {
			if (setPermissionAndOk(status)) {
				return false;
			}
			status.addEventListener('change', event => {
				console.log(event, status);
				setPermissionAndOk(status);
			});
		});
	});

	/// --- end: manipulate permission --- ///

	/// --- beg: update document --- ///
	const [displayDivRef, setDisplayDivRef] = createSignal<
		HTMLDivElement | undefined
	>();

	createEffect(() => {
		const doRender = (renderer: typst.TypstRenderer) => {
			const divElem = displayDivRef();
			if (!divElem) {
				return;
			}

			return renderer.renderToCanvas({
				artifactContent: artifact || new Uint8Array(0),
				format: "vector",
				backgroundColor: fill || "#ffffff",
				container: divElem,
				pixelPerPt: 3,
			});
		};

		/// get display layer div
		const divElem = displayDivRef();
		if (!divElem) {
			return;
		}

		/// we allow empty artifact
		if (!artifact?.length) {
			divElem!.innerHTML = "";
			return;
		}

		console.log(displayDivRef());
		/// render after init
		withGlobalRenderer(typst.createTypstRenderer, moduleInitOptions, doRender);
	}, [permission, displayDivRef, fill, artifact, format]);

	/// --- end: update document --- ///

	return <div>
		{/* todo: remove this embedded css */}
		<style>{injectedCss}</style>
		<div ref={setDisplayDivRef}></div>
	</div>;
};

TypstDocument.setWasmModuleInitOptions = (opts: typst.InitOptions) => {
	moduleInitOptions = opts;
};
