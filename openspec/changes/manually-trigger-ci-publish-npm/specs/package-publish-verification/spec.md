## ADDED Requirements

### Requirement: Manual package verification workflow
The repository SHALL provide a manually triggered GitHub Actions workflow that verifies the generic workspace package release lane by running the package build and package dry-run publish steps from the repository root for the packages in scope.

#### Scenario: Maintainer verifies workspace packages before release
- **WHEN** a maintainer dispatches the package verification workflow for a selected ref
- **THEN** the workflow checks out that ref
- **AND** installs workspace dependencies
- **AND** runs the workspace package build step
- **AND** runs the workspace package dry-run publish step for the in-scope packages
- **AND** fails if any in-scope package cannot build or complete the dry-run publish step

### Requirement: Verification scope shall be explicit
The package verification workflow SHALL report which workspace packages are included in the generic package lane and which packages are skipped because they require a different release path or do not expose the generic publish commands.

#### Scenario: Workflow reaches an excluded package
- **WHEN** the verification workflow evaluates a package such as `@myriaddreamin/typst-ts-node-compiler` or `enhanced-typst-svg`
- **THEN** the workflow does not attempt the generic package dry-run publish step for that package
- **AND** the workflow summary records that the package was skipped
- **AND** the workflow summary records why the package was skipped
