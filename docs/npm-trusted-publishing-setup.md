# npm Trusted Publishing Setup Runbook

This runbook records the external npm and GitHub configuration required before
the manual release workflow can publish the npm packages covered by
`release-orchestration.yml`.

This is intentionally separate from `release-workflows.md`: the workflow file is
repository state, while trusted publisher registrations and protected
environment settings live in external accounts and must be recreated or audited
by maintainers.

Last checked against npm documentation: 2026-05-28.

Official references:

- npm trusted publishers: <https://docs.npmjs.com/trusted-publishers/>
- `npm trust` CLI: <https://docs.npmjs.com/cli/v11/commands/npm-trust/>

## Required Access

The maintainer performing this setup needs:

- GitHub admin or maintain access on `Myriad-Dreamin/typst.ts`, enough to manage
  repository environments.
- npm write access on every npm package that the release workflows publish or
  are expected to publish after an explicit release-scope change.
- npm account-level 2FA enabled.
- For CLI setup, `npm@11.10.0` or newer. The release workflow itself pins a
  trusted-publishing-compatible runtime: Node `22.14.0` and npm `11.5.1`.

## GitHub Environment

Create or audit this GitHub repository environment:

```text
Repository: Myriad-Dreamin/typst.ts
Settings: Environments
Environment name: npm-publish
```

Recommended environment protection:

- Add required reviewers from the release maintainer group.
- Restrict deployment branches or tags to the project's release policy.
- Keep the environment name exactly `npm-publish`; the workflow jobs reference
  that name directly.

The npm publish jobs using this environment are reached from
`.github/workflows/release-orchestration.yml`:

- `publish-packages`
- `publish-typst-node`, which calls `.github/workflows/release-node.yaml`
- `publish-npm-projects`

## Trusted Publisher Values

For each npm package in scope, create one GitHub Actions trusted publisher with
these exact values:

```text
Provider: GitHub Actions
Organization or user: Myriad-Dreamin
Repository: typst.ts
Workflow filename: release-orchestration.yml
Environment name: npm-publish
Allowed actions: npm publish
```

Important details:

- npm asks for the workflow filename only. Use `release-orchestration.yml`, not
  `.github/workflows/release-orchestration.yml`.
- `release-node.yaml` is a reusable workflow when it is called by
  `release-orchestration.yml`. npm validates the caller workflow name for
  reusable workflow publishes, so the trusted publisher entry for `typst.node`
  packages also uses `release-orchestration.yml`.
- `release-node.yaml` intentionally has no standalone live publish trigger. Use
  `release::orchestrate` as the live publish entry point for `typst.node`.
- The workflow file must exist in `.github/workflows/` on the branch/tag being
  released.
- `Allowed actions` must include `npm publish`. Do not choose only staged
  publishing unless the release workflow is changed to run `npm stage publish`.
- npm currently allows one trusted publisher configuration per package. If a
  package already has a different trusted publisher, inspect it before replacing
  it.
- The npm CLI can only configure trusted publishing for packages that already
  exist and that the current npm account can manage. If a future package is not
  present in npm yet, leave a setup note and configure it before the first
  publish that includes it.

## Package Scope

Do not duplicate the package list in this document. The authoritative npm
package scope lives in the release implementation:

- Generic workspace npm packages are selected by the `publish-packages` job in
  `.github/workflows/release-orchestration.yml`.
- Dedicated native Node packages are selected by `.github/workflows/release-node.yaml`
  and the package manifests under its release directory.
- Project npm packages are selected by the `publish-npm-projects` job in
  `.github/workflows/release-orchestration.yml`.
- Any currently private package entry must have an explicit release-scope change
  before it is added to a live publish job.

When auditing trusted publisher setup, derive the package names from the current
workflow jobs and their package manifests, then configure each npm package with
the values in "Trusted Publisher Values".

## Non-npm Release Targets

Do not create npm trusted publishers for non-npm release targets. Cargo crates,
container images, and GitHub Release assets remain part of the release trust
boundary, but they are controlled by their own registry credentials or GitHub
permissions rather than npm trusted publishers.

## Private Package Warning

Some package manifests may contain `"private": true` even when maintainers want
to track their future trusted-publishing setup.

That means:

- You can treat them as intended future trusted-publishing targets and keep them
  in the npm setup audit.
- A real `npm publish` will not succeed until the package manifest and release
  workflow are deliberately changed to publish the package.
- Do not remove `"private": true` casually; doing so changes the public release
  surface.

## Web Setup

For each package:

1. Open the package on <https://www.npmjs.com/>.
2. Go to `Settings`.
3. Find `Trusted Publisher` or `Trusted publishing`.
4. Select `GitHub Actions`.
5. Enter the values from "Trusted Publisher Values".
6. Save the package settings.
7. Reopen the trusted publisher section and confirm the saved values match.

Repeat this for every npm package discovered from the current release scope.

## CLI Setup

Use the CLI when configuring many packages at once. The command requires
`npm@11.10.0` or newer and npm write access on each package. Keep any local bulk
script outside this repository so the package list remains derived from the
current release workflow instead of copied into documentation.

Command template:

```sh
npm trust github <package> \
  --repo Myriad-Dreamin/typst.ts \
  --file release-orchestration.yml \
  --env npm-publish \
  --allow-publish
```

Verify a package after setup:

```sh
npm trust list <package>
```

If a package already has a trusted publisher and must be replaced, inspect it
first:

```sh
npm trust list <package>
npm trust revoke --id <trust-id> <package>
```

Then recreate it with the expected values.

## First Release After Setup

Before the first live publish:

1. Confirm `npm-publish` exists in GitHub and has the intended required
   reviewers.
2. Confirm every npm package discovered from the current release scope has the
   trusted publisher values from this document.
3. Confirm the release branch or tag contains `.github/workflows/release-orchestration.yml`.
4. Confirm the workflow still grants `id-token: write` to `publish-packages`,
   `publish-typst-node`, and `publish-npm-projects`.
5. Confirm the publishing jobs still install Node `22.14.0` or newer and npm
   `11.5.1` or newer.
6. Trigger `release::orchestrate` manually from GitHub Actions with the release
   tag.
7. Review and approve the `npm-publish` environment deployment when prompted.
8. After the run, confirm the published npm package pages show the expected new
   versions and provenance.

## Token Migration Warning

Do not immediately revoke every npm automation token after configuring the
trusted publishers.

The workspace package, `typst.node`, and project npm publish jobs in
`release-orchestration.yml` should not need `NPM_TOKEN`. Revoke or disallow
traditional npm tokens only after the release owner confirms no remaining npm
release path still depends on them.

npm's recommended order for migration is:

1. Configure trusted publishers.
2. Verify a real publish works.
3. Restrict traditional token publishing for packages that no longer need it.
4. Revoke unneeded automation tokens.

## If The Workflow Is Renamed

If `.github/workflows/release-orchestration.yml` is renamed, moved, split, or
replaced:

1. Update this runbook.
2. Update `docs/release-workflows.md`.
3. Update every npm trusted publisher registration before the next live publish.
4. Recheck the exact workflow filename npm expects. For GitHub Actions, npm wants
   only the file name, not the `.github/workflows/` path.
