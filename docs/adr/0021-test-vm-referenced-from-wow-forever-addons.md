# 0021. The headless test VM stays in wow-forever-addons and is referenced as a submodule

* **Status:** Accepted
* **Date:** 2026-11-21
* **Deciders:** Hugh Kaznowski

## Context

The standalone addon needs the headless WoW Lua test VM (`wowluavm`), which lives in `wow-forever-addons` with its fakes and the API export `wow-forever-api.json` (see 0020).

## Decision

Keep one copy of the VM in `wow-forever-addons` (now at `/Users/hughkaznowski/Projects/gitlloop/wow-forever-addons`; the project moved). The `forever-together` repository takes `wow-forever-addons` as a git submodule at `vendor/wow-forever-addons`, and its `test-addon` task builds and runs the VM from there, with `--api` pointing at the submodule's `wow-forever-api.json`. The VM additions the finder needs (a modified `GetCurrentRegion` that returns `setup.region or 3`, `GetAverageItemLevel`, and `vm.read_json`. The role comes from the fifth return of the existing `GetSpecializationInfo`, so no new role function is needed) are committed to `wow-forever-addons`.

## Consequences

One VM, no duplicate to maintain. The only change in `wow-forever-addons` is to the VM and its tests must still pass. Building the new repository needs the submodule to be reachable.

## Alternatives considered

* Copy the VM into `p2prdf`: not chosen by author, the copies would drift.
* Extract the VM into its own repository used by both projects: not chosen by author, a third repository and a change to `wow-forever-addons`.
