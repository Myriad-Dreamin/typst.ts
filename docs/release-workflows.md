# Release Workflows

This repository now uses a manual, auditable release layout centered on one top-level release orchestrator. The generic npm package and project publish jobs run inside that orchestrator with npm trusted publishing, while the dedicated `typst.node`, Docker, and Rust crate lanes remain separate reusable workflows.

## Protected npm environment

- GitHub Environment: `npm-publish`
- Trusted publishing runtime: Node `22.14.0` or newer with npm `11.5.1` or newer
- npm credentials: GitHub OIDC with `id-token: write`
- npm token handling: the generic package and project publish jobs do not use `NPM_TOKEN` or write a registry token into `.npmrc`

## Workflow coverage

### `.github/workflows/release-orchestration.yml`

Manual coordinator for one full release run per version. This is the top-level trusted publishing workflow filename for the covered npm packages and projects. It does not expose partial package or project selection.

The fixed release order is:

1. verify the generic workspace npm package lane
2. verify the publishable project batch
3. publish the generic workspace npm package lane
4. call `.github/workflows/release-node.yaml`
5. publish the npm and Cargo project batch
6. call `.github/workflows/docker.yaml`
7. call `.github/workflows/reelase-crates.yaml`

The generic workspace npm package lane covers:

- `@myriaddreamin/typst-ts-web-compiler`
- `@myriaddreamin/typst-ts-parser`
- `@myriaddreamin/typst-ts-renderer`
- `@myriaddreamin/typst-all-in-one.ts`
- `@myriaddreamin/typst.angular`
- `@myriaddreamin/typst.react`
- `@myriaddreamin/typst.solid`
- `@myriaddreamin/typst.svelte`
- `@myriaddreamin/typst.ts`
- `@myriaddreamin/typst.vue3`

The generic workspace npm package lane skips:

- `@myriaddreamin/typst-ts-node-compiler`: released by `.github/workflows/release-node.yaml`
- `enhanced-typst-svg`: private package outside the generic workspace lane

The project release batch covers:

- `hexo-renderer-typst`
- `@myriaddreamin/rehype-typst`
- `@myriaddreamin/vite-plugin-typst`
- `@myriaddreamin/highlighter-typst`
- `projects/rustdoc-typst-demo`

The project release batch skips:

- `@myriaddreamin/vistyp`: private project outside the versioned release batch

### `.github/workflows/release-node.yaml`

Dedicated `typst.node` release lane. It remains manually dispatchable, is reused from the orchestrator through `workflow_call`, and preserves the existing napi-rs build, test, artifact, npm publish, and GitHub Release upload flow for these packages:

- `@myriaddreamin/typst-ts-node-compiler`
- `@myriaddreamin/typst-ts-node-compiler-android-arm-eabi`
- `@myriaddreamin/typst-ts-node-compiler-android-arm64`
- `@myriaddreamin/typst-ts-node-compiler-darwin-arm64`
- `@myriaddreamin/typst-ts-node-compiler-darwin-x64`
- `@myriaddreamin/typst-ts-node-compiler-linux-arm-gnueabihf`
- `@myriaddreamin/typst-ts-node-compiler-linux-arm64-gnu`
- `@myriaddreamin/typst-ts-node-compiler-linux-arm64-musl`
- `@myriaddreamin/typst-ts-node-compiler-linux-x64-gnu`
- `@myriaddreamin/typst-ts-node-compiler-linux-x64-musl`
- `@myriaddreamin/typst-ts-node-compiler-win32-arm64-msvc`
- `@myriaddreamin/typst-ts-node-compiler-win32-x64-msvc`

This workflow also uploads the per-platform `.node` assets to the GitHub Release tag supplied to the workflow. It remains separate from the trusted-publishing lanes so the existing napi-rs release flow stays intact.

### `.github/workflows/docker.yaml`

Manual and reusable Docker publish lane for the repository image on GHCR. It publishes:

- `ghcr.io/<owner>/<repo>` for `linux/amd64`
- `ghcr.io/<owner>/<repo>` for `linux/arm64`

Authentication uses `GITHUB_TOKEN` with `packages: write`.

### `.github/workflows/reelase-crates.yaml`

Manual and reusable Rust workspace crate lane. It verifies and publishes:

- `reflexo`
- `reflexo-typst2vec`
- `reflexo-vec2bbox`
- `reflexo-vec2canvas`
- `reflexo-typst2hast`
- `reflexo-vec2sema`
- `reflexo-vec2svg`
- `reflexo-vec2dom`
- `reflexo-typst`

Authentication uses `CRATES_IO_TOKEN`.

## npm trusted publisher checklist

Before the first live npm publish for a package:

1. Create or update the protected GitHub Environment named `npm-publish` with the required reviewers and any branch or tag restrictions you want for live publish approvals.
2. In npm package settings, add a trusted publisher entry for the exact workflow file that publishes that package.
3. Confirm that the trusted publisher entry points at this repository and the `npm-publish` environment expectations match the workflow.
4. Keep the workflow filename stable after registration, or update the npm trusted publisher entry before the next release.

Register this workflow filename for every covered trusted-publishing package:

- `.github/workflows/release-orchestration.yml` for the generic workspace npm packages listed above and for `hexo-renderer-typst`, `@myriaddreamin/rehype-typst`, `@myriaddreamin/vite-plugin-typst`, and `@myriaddreamin/highlighter-typst`

## Credential and approval boundaries

- generic workspace npm package publishing and npm project publishing: protected `npm-publish` environment plus GitHub OIDC with `id-token: write`
- `typst.node` npm publishing: existing dedicated release workflow credentials plus `GITHUB_TOKEN` with `contents: write` for GitHub Release asset uploads
- GHCR Docker publishing: `GITHUB_TOKEN` with `packages: write`
- Rust workspace crates and `projects/rustdoc-typst-demo`: `CRATES_IO_TOKEN`
