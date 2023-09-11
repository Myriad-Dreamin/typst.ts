### Download and install `wasm-pack`

To download and install `wasm-pack`, use the following command:

```shell
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | bash
```

If you meet following error when running `curl ... | sh`:

```
sh: 139: [: x86_64: unexpected operator
```

It is because the `sh` command is incompatible with the script. To resolve this issue, please use the installation command (bash) specified in this section.

### `wasm-pack` crashes with `SIGSEGV`

If you receive the following error:

```
'wasm-pack build --target web --…' terminated by signal SIGSEGV (Address boundary error)
```

It may be due to an outdated or corrupted version of `wasm-pack` installed on your computer. Please update `wasm-pack` to the correct version.

### `wasm-pack` reports missing field at line...

If you see the following error message:

```
Error: failed to parse manifest: $HOME/project/typst.ts/Cargo.toml
Caused by: failed to parse manifest: $HOME/project/typst.ts/Cargo.toml
Caused by: missing field package at line 91 column 1
```

It may be due to an outdated or corrupted version of `wasm-pack` installed on your computer. Please update `wasm-pack` to the correct version.

### LinkError: function import requires a callable

If you receive the following error when you are running Wasm modules:

```
Uncaught (in promise) LinkError: WebAssembly.instantiate(): Import #122 module="wbg" function="\_\_wbindgen_closure_wrapper2753" error: function import requires a callable
```

It may be due to inconsistency between `packages/typst.ts` and Wasm modules you are using. Please also rebuild the js library `packages/typst.ts` to ensure consistency between `packages/typst.ts` and Wasm modules.

Note: the correct build command is `cd packages/typst.ts && yarn run build` instead of build renderer or compiler module individually.

### Component download failed for rust-std-wasm32-unknown-unknown: could not rename downloaded file

```
info: downloading component 'rust-std' for 'wasm32-unknown-unknown'
  error: component download failed for rust-std-wasm32-unknown-unknown: could not rename downloaded file from '/home/runner/.rustup/downloads/fffce79.partial' to '/home/runner/.rustup/downloads/fffce79'
```

This is because concurrent downloads are not supported by the `rustup`.

Please install wasm target toolchain before cocurrently building wasm modules:

```shell
rustup target add wasm32-unknown-unknown
```

### Module not found: Error: Can't resolve 'env' in '...'

```
Module not found: Error: Can't resolve 'env' in '@myriaddreamin/typst-ts-web-compiler/pkg'
```

This is your cargo cache is corrupted. Please clean your cargo cache and rebuild the project.

```shell
cargo clean
```
