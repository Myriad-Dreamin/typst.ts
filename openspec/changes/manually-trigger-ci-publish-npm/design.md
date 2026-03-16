## Context

The repository already exposes the package release primitives from the root workspace:

- `yarn build:pkg`
- `yarn publish:dry`
- `yarn publish:lib`

Those commands are useful, but the current release surface is split across:

- regular CI that builds packages
- a dedicated `packages/typst.node` release workflow that still uses `NPM_TOKEN`
- a dedicated Docker image workflow that publishes to GHCR
- a dedicated crates publish workflow that uses `CRATES_IO_TOKEN`
- `scripts/publish-project.ps1`, which mixes npm publishes with a `cargo publish`

The result is that maintainers do not yet have one consistent manual CI story for publish operations across npm, containers, and Rust crates. At the same time, npm trusted publishing requires workflow-level decisions that are easy to get wrong later:

- live publish jobs need `id-token: write`
- npm currently requires Node 22.14.0 or newer and npm 11.5.1 or newer for trusted publishing
- npm validates the calling workflow identity, so the top-level workflow file used by `workflow_dispatch` matters

This change creates the proposal and implementation plan for a staged rollout instead of a single large migration.

## Goals / Non-Goals

**Goals:**

- Introduce a manual CI verification lane for publishable workspace packages.
- Introduce a manual CI verification lane for the publishable subprojects under `projects/`.
- Standardize new npm release workflows on GitHub OIDC trusted publishing instead of registry tokens.
- Add a manual CI release lane for the generic workspace npm packages.
- Add a separate manual CI release lane for the publishable subprojects under `projects/`, including `projects/rustdoc-typst-demo`.
- Add a coordinated manual release lane that can include `typst.node`, Docker image publishing, workspace crates, and Rust project publishing.
- Keep the rollout explicit and auditable through protected environments and workflow summaries.

**Non-Goals:**

- Replace ecosystem-specific tooling such as the NAPI build matrix, Docker buildx, or Cargo publish with one homogeneous publish job.
- Publish `projects/rustdoc-typst-demo` through npm-specific publish steps.
- Force Docker and Cargo publishing to use npm trusted publishing semantics.
- Auto-publish on push or tag creation. The new workflows are intentionally manual.

## Decisions

### 1. Use top-level manual workflows for live publish jobs

The live publish entry points will be top-level `workflow_dispatch` workflows rather than only reusable workflow calls.

Why:

- npm trusted publishing validates the workflow identity seen by GitHub Actions.
- A top-level workflow is simpler to register in npm trusted publisher settings and easier for maintainers to reason about.

Alternative considered:

- Reusable workflow-only publish orchestration.

Why not:

- It adds ambiguity around which workflow filename npm must trust for manual dispatch and future maintenance.

### 2. Split release execution into dedicated lanes, with an optional coordinated orchestrator

The change will use dedicated manual lanes for:

- package verification
- project verification
- workspace package release
- project release
- integrated multi-ecosystem release orchestration

Why:

- The workspace packages, `projects/*`, `typst.node`, Docker image, and Rust publish targets all have different build graphs, credentials, and release side effects.
- `scripts/publish-project.ps1` currently mixes npm and Cargo publishing, which is a poor fit for a single trusted publishing workflow.

Alternative considered:

- Flatten all release targets into one homogeneous publish implementation.

Why not:

- It makes approval, auditing, and failure handling harder.
- It increases the chance of mixing ecosystem-specific credentials and publish logic incorrectly.

### 3. Publish one version through one full release run

The release design assumes one coordinated release run per version, and that run publishes the full artifact set in scope rather than supporting partial package or project publishes.

Why:

- This repository versions related artifacts together.
- The maintainers have stated that partial publishes are not a user case.
- Fixed release scope simplifies verification, operator expectations, and trusted publisher setup.

Alternative considered:

- Support partial package, project, or ecosystem selection per run.

Why not:

- It adds complexity without a concrete user need.
- It increases the chance of creating incomplete releases for a shared version.

### 4. Use an explicit allowlist for the generic workspace package release lane

The workspace package release workflow will publish only the packages that are known to support the root `publish:dry` and `publish:lib` flow. The initial allowlist is:

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

Why:

- `packages/typst.node` has a dedicated NAPI publish flow and its root `publish:*` scripts are no-ops.
- `packages/typst.svelte` is intended to be part of the generic package lane and its missing `publish:*` scripts are a repo miss that this change should fix.
- `packages/enhanced-typst-svg` is private and not part of the publish lane.

Alternative considered:

- Infer publishability dynamically from all package manifests.

Why not:

- Deterministic release scope is more important than convenience for the first rollout.

### 5. Move all trusted-publishing workflows to Node 22.14.0+ and npm 11.5.1+

Any workflow created or migrated by this change will standardize on the runner requirements that npm trusted publishing currently expects.

Why:

- This removes one of the most common migration failures before maintainers start configuring trusted publishers in npm.
- It keeps verification and release lanes aligned.

Alternative considered:

- Keep existing Node 20 release runners until later.

Why not:

- It would force a second migration step before trusted publishing can actually work.

### 6. Add a dedicated project verification stage before live project publish

The project release lane will verify all covered publishable subprojects under `projects/` before any live project publish job runs.

Why:

- The desired release sequence is verify packages and projects first, then publish packages, then publish projects.
- Project verification is different from project publishing and deserves its own clear failure boundary.

Alternative considered:

- Rely only on live publish jobs to surface project build or pack errors.

Why not:

- It weakens the release gate.
- It gives maintainers less confidence before live publishing begins.

### 7. Publish all versioned subprojects under `projects/` in one project release batch

The manual project publish workflow will cover the versioned publishable subprojects under `projects/`:

