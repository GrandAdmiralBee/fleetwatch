# ADR-0001. Monorepo and cargo workspaces

- **Status:** accepted
- **Date:** 2026-10-08
- **Related documents:** [01-containers](../architecture/01-containers.md), [02-agent-identity](../architecture/02-agent-identity.md)

## Context

This project (in its initial imaginatory state) includes several binaries:

- Agent
- Control plane
- ingest
- metrics-writer
- alerting
- loadgen

This project also uses **1 shared protocol** is developed by **One** person, uses CI/CD and Nix for building.

## Decision Drivers (What is important for me)

- I need atomic changes in the protocol and both communication sides.
- Minimum effort for one developer
- Reproducible dev environment
- Ability to deploy services independently from each other
- Clear API and its boundaries for its clients

## Considered Options

| Option                          | Pros                                                                    | Cons                                                                |
| ------------------------------- | ----------------------------------------------------------------------- | ------------------------------------------------------------------- |
| Cargo workspace, monorepo       | shared `proto`, one commit for contract change, one CI, one build cache | every commit triggers checks. Can be manages with `crane` and cache |
| Repo per service                | Independent releases and access rights                                  | desycn `proto`, duplication of CI. Extra overhead for one developer |
| Core monorepo + repo per client | Proof that API is enough for clients                                    | More than one repo                                                  |

## Decision (proposition for now)

**Third option**: core application and agent in one workspace, API clients (Telegram, CLI, WebUI) can be placed separatly, because they need only OpenAPI specs.

## Consequences

- **Root key doesn't live on the server.** Its creation, updation and etc is managed via a script.

## Revisit When and If

Revisit this descision when agent's amount is too big to be left without key auditioning.
