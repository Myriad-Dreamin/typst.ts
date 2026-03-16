## ADDED Requirements

### Requirement: Covered npm publish workflows shall use trusted publishing
Any npm publish workflow created or migrated by this change SHALL use GitHub Actions trusted publishing instead of an npm registry token.

#### Scenario: Live publish job authenticates to npm
- **WHEN** a covered live publish workflow starts its npm publish step
- **THEN** the workflow requests `id-token: write`
- **AND** the workflow does not require `NPM_TOKEN`
- **AND** the workflow does not write a registry authentication token into `.npmrc`

### Requirement: Covered publish workflows shall use trusted-publishing-compatible runtimes
Any npm publish workflow created or migrated by this change SHALL run its live publish job with Node 22.14.0 or newer and npm 11.5.1 or newer.

#### Scenario: Maintainer runs a covered publish workflow
- **WHEN** a maintainer dispatches a covered live publish workflow
- **THEN** the workflow configures Node 22.14.0 or newer before publish
- **AND** the workflow uses npm 11.5.1 or newer for the publish command

### Requirement: Live npm publish shall be manually gated
The repository SHALL gate live npm publish jobs behind a top-level `workflow_dispatch` entry point and a protected GitHub Environment.

#### Scenario: Publish job waits for approval
- **WHEN** a maintainer dispatches a live npm publish workflow
- **THEN** GitHub requires the configured environment approval before the workflow runs npm publish steps
