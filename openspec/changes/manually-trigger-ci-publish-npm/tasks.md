## 1. Package Verification Workflow

- [ ] 1.1 Add a manual GitHub Actions workflow for workspace package verification on a trusted-publishing-compatible Node runner.
- [ ] 1.2 Run the existing package build and dry-run publish commands in that workflow and fail on any in-scope package error.
- [ ] 1.3 Add a workflow summary or artifact that lists the packages included in the generic lane and the packages intentionally skipped.

## 2. Project Verification Workflow

- [ ] 2.1 Add a manual GitHub Actions workflow that verifies the publishable subprojects under `projects/` as one release batch, including `projects/rustdoc-typst-demo`.
- [ ] 2.2 Run the required project build steps and dry-run publish, pack, or Cargo verification commands in that workflow.
- [ ] 2.3 Fail the workflow on any covered project verification error and summarize the verified project set.

## 3. Trusted Publishing Groundwork

- [ ] 3.1 Define the top-level manual workflow filenames and protected GitHub Environment used for live npm publishing.
- [ ] 3.2 Update the new or migrated publish workflows to use `id-token: write`, Node 22.14.0 or newer, and npm 11.5.1 or newer.
- [ ] 3.3 Remove `NPM_TOKEN`-based authentication from the workflows covered by this change and document the npm trusted publisher setup required per package.

## 4. Workspace Package Release Workflow

- [ ] 4.1 Create a manual workspace package release workflow for the generic `packages/*` npm release lane.
- [ ] 4.2 Add the missing `publish:*` scripts for `packages/typst.svelte` and include it in the generic package allowlist.
- [ ] 4.3 Encode the explicit package allowlist for the generic lane and exclude `@myriaddreamin/typst-ts-node-compiler` and `enhanced-typst-svg`.
- [ ] 4.4 Gate live workspace publishing on the successful package verification stage and expose a clear release summary.

## 5. Project Release Workflow

- [ ] 5.1 Create a manual project release workflow for `hexo-renderer-typst`, `@myriaddreamin/rehype-typst`, `@myriaddreamin/vite-plugin-typst`, `@myriaddreamin/highlighter-typst`, and `projects/rustdoc-typst-demo` as one release batch.
- [ ] 5.2 Gate live project publishing on successful package verification, project verification, and package publishing.
- [ ] 5.3 Add project-specific build steps so `@myriaddreamin/highlighter-typst` and `@myriaddreamin/vite-plugin-typst` build before publish.
- [ ] 5.4 Use project-specific publish commands so `projects/rustdoc-typst-demo` publishes through Cargo while the npm projects use their npm publish paths.
- [ ] 5.5 Exclude unpublished or private projects such as `@myriaddreamin/vistyp` from the release batch.

## 6. Integrated Typst.Node, Docker, And Rust Release Workflow

- [ ] 6.1 Create a coordinated manual release workflow that runs one full release per version without partial package or project selection.
- [ ] 6.2 Enforce the fixed order of package verification, project verification, package publishing, and project publishing in that workflow.
- [ ] 6.3 Reuse the existing `typst.node`, Docker, and Rust release workflows from the coordinator through `workflow_call`.
- [ ] 6.4 Integrate the existing `typst.node` build, test, artifact, npm publish, and GitHub Release upload sequence into the package-release stage without forcing it through the generic package lane.
- [ ] 6.5 Integrate the Docker image publish path and Rust workspace crate publish paths into the coordinated workflow with ecosystem-specific permissions and verification.

## 7. Documentation And Follow-up Scope

- [ ] 7.1 Document which packages, container artifacts, and Rust publish targets are covered by each workflow.
- [ ] 7.2 Record the npm trusted publisher registration steps for maintainers, including the workflow filenames and environment expectations.
- [ ] 7.3 Document the fixed workflow order and which credentials and approval boundaries apply to npm, GHCR, GitHub Release uploads, and crates.io.
