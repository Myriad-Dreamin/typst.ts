## ADDED Requirements

### Requirement: Manual project verification workflow
The repository SHALL provide a manually triggered GitHub Actions workflow that verifies the publishable subprojects under `projects/` by running the required build and dry-run publish, pack, or Cargo verification steps for the covered project batch.

#### Scenario: Maintainer verifies projects before release
- **WHEN** a maintainer dispatches the project verification workflow for a selected ref
- **THEN** the workflow checks out that ref
- **AND** installs the required dependencies
- **AND** runs the required build steps for the covered project batch
- **AND** runs the project dry-run publish, pack, or Cargo verification step for each covered project
- **AND** fails if any covered project cannot complete its verification steps

### Requirement: Project verification scope shall be fixed
The project verification workflow SHALL verify the project batch `hexo-renderer-typst`, `@myriaddreamin/rehype-typst`, `@myriaddreamin/vite-plugin-typst`, `@myriaddreamin/highlighter-typst`, and `projects/rustdoc-typst-demo` as one release batch for a version.

#### Scenario: Maintainer runs project verification
- **WHEN** a maintainer dispatches the project verification workflow
- **THEN** the workflow verifies all covered projects for that version
- **AND** the workflow does not expose partial project selection
