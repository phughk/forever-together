# Consolidated review report (round 1)

Target: `../design_draft.md` (282 lines, status Proposed). Mode: consolidated full flow (no stops, no remediation applied). Run 2026-11-21. FeatureRange 1-190, DetailedRange 191-282. Reviewers: zoom-out, inventive catalyst, comprehension, design critic, structural, style, implementation readiness, codebase details. Stage reports in this folder:

| Stage | Report | Findings |
|---|---|---|
| 5 | `r1-zoom_out_report.md` (zoom-out Z1-Z13, inventive catalyst C1-C5) | 8 Major, 5 Nit (Z); 2 Major, 3 Nit (C) |
| 6 | `r1-comprehension_report.md` (P1-P16) | 10 Major, 6 Nit |
| 7 | `r1-critic_report.md` (design D1-D19, structural S1-S12, style T1-T17) | 9 Major, 10 Nit (D); 1 Blocker, 4 Major, 7 Nit (S); 7 Major, 10 Nit (T) |
| 8 | `r1-intermediate_zoom_out_report.md` (N1-N4 plus rulings) | 2 Major, 2 Nit |
| 9 | `r1-implementation_readiness_report.md` (R, K, Y) | 5 Blocker, 8 Major (R); 2 Blocker, 12 Major (K); 11 Major (Y); plus nits |

Verdict: **the design is not ready to build.** Part 1 has one structural gap (no Risks section), one unverified premise (the `varaq` lock), and a file plan that contradicts itself and the code in several places. None of the issues questions the decisions you made (plain key, `key` subcommands, `age` to `BUILTIN[0]`, headless same CLI, `--stdout`, store lock). They are gaps and stale text.

## A. Decisions that need the author

1. **Separate intake key (C1).** The reviewers raised again the option you rejected: encrypt exports to a dedicated `age` X25519 key kept offline instead of `BUILTIN[0]`. New reasons since you decided: the `ssh` feature of `age` pulls in `rsa` (RUSTSEC-2023-0071, reported by two reviewers, unverified by me); it removes the Ed25519-to-X25519 conversion; a compromised trusted node would no longer expose exports; `key reset` would stop affecting old exports. The zoom-out reviewer calls it net simplifying; it is yours to keep or reverse. If you keep your decision, record the `rsa` pull-in as an accepted cost.
2. **Scope cuts (Z5, Z9).** `key reset` (reproducible as `ssh-keygen` plus `key import --force`), `--public`/`.pub`, and the `--print-public-key` alias are the three items the zoom-out reviewer would drop. Cutting `reset` also removes the need for C3.
3. **Read paths must not create keys (C5, N2, D15).** `key show` and `key export` on an empty store currently generate a key, which makes your deploy path ("import before first start") fail without `--force` after any earlier `key show`. Proposed: they report "no key yet". Cost: `--print-public-key` loses create-on-first-use, which `README.md`, `CLAUDE.md:89` and ticket 0008 rely on.
4. **If `varaq` does not lock (Z1, N1).** Which fallback you prefer (lock file, store-level check, warning only) should be decided now, because the reviewers think "no lock" is the likely case (`VaraqStore` holds only an in-process mutex). Decision 6 is unverified until the check runs.
5. **`BUILTIN[0]` safety (D4) and standby key (C4).** Whether to add a release gate that decrypts with the real backed-up key before shipping. The standby-key idea (C4) and the watchdogs (C2) were ruled out by the zoom-out reviewer; no decision needed unless you want them.
6. **Windows (K14).** Release builds use the GUI subsystem (`main.rs:2`), so the key commands print nothing from a terminal. Restrict them to non-Windows, or scope the fix into the Windows ticket.
7. **`dyn Store` exception (Z6 downgraded).** One question: is the new-code-without-`dyn` exception worth a second store-opening path?

## B. Action items by priority

### Blocker

1. **Add a Risks & Mitigations section** (S1, D1). Gather the scattered "cost accepted" items into failure mode, mitigation or explicit acceptance: key loss or leak, partial writes, second process on the store (Z1), compromised trusted node (D3, C1), beta `age` and `rsa` (D9), headless silent new key (D7, C2b), destructive commands without a safety net (D6, C3), GitHub limits (D8), `BUILTIN[0]` correctness (D4), forward secrecy (D3).
2. **Make the `varaq` check a gate at step 0** (Z1, N1, D5, P8, R-B2, K7). One owner, one place in the build order; say what the commands do if it fails; state the real corruption risk if two openers are allowed; fix R-B2 (lock errors are untyped, `varaq_store.rs` and `store/mod.rs` missing from the table); state that read commands are also affected and that the backup process (line 114) may need the node stopped.
3. **Resolve `key show` versus `runtime::public_key`** (T1, R-B1, Y-M1, K3). One implementation; what the alias prints; what the existing test asserts; who creates the data directory (Y-M6).
4. **Specify the CLI completely** (P4, D14, S8, R-B3, K4, Y-N7, Y-N8). `export --stdout` and `--public` in the row; file argument and `--stdout` exclusive; `KeyCommand` definition; `global = true` for `--data-dir`; precedence of `--print-public-key` and `--headless` against a subcommand; fate of `Mode::PrintPublicKey`; clap derives; test cases.
5. **Define the `keys.rs` contract** (Z7, R-B5, R-M1, Y-M2, Y-M4, Y-M5, K5, K6, N3). Return data and print in `lib.rs`, or take a writer; where the prompt lives and how stdin is read; the `KeyError` variants (malformed file, I/O, locked, declined, bad stored key); behaviour on a missing or corrupt old key (best-effort old ID, "unreadable"); the identity helper set (read without create, put, generate, derive ID).
6. **Fix the unworkable tests** (Z10, R-B4, K8, Y-M3, R-M4, Y-M11, D10, T4). Test through `key import` then `public_key`/`Identity::load_or_create`, not `run_headless`; enumerate the `keys::` tests (mode 0600, refusal to overwrite, wrong type, encrypted, malformed, trailing data, CRLF, `--force`, unchanged store after failed import, stdout/stderr split); add a fixed known vector; add a manual `age -d -i` CLI check; correct "proven" wording (T4).
7. **Correct stale and false rows** (K1, K2, Y-M7 to Y-M10, K12, K13, T2). ADR 0038 and ADR 0039 rows describe edits already made (reduce to the real delta: ADR 0038 line 42, ADR 0039 Status and crate names); `runtime::Runtime.store` does not exist (`NetContext.store`); ticket 0045 exists (also fix ticket 0045 line 11); `spike-s0.md` row 12 already exists; the README has no run-a-trusted-peer section (add, not update); line 119's claim about store permissions is untrue for the directory.
8. **Pin the dependencies** (Z3, D9, T3, R-M2, K9). Crate, version and features for the OpenSSH key crate and `age`; check duplicate `ed25519-dalek`/`rand_core` versions; `Cargo.lock`; run `cargo audit`/`cargo deny` for `rsa`; close the `ssh-key`-versus-`age`-parser question or mark dependent sections provisional with an owner.

### Major

9. **Part 2** (Z3, R-M6, K10, Y-N1): either cut to a one-paragraph interface note, or give the API: the `ssh-ed25519 <base64>` construction from 32 bytes, signature, error enum (including invalid curve point), filename and date, review-before-export step, real paths (they do not exist yet).
10. **Comprehension fixes** (P1, P2, P3, P10): two sentences saying what the app and the scan log are; define `varaq`, iroh, trusted peer, `BUILTIN`, `trusted_peers`, headless, always-on node; prefix every number with "ADR" or "ticket"; one voice (no "I"); drop or define `egm-review` and "reconciled".
11. **Contradictions to remove** (P4 to P6, T6, S6, T7, T12, D19): `--stdout` described as addition and as alternative; dispatch owner (`runtime` against `lib.rs`); "five tickets collapse into one" against the ticket list (drop the goal); "no warning-only mode" against the interim notice; "nothing else can be done with it" against `--print-public-key`; `.pub` default against `--public` flag.
12. **Rejection attribution** (D2, S2): link ADR 0038 and ADR 0039 as the record and state the technical trade-off as the reason; remove the circular line 144 reasoning; add PKCS#8 and "copy the data directory" as alternatives (D18); move the three misplaced alternatives into Alternatives Considered (S5, P9, R-N2).
13. **File-creation pinning** (R-M3, K17, D12): one `create_new(true)` open with mode 0600; partial file on failure; relative and missing-parent behaviour; `--stdout > f` creates 0644 and defeats `ssh-keygen` (K16, D13).
14. **Structure** (S3, S4, S5): rename "Technical Plan" or split out "Proposed Architecture"; move Decisions before Open Questions; give Alternatives one template.
15. **Build order and file table** (R-M8, K11, R-M7): `mod keys;`, the dependency and the `identity.rs` helper in step 1; `reset` in step 2 or move; the `varaq` spike as step 0; mark each row Create or Modify; name the new tickets and files.
16. **Windows** (K14, D16): GUI-subsystem consoles and owner-only on Windows.
17. **GitHub issue limits** (D8): 65,536-character body limit, attachment extension allowlist, armor overhead; ADR 0038 leaves the size cap undecided.

### Nit (group)

Metadata staleness (line 4, S12, T8); wording ("clear", filler, "proven"); `BUILTIN 0` label (T16); links and unlinked references (S9, S10, S11); line 38 ticket goal; diagram gaps (S11); `age` recipient from raw bytes unverified (Y-N1); verification command list (R-N3, K15); doc edit locations (K18, K19, R-N4, R-N5); key retention rule for old `BUILTIN[0]` secrets (D11); Windows ACL notes (D16, D17); `--stdout` on a TTY (D13); filler lines (Y-N2 to N5).

## C. Rejected or deferred by the reviewers

* C2 (watchdog polling and `--expect-peer-id`): rejected by the zoom-out reviewer as YAGNI and built on the unverified `varaq` fact; keep only the README line comparing the logged peer ID with `BUILTIN`.
* C3 (`--save-old`/`--discard-old`): rejected unless `reset` stays; replace with a README line "export before you force-import or reset".
* C4 (standby trusted key): rejected for this design; if wanted, belongs in ticket 0008. Keep a sentence that old builds fall back to remembered peers and peer exchange (ADR 0023).
* Z6 downgraded to Nit; N4 shows its supporting claim (`create_dir_all`) does not hold.

## D. Reviewer limits

Reviewers could not write files; I saved their reports. They read only the files in the shared context; the zoom-out and style reviewers each skipped some of them (listed in their reports). Crate claims (`age` recipient from raw bytes, the `rsa` advisory, GitHub's attachment allowlist) came from reviewers' knowledge and are unverified. The critic report did not exist when Stage 8 ran, so Stage 8 used the Stage 5 findings and the comprehension report.

## E. Traceability matrix

| Action item | Raw findings |
|---|---|
| 1 Risks section | S1, D1, D3 to D8, C1 to C4 |
| 2 `varaq` gate | Z1, N1, D5, P8, R-B2, K7 |
| 3 `key show` | T1, R-B1, Y-M1, K3, Y-M6 |
| 4 CLI spec | P4, D14, S8, R-B3, K4, Y-N7, Y-N8 |
| 5 `keys.rs` contract | Z7, R-B5, R-M1, Y-M2, Y-M4, Y-M5, K5, K6, N3, C5, N2, Z2, D15 |
| 6 Tests | Z10, R-B4, K8, Y-M3, R-M4, Y-M11, D10, T4, K16 |
| 7 Stale rows | K1, K2, Y-M7 to Y-M10, K12, K13, T2, T5, R-N5, K19 |
| 8 Dependencies | Z3, D9, T3, R-M2, K9 |
| 9 Part 2 | Z3, R-M6, K10, Y-N1, D4, M8 equivalents |
| 10 Comprehension | P1, P2, P3, P10, T15 |
| 11 Contradictions | P4 to P6, T6, S6, T7, T12, D19 |
| 12 Alternatives | D2, S2, S5, P9, R-N2, D18, N6 |
| 13 File creation | R-M3, K17, D12, D13 |
| 14 Structure | S3, S4, S5 |
| 15 Build order | R-M8, K11, R-M7 |
| 16 Windows | K14, D16, D17 |
| 17 GitHub limits | D8 |
