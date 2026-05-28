## Why

The repository already has local publish commands for workspace packages and selected projects, but the release paths are inconsistent across ad hoc scripts and workflows. npm trusted publishing is now generally available for GitHub Actions, so this is the right time to replace registry-token-based npm publishing with a manual, auditable CI flow that matches the repo's existing build and release structure.

## What Changes

- Add a manual package verification workflow that runs the existing package build and dry-run publish steps in CI before release.
- Add a manual project verification workflow that validates the publishable subprojects under `projects/`, including `projects/rustdoc-typst-demo`, before any live project publish runs.
- Add shared npm trusted publishing conventions for new or migrated npm publish workflows, including OIDC, manual dispatch, protected environments, and supported Node/npm versions.
- Add a manual workspace package release workflow for the generic `packages/*` npm packages that can publish through the existing root publish flow.
- Add a manual project release workflow for the publishable subprojects under `projects/`, including `projects/rustdoc-typst-demo`.
- Add a fifth coordinated release proposal that integrates `packages/typst.node`, the Docker image publish path, and Rust package publishing into the broader manual release workflow while preserving ecosystem-specific authentication and validation.

## Capabilities

### New Capabilities
- `package-publish-verification`: Verify publishable workspace packages in CI by running the build and dry-run publish steps before release.
- `project-publish-verification`: Verify the publishable subprojects under `projects/`, including `projects/rustdoc-typst-demo`, before any live project publish runs.
- `npm-trusted-publishing`: Define the GitHub Actions and npm configuration required for trusted publishing without `NPM_TOKEN`.
- `workspace-package-release`: Manually publish the generic workspace npm packages from CI after verification succeeds.
- `project-release`: Manually publish the versioned subprojects under `projects/`, including the Rust subproject `projects/rustdoc-typst-demo`, from CI while preserving project-specific publish commands.
- `integrated-release-orchestration`: Coordinate `typst.node`, Docker image publishing, and Rust package publishing under the broader manual release workflow without collapsing their ecosystem-specific requirements.

### Modified Capabilities

None.

## Impact

- Affects GitHub Actions workflows under `.github/workflows/`.
- Affects release process documentation and maintainer runbooks.
- Requires npm trusted publisher setup for the covered npm packages.
- Affects the existing `packages/typst.node` NAPI release workflow, Docker image publishing, workspace crate publishing, and Rust project publishing.
