
// packages/typst.ts/

import { copyFileSync, mkdirSync } from "fs";

import { join } from "path";

// import.meta.dirname
const __dirname = join(import.meta.dirname, '../..');

mkdirSync(join(__dirname, "typst-all-in-one.ts/dist/cjs"), { recursive: true });
mkdirSync(join(__dirname, "typst-all-in-one.ts/dist/esm"), { recursive: true });

copyFileSync(
    join(__dirname, "typst.ts/dist/esm/contrib/all-in-one.bundle.js"),
    join(__dirname, "typst-all-in-one.ts/dist/esm/index.js")
)
copyFileSync(
    join(__dirname, "typst.ts/dist/esm/contrib/all-in-one.d.mts"),
    join(__dirname, "typst-all-in-one.ts/dist/esm/index.d.mts")
)
copyFileSync(
    join(__dirname, "typst.ts/dist/cjs/contrib/all-in-one.bundle.js"),
    join(__dirname, "typst-all-in-one.ts/dist/cjs/index.js")
)
copyFileSync(
    join(__dirname, "typst.ts/dist/cjs/contrib/all-in-one.d.cts"),
    join(__dirname, "typst-all-in-one.ts/dist/cjs/index.d.cts")
)