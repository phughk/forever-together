# Resolutions

The author's answers to the open decisions in `reviews/r1-consolidated_review_report.md` (section A), 2026-11-21. Each answer was a single-choice question; the first option was the reviewer-recommended default.

| # | Question | Answer |
|---|----------|--------|
| A1 | Keep `BUILTIN[0]` as the export key despite `age`'s `ssh` feature pulling `rsa`? | **Keep `BUILTIN[0]`; record the `rsa` cost and check `cargo audit` first.** The separate intake key was rejected again. |
| A2 | Which optional pieces to keep (`reset`, `--public`, alias)? | **No alias.** Keep import, export and reset. `--public` dropped (the author's answer named only the three kept operations; `key show` prints the public line). |
| A3 | Should `key show` and `key export` create a key on an empty store? | **Create a key, as today.** A later `key import` onto an auto-created key needs `--force`. |
| A4 | Fallback if `varaq` does not lock the store? | **Verify at step 0; if no lock, a lock file taken by the app and the key commands.** |
| A5 | Guard for a wrong `BUILTIN[0]`? | **Assume `BUILTIN` entries are valid and panic if one is not.** (The orchestrator also adds a cheap CI test that every `BUILTIN` entry is valid, to match; say if you do not want it.) |
| A6 | Windows GUI subsystem has no console for the key commands. | The author first asked for the functionality in Settings, then, shown that it puts key operations on the web API, chose to **drop it for now and create a ticket** (0046). Windows key commands stay unsupported until then. |
| A7 | Keep the new-code-without-`dyn` exception? | **Keep it**; existing `dyn Store` code is converted later (ticket 0045). |

Applied to `design_draft.md` as decisions 7 to 12 and the matching edits. Review action items B1 to B17 were applied by rewriting `design_draft.md` on 2026-11-21 (Risks section, complete CLI spec, `keys.rs` contract, corrupt-key behaviour, enumerated tests, corrected file rows, dependency decision, comprehension and structure fixes). Part 2 is described as an interface only because it cannot be built yet.
