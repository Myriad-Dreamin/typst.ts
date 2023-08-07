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
