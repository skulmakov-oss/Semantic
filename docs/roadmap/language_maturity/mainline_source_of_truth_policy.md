# Mainline Source Of Truth Policy

Status: active repository governance rule

## Goal

Keep post-stable work disciplined after the repository has accumulated many
historical feature branches, temporary worktrees, and archived validation
artifacts.

This policy exists to prevent stale branch state from becoming an accidental
second source of truth.

## Canonical Working Rule

For current development (post-C1, after the `v1.2.0` Stable Foundation
release), the only active source of truth is:

- clean `main`
- in the canonical working repository
- with a clean tracked-file status before opening the next step

In the current setup, that means:

- repository: `skulmakov-oss/Semantic`
- branch: `main`

## What This Means In Practice

- historical feature branches are retained as history, not as active planning
  surfaces
- temporary worktrees may be used for verification or isolation, but must not
  become the planning baseline for the next track
- archived release evidence and temporary `target-*` directories are not part
  of the repository source of truth
- roadmap docs, code, tests, and release-facing statements must be read from
  current `main`, not from stale branch snapshots

## Required Sanity Check Before New Work

Before opening a new track or slice:

1. confirm the active branch is `main`
2. confirm `git status --short` is empty for tracked files
3. confirm the intended source of truth is the canonical working repository
4. only then open the next scope checkpoint or code slice

## Non-Goals

This policy does not:

- delete historical branches
- forbid temporary worktrees for verification
- claim that the published `v1.2.0` release (C1) and current `main` are the
  same contract

It only fixes the active engineering baseline for new work.
