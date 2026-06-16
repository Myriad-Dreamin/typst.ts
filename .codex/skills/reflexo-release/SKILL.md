---
name: reflexo-release
description: Guide Reflexo/typst.ts release preparation and operator handoffs. Use when the user asks about 发版, release, publish, version bumping, release dry-runs, npm trusted publishing, GitHub Release artifacts, Docker/crates publishing, or wants a guide/checklist agent for this repository's release process.
---

# Reflexo Release Guide

## Purpose

Guide a release operator through the current repository release process without copying release workflow contents into this skill. Treat the repository as the source of truth every time.

Do not perform a live publish, create a tag, push, approve an environment, revoke credentials, or change package visibility unless the user explicitly asks for that action.

## Ground Rules

- Read current release docs, GitHub Actions workflows, package manifests, changelog files, and release scripts before giving concrete steps.
- Do not hardcode package lists, workflow job lists, line numbers, token names beyond what is found in the current checkout.
- Summarize source-derived facts in your answer, but avoid pasting workflow contents or long file excerpts.
- Prefer an operator checklist over a narrative explanation.
- Separate local preparation, remote GitHub Actions steps, external account setup, and post-release verification.
- State whether each step is advisory, a dry-run, or a live-publish action.
- If the user asks for execution, confirm the current git state and target version/tag before making changes.

## Discovery Pass

Before advising, gather the current facts:

```sh
git status --short
rg -n "release|publish|trusted|workflow_dispatch|tag|dry-run|crate|docker|npm" docs .github scripts package.json Cargo.toml openspec --glob '!node_modules' --glob '!target'
rg --files .github docs scripts openspec packages projects crates --glob '!node_modules' --glob '!target' --glob '!*dist*'
```

Then inspect only the relevant files found by that search. Prioritize:

- Maintainer release documentation.
- Top-level release orchestration workflow.
- Dedicated workflows called by the orchestration workflow.
- Tag-driven or GitHub Release artifact workflows.
- npm trusted-publishing runbooks.
- Version bumping, announcement, and publish wrapper scripts.
- Package manifests and Cargo manifests that participate in the release.

When reporting facts, cite file paths and avoid embedding their contents.

## Operator Workflow

Use this structure for release guidance.

### 1. Define The Target

Clarify or infer:

- Release version and tag.
- Whether this is dry-run, prerelease, nightly, or live release.
- Whether the user wants a checklist, local prep edits, workflow dispatch commands, or end-to-end execution.
- Whether package publication, GitHub Release artifacts, Docker, and Cargo crates are all in scope according to current workflows.

If the target version/tag is not clear and action would mutate files or publish artifacts, ask a concise question before proceeding.

### 2. Prepare The Repository

Build a prep checklist from current repository files:

- Confirm clean or intentionally dirty git state.
- Confirm the current version and target version across workspace/package/Cargo manifests.
- Inspect the version bump script, but do not assume it covers every release manifest.
- Update changelog and release notes if requested.
- Check lockfiles after dependency or manifest edits.
- Run the repo's normal formatting/build/test gates appropriate to the change size when requested or when making release-prep edits.

For actual edits, keep changes scoped to release metadata unless the user explicitly asks for release automation changes.

### 3. Verify Remote Preconditions

List checks the human maintainer must confirm in GitHub, npm, GitHub Container Registry, and crates.io. Derive exact environment names, workflow names, secrets, and permission boundaries from current docs/workflows.

Include these categories when they exist in the current release flow:

- Protected GitHub environments and required reviewers.
- npm trusted publisher registration and provenance requirements.
- Registry tokens or secrets for ecosystems that do not use npm trusted publishing.
- GitHub Release tag/draft expectations before uploading assets.
- Branch/tag restrictions.

Do not claim an external setting is configured unless you have observed it through a tool or the user confirms it.

### 4. Dry-Run Or Verification Stage

Identify the safest non-publishing validation route from current workflows. If there is a dedicated dry-run input, explain how to use it. If not, list the current verify jobs or local dry-run commands found in the repo.

When preparing a dispatch command, prefer showing it as an operator command unless the user explicitly asks you to run it. Treat GitHub Actions dispatch as a remote action and explain what it will do.

### 5. Live Release Stage

Present live steps in dependency order as discovered from the current orchestration workflow. Clearly mark:

- manual workflow dispatches,
- approval points,
- reusable workflow calls,
- artifact upload stages,
- registry publish stages,
- steps that are safe to rerun because already-published versions are skipped,
- steps that are not clearly idempotent.

If the repository has both tag-driven artifact workflows and a manual publish orchestrator, describe their relationship and required order without merging them into one imaginary workflow.

### 6. Post-Release Verification

After a live release, guide the operator to verify:

- GitHub Release exists, has expected artifacts, and is draft/non-draft as intended.
- npm package versions and provenance are visible.
- Docker image tags exist when Docker is in scope.
- crates.io packages exist when Cargo publishing is in scope.
- Workflow summaries show expected included/skipped entries.
- Any failed or skipped jobs have an understood reason.

Use current registry/package names derived from manifests and workflow summaries, not names stored in this skill.
