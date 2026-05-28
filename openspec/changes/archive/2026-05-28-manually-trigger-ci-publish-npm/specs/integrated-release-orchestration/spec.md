## ADDED Requirements

### Requirement: Manual integrated release orchestration workflow
The repository SHALL provide a manually triggered GitHub Actions workflow that coordinates the broader release process across the dedicated npm, `typst.node`, Docker, and Rust release lanes.

#### Scenario: Maintainer starts one full release run for one version
- **WHEN** a maintainer dispatches the integrated release workflow
- **THEN** the workflow runs one coordinated release for the target version
- **AND** the workflow does not expose partial package or project selection for that release run

### Requirement: Integrated workflow shall enforce the fixed npm release order
The integrated release workflow SHALL enforce the npm release order of verifying packages and projects first, then publishing packages, then publishing projects.

#### Scenario: Integrated workflow reaches the live publish stages
- **WHEN** the integrated release workflow reaches live publishing
- **THEN** package verification and project verification have already completed successfully
- **AND** workspace package publishing completes before project publishing begins

### Requirement: Integrated workflow shall preserve the dedicated `typst.node` release behavior
The integrated release workflow SHALL execute `typst.node` through its dedicated multi-platform build, test, packaging, and publish sequence instead of the generic workspace package publish path.

#### Scenario: Maintainer includes `typst.node` in an integrated release
- **WHEN** the integrated release workflow reaches the dedicated `typst.node` package lane
- **THEN** the workflow runs the required `typst.node` build and test stages before publish
- **AND** publishes the `typst.node` npm packages through the dedicated release path
- **AND** uploads the expected GitHub Release assets produced by that path

### Requirement: Integrated workflow shall support Docker image publishing
The integrated release workflow SHALL support the existing Docker image release target and preserve multi-platform publishing to GHCR.

#### Scenario: Maintainer includes Docker image publishing in an integrated release
- **WHEN** the integrated release workflow reaches the Docker release lane
- **THEN** the workflow builds the configured image for the supported platforms
- **AND** pushes the image to GHCR using the repository package publishing permissions

### Requirement: Integrated workflow shall support Rust workspace crate publishing
The integrated release workflow SHALL support the Rust workspace crates publish path without routing workspace crate publishing through the project publish lane.

#### Scenario: Maintainer includes Rust publishing in an integrated release
- **WHEN** the integrated release workflow reaches the Rust release lane
- **THEN** the workflow runs the configured Cargo verification or dry-run stage before live publish
- **AND** publishes the configured Rust workspace crates using the Cargo release path
- **AND** does not route those crates through the project publish workflow

### Requirement: Integrated workflow shall reuse dedicated workflows through `workflow_call`
The integrated release workflow SHALL reuse the dedicated `typst.node`, Docker, and Rust release workflows through `workflow_call`.

#### Scenario: Integrated workflow calls an ecosystem-specific release lane
- **WHEN** the integrated release workflow reaches the dedicated `typst.node`, Docker, or Rust release lane
- **THEN** it invokes the existing reusable workflow for that lane through `workflow_call`

### Requirement: Integrated workflow shall isolate ecosystem-specific credentials
The integrated release workflow SHALL scope credentials and permissions by release lane so that npm, GHCR, GitHub Release uploads, and crates.io publishing use only the permissions required for that lane.

#### Scenario: Integrated workflow runs multiple ecosystems in one release
- **WHEN** the integrated release workflow coordinates npm, GHCR, GitHub Release upload, and crates.io publishing in one run
- **THEN** each lane uses only the credentials and permissions required for that ecosystem
- **AND** a failure in one lane does not cause another lane to inherit broader credentials than it needs
