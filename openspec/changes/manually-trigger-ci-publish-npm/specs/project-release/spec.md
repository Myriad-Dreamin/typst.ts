## ADDED Requirements

### Requirement: Manual project release workflow
The repository SHALL provide a manually triggered GitHub Actions workflow for publishing the versioned publishable subprojects under `projects/` as one release batch for a version.

#### Scenario: Maintainer publishes the project batch
- **WHEN** a maintainer dispatches the project release workflow
- **THEN** the workflow publishes `hexo-renderer-typst`, `@myriaddreamin/rehype-typst`, `@myriaddreamin/vite-plugin-typst`, `@myriaddreamin/highlighter-typst`, and `projects/rustdoc-typst-demo`

### Requirement: Project release workflow shall depend on successful verification and package publishing
The project release workflow SHALL run only after the package verification stage, the project verification stage, and the package publish stage complete successfully.

#### Scenario: A prerequisite stage fails before project publish
- **WHEN** package verification, project verification, or package publishing fails
- **THEN** the project publish stage does not run

### Requirement: Project release workflow shall run required build steps
The project release workflow SHALL run project-specific build steps before publish for any covered project that requires a build artifact.

#### Scenario: Covered project requires a build before publish
- **WHEN** the release workflow prepares `@myriaddreamin/highlighter-typst` or `@myriaddreamin/vite-plugin-typst` for publish
- **THEN** the workflow runs that project's build command before npm publish
- **AND** the workflow fails if the build command fails

### Requirement: Project release workflow shall use project-specific publish commands
The project release workflow SHALL use the publish command appropriate for each covered project instead of assuming every project uses npm publish.

#### Scenario: Workflow reaches rustdoc-typst-demo in the project batch
- **WHEN** the project release workflow prepares `projects/rustdoc-typst-demo` for publish
- **THEN** the workflow runs the Cargo publish command for that subproject

### Requirement: Project release workflow shall exclude unpublished project entries
The project release workflow MUST NOT include unpublished or private project entries that are outside the versioned project batch.

#### Scenario: Workflow evaluates an unpublished project entry
- **WHEN** the project release workflow considers a private project such as `@myriaddreamin/vistyp`
- **THEN** the workflow excludes that project from the release batch
