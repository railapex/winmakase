# Slice handoff template

Copy into the next build's evidence directory. Fill from actual state; delete unused fields.

## Contract

- Slice ID / parent package / requirement IDs:
- Outcome and acceptance cases:
- Explicit exclusions:
- Base repository, branch, commit and integration contract:
- Worker worktree / owned paths / shared interfaces:
- Requested model and effort; observed actual model:
- Authorized actions and test environment; live-change boundary:
- Budget/time limit if explicitly set by the run:
- Dependencies verified against:

## Build return

- Commit(s) and actual changed behavior:
- Required commands, runtime/build/config, executed counts and evidence paths:
- Regression failed on prior behavior where applicable:
- Unverified cases / failed checks / unresolved defects:
- Independent reviewer and disposition:
- Integration checks and accepting owner:

## Checkpoint

- Actual HEAD / clean status / claims:
- Running processes/tasks and exact owner; cleanup needed:
- Decisions for Chris (options, recommendation, consequence, blocking scope):
- Next executable action and its prerequisites:
- Next recenter trigger:
- Report regenerated from source:
- Prior checkpoint reference:

A return with unresolved required tests is incomplete even when implementation is committed. A clean worktree is not proof the shared test environment is unclaimed.