- `hexo-renderer-typst`
- `@myriaddreamin/rehype-typst`
- `@myriaddreamin/vite-plugin-typst`
- `@myriaddreamin/highlighter-typst`
- `projects/rustdoc-typst-demo`

Why:

- The repository treats these subprojects as part of the same versioned release story.
- `projects/rustdoc-typst-demo` is a subproject and should be published alongside the other projects, while still using Cargo publish commands.

Alternative considered:

- Keep `projects/rustdoc-typst-demo` outside the project release batch as a separate Rust-only lane.

Why not:

- It creates unnecessary release fragmentation for one versioned project set.
- The project batch can still preserve project-specific publish commands without pretending every project is npm-only.

### 8. Integrate `typst.node`, Docker, and Rust releases through orchestration, not by folding them into the generic npm lanes

The fifth release proposal will add a coordinated manual release workflow that can include:

- the dedicated `typst.node` NAPI release path
- the GHCR Docker image publish path
- the Rust workspace crates publish path
- the project release batch, which includes `projects/rustdoc-typst-demo`

Why:

- These targets belong in the broader release story, but they do not fit the generic root npm publish commands.
- `typst.node` has a dedicated build, test, artifact movement, publish, and GitHub Release upload sequence.
- Docker image publishing and Cargo publishing use different authentication models from npm trusted publishing.

Alternative considered:

- Leave `typst.node`, Docker, and Rust publishing permanently outside the new release plan.

Why not:

- Maintainers would still have to stitch together several unrelated workflows by hand for a real release.
- The repo would continue to have multiple competing "main" release paths.

### 9. Preserve ecosystem-specific authentication inside the coordinated workflow

The integrated release proposal will keep credentials and permissions scoped by ecosystem:

- generic npm packages and `typst.node` use npm trusted publishing and GitHub OIDC where applicable
- GHCR publishing uses `GITHUB_TOKEN` with `packages: write`
- crates.io publishing uses `CRATES_IO_TOKEN`

Why:

- Each registry has different trust and permission requirements.
- Reusing the wrong authentication mechanism would make the workflow fragile and harder to audit.

Alternative considered:

- Try to normalize all publish targets onto one shared authentication strategy.

Why not:

- The registries do not support the same trust model.
- Security boundaries are clearer when each lane uses the credential model intended for that ecosystem.

### 10. Reuse existing `typst.node`, Docker, and Rust workflows through `workflow_call`

The coordinated workflow will reuse the existing `typst.node`, Docker, and Rust release workflows through `workflow_call` instead of re-implementing their internal job logic in a new file.

Why:

- The existing workflows already encode ecosystem-specific build and publish behavior.
- Reuse keeps the orchestration layer thin and reduces drift between standalone and coordinated release paths.

Alternative considered:

- Inline the current `typst.node`, Docker, and Rust release jobs directly into the new coordinated workflow.

Why not:

- It duplicates release logic that already exists.
- It makes future changes harder to keep consistent.

### 11. Treat `typst.node` as a dedicated release lane under the integrated workflow

The coordinated workflow will include `typst.node` as a dedicated lane within the full release run, but it will retain its dedicated multi-platform build and test behavior instead of running through the root package publish commands.

Why:

- The current `typst.node` flow builds platform artifacts, downloads them, repackages them, uploads GitHub Release assets, and publishes multiple npm packages.
- Its root `publish:dry` and `publish:lib` scripts are intentionally no-ops, so generic package workflows are the wrong abstraction.

Alternative considered:

- Force `typst.node` through the root `yarn publish:lib` lane.

Why not:

- That would skip the build/test/repackage structure the package relies on today.
- It would hide the fact that `typst.node` publishes multiple packages and release artifacts, not a single npm package.

## Risks / Trade-offs

- [npm trusted publisher setup per package] -> Maintain a checklist of the package names and trusted workflow filenames before enabling live publish.
- [Workflow sprawl] -> Keep one workflow per lane and document the purpose of each in the workflow names and summaries.
- [Package scope drift] -> Use explicit allowlists and workflow summaries so maintainers can see exactly what is in or out of scope.
- [Runner version drift] -> Pin Node to a compliant version in the new workflows and verify npm version before publish.
- [Cross-ecosystem orchestration failures] -> Enforce the fixed release order and stop later publish jobs when prerequisite lanes fail.
- [`typst.node` release complexity] -> Keep the NAPI build/test/publish flow isolated as its own lane even when called from the coordinated workflow.
- [Mixed credential usage] -> Scope permissions per job and keep Docker, npm, and Cargo credentials isolated.
- [Reusable workflow contract drift] -> Keep workflow inputs and outputs explicit so the coordinator can call reusable workflows without depending on undocumented behavior.

## Migration Plan

1. Add the manual package verification workflow and confirm that the current package lane is stable on a trusted-publishing-compatible Node runner.
2. Add the manual project verification workflow for the publishable subprojects under `projects/`, including `projects/rustdoc-typst-demo`.
3. Add the environment, permissions, and documentation needed for npm trusted publishing.
4. Add the missing `publish:*` scripts for `packages/typst.svelte` and include it in the generic workspace package release lane.
5. Add the manual workspace package release workflow using the explicit allowlist and verification gate.
6. Add the manual project release workflow, and gate it behind successful package and project verification plus successful package publishing.
7. Add the coordinated multi-ecosystem release workflow that executes one full release run per version.
8. Reuse the existing `typst.node`, Docker, and Rust publish workflows from that coordinator through `workflow_call`.
9. Update maintainer documentation with the trusted publisher setup steps, workflow order, and the packages or artifacts covered by each lane.

Rollback strategy:

- Disable the new manual workflows or remove the trusted publisher entries from npm if a release issue is found.
- Fall back to the existing local/manual publish steps until the workflow is fixed.

## Open Questions

None currently.
