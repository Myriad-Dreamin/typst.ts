## ADDED Requirements

### Requirement: Manual workspace package release workflow
The repository SHALL provide a manually triggered GitHub Actions workflow that publishes the generic workspace npm packages after verification succeeds.

#### Scenario: Maintainer publishes the workspace package lane
- **WHEN** a maintainer dispatches the workspace package release workflow for a selected ref
- **THEN** the workflow checks out that ref
- **AND** runs the verification gate required for the workspace package lane
- **AND** publishes the in-scope workspace packages through the root package release path or an equivalent deterministic fan-out

### Requirement: Workspace package release scope shall use an explicit allowlist
The generic workspace package release lane SHALL publish only the packages that are explicitly allowlisted for the root package publish flow.

#### Scenario: Workflow evaluates package scope for live publish
- **WHEN** the workspace package release workflow determines which packages to publish
- **THEN** it includes only the allowlisted generic workspace packages
- **AND** it excludes packages with dedicated publish flows or missing generic publish support
- **AND** the exclusion set includes `@myriaddreamin/typst-ts-node-compiler` and `enhanced-typst-svg`

### Requirement: Workspace package lane shall include typst.svelte
The generic workspace package release lane SHALL include `@myriaddreamin/typst.svelte` after adding the missing `publish:*` scripts needed for the root package publish flow.

#### Scenario: Workflow reaches typst.svelte during verification or live publish
- **WHEN** the generic workspace package lane evaluates `@myriaddreamin/typst.svelte`
- **THEN** the package participates in the same verification and live publish flow as the other allowlisted generic workspace packages

### Requirement: Live workspace publish shall depend on successful verification
The workspace package release workflow SHALL stop before npm publish if the verification stage fails.

#### Scenario: Verification fails before live publish
- **WHEN** the workspace package verification stage fails
- **THEN** the live npm publish stage does not run
- **AND** the workflow result clearly reports that publishing was blocked by verification failure
