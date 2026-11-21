# Stage 5: Zoom Out and Inventive Catalyst (round 1)

Target: `../design_draft.md`, full file (lines 1-282). Run 2026-11-21. Reviewers: `egm-zoom-out` (Mode REVIEW, `zoom-out-standard`) and `egm-inventive-catalyst` (Mode REVIEW). Reports were returned in chat by the reviewers and written here by the orchestrator. Finding ids: `Z<n>` for the zoom-out reviewer, `C<n>` for the inventive catalyst.

## Zoom Out (`egm-zoom-out`)

Reviewer note: it read the persona, the standard, the draft, the context files it opened and `app/Cargo.toml`. It did not open ADRs 0008, 0016, 0023 or 0036, tickets 0008 and 0044, or `app/src/net/iroh_net.rs`. Blockers: none.

**Z1 (Major), lines 51, 98, 254-278: the running-app guard rests on an unverified fact.**
Decision 6 relies on `varaq` refusing a second opener, and the build order does not gate on checking it. `VaraqStore::open` takes only an in-process `Mutex<Db>` and plain `OpenOptions` handles; no cross-process lock is visible. If `varaq` has none, the cost is larger than the "stale identity in memory" at line 273: two processes would write the same log and data files, which risks corrupting the whole store (known peers, settings). State that risk. Add a step 0 before step 1: verify the lock and choose the fallback before any code lands. The interim one-line notice at line 98 is a UI patch over an unknown.

**Z2 (Major), lines 98, 219: "read-only, not guarded" contradicts itself.**
`key show` and `key export` open the store through the same `VaraqStore::open`; if `varaq` locks, both fail while the app runs. The `export` pseudocode creates the secret if missing, which is a write. State the intended behaviour with the app running, and decide whether `show` and `export` should fail rather than create a key.

**Z3 (Major), lines 193, 237-246: Part 2 is detailed design for work that cannot be built.**
It depends on the scan log (tickets 0039, 0040, not built) and on an empty `BUILTIN` (ticket 0008), yet specifies files, crate features and tests. ADR 0038 already holds the decision; a one-paragraph interface (recipient as 32 bytes, armored output) is enough. The open questions at lines 52-53 (`ssh-key` versus `age` parser; `age` beta, `ssh` feature pulls `rsa`) undercut it. Line 241 commits to `age` with `ssh` anyway; `rsa` has a published timing side-channel advisory (RUSTSEC-2023-0071), so check `cargo audit` or `cargo deny` before committing. Two crates parsing OpenSSH ed25519 keys is a duplication risk: resolve or spike before naming features.

**Z4 (Major), lines 37, 102, 109, 210: the central compatibility claims are asserted, not verified.**
The design claims `age -d -i <OpenSSH key file>` decrypts an `age` ssh-ed25519 recipient built from the same 32 bytes. `docs/spike-s0.md` rows list only `ssh-keygen -y -f`, not `age -d -i`. The line 37 goal depends on it. Add the `age -d -i` round trip as a spike row and a gate. The in-app test (line 244) uses the `age` library, not the CLI, so it does not prove the CLI workflow.

**Z5 (Major), lines 94, 96: `key reset` and `--public` are speculative scope.**
`reset` equals `ssh-keygen -t ed25519` plus `key import --force`, and `import` already closes the "leaked key can only be replaced by hand" gap. `--public` and `.pub` is an open question (line 54) and `key show` prints the same line. Drop both until a need appears; that also removes the reset confirmation, `--yes`, the warning logic and a test matrix.

**Z6 (Major), lines 201, 250: the "no `dyn`" rule adds churn against codebase convention.**
The code uses `&dyn Store` throughout (31 places, including `get_json` and `put_json`). With `?Sized`, `load_or_create` still accepts `&dyn Store`, so static dispatch is not delivered for the callers that matter. The design bypasses `runtime::open_store` to get a concrete `VaraqStore` (line 203), which already does `create_dir_all`. This breaks pattern consistency and mixes two styles on the critical path for a rule with no stated benefit here. Either use `&dyn Store` like the neighbours and let ticket 0045 convert everything, or justify the split.

**Z7 (Major), lines 201, 215-235: identity helpers and pseudocode ignore existing code.**
`Identity::generate()` and `secret_bytes()` already exist; "regenerate" is `put(KEY_IDENTITY_SECRET, Identity::generate().secret_bytes())`; "replace" is one `store.put`. `Store::put` is already a single write, so "atomic replace semantics" (line 201) describes existing behaviour. Decide whether `keys.rs` or `identity.rs` owns the store-key operations; both are named.

**Z8 (Major), lines 197-235: altitude drops into the weeds.**
The pseudocode restates the Commands section (91-98). Rows for `docs/spike-s0.md`, `CLAUDE.md`, `tickets/README.md`, `docs/egm/README.md` and ticket edits are bookkeeping; the table also restates test names and clap test cases. Keep modules and interfaces (`keys`, `identity`, `cli`, `lib`, `runtime`); move bookkeeping to the ticket.

**Z9 (Major), lines 60-61, 202-203: CLI structure gaps.**
`--data-dir` and `--headless` are top-level; `forever-together key import f --data-dir X` fails in clap unless `data_dir` is `global = true`. `--headless` plus `key ...` is a valid parse the design does not address. The subcommand design has the same invalid-combination issue it cites against flags (line 165). Say whether `--print-public-key` wins over a `key` subcommand. The alias keeps two paths (`Mode::PrintPublicKey` and `Mode::Key(Show)`) to one behaviour; for a pre-release app, dropping the alias is simpler, and the design lists the doc updates anyway.

**Z10 (Nit), line 205: the headless test is heavier than needed.**
`run_headless` loops until Ctrl-C and starts real iroh unless `WOWFT_NET=memory`. Test through `public_key` after import; the fresh-directory case already exists as `public_key_is_created_once_and_stable`.

**Z11 (Nit), lines 268-283: misplaced sections.**
Three rejected alternatives (lock file, warning only, no stdout) sit under Detailed Implementation after Verification. They belong in Alternatives Considered; two restate decisions 5 and 6.

**Z12 (Nit), lines 6, 125, 193: "0039" is ambiguous** (ADR 0039 versus ticket 0039). Write "ADR 0039" or "ticket 0039".

**Z13 (Nit), line 63: `--stdout` bypasses owner-only creation.** `key export --stdout > f` creates `f` under the shell's umask. Note it in the README requirement.

Summary: no blockers. Gate before Part 1: verify the `varaq` lock (Z1) and the `ssh-keygen -y -f` and `age -d -i` round trips (Z4). Cut: `reset`, `--public`, and the alias if the author agrees; reduce Part 2 to an interface note; remove the weeds.

## Inventive Catalyst (`egm-inventive-catalyst`)

Reviewer note: it read the persona, the draft and every listed context file. Findings are advisory. It found no false compromise in Decision 1 (plain key, no passphrase), Decision 3 (`key` subcommands), the choice of `age` over a custom envelope, or `--stdout`.

**C1 (Major). Scope: the rejected intake-key alternative (lines 133-138), Decision 2 (60), scan-log encryption (104-109), "Losing the key" (115), `key reset` (96), the `age` open question (53).**
False compromise: the draft accepts that the decrypting secret is the live trusted node's identity key, with costs at lines 138 and 115: a cloud-server compromise exposes every export and lets the attacker impersonate the trusted peer; `key reset`, the remedy for a leaked identity, makes every old export unreadable; losing the node key orphans the exports. The author declined a separate intake key and two-key sealing, but the costs come from a narrower fact: the decrypt secret must be the same secret that runs on an always-on internet host. The same coupling causes the `rsa` dependency problem (line 53): the Ed25519-to-X25519 conversion exists only so the node key can double as an encryption key.
Proposed resolution (segmentation of the two roles): keep the rejection of "two keys in the file" and "a hosted key service". Make the intake recipient a native `age` X25519 public key, a 32-byte constant beside `BUILTIN`, whose `AGE-SECRET-KEY-1…` secret is generated once, kept offline and never loaded on any node. Then `age` needs only the core and `armor` features (no `ssh`, no `rsa`, no conversion); `key reset` and `key import --force` stop affecting export readability; a compromised trusted node no longer exposes player data; and the author decrypts with `age -d -i intake.txt`, with `key export` off the decrypt path.
Secondary liabilities: one more constant to ship and one more offline secret to back up and rotate; losing the intake secret loses every export; `BUILTIN[0]` ownership no longer gives the decrypt key for free; ADRs 0038 and 0039 need amending again. The author has weighed and rejected a separate key once; put this to them as "the same decision, with the `rsa` dependency and the reset coupling now priced in", not as a re-argument.

**C2 (Major). Scope: the first open question (line 51), "Running app (decision 6)" (98), the rejected lock file (268-278); `app/src/runtime.rs:193` and `app/src/identity.rs:85`.**
False compromise: "import while the app runs" is either guarded by an unverified third-party lock or silently wrong (line 273: the app keeps the old identity in memory). A related passive failure is the headless deploy path: a trusted node started on an empty or ephemeral volume generates a random key and runs, and nothing compares its peer ID with `trusted::BUILTIN` (accepted as a cost at line 172). Both are handled as operator problems, not detected.
Proposed resolution (two watchdogs, no lock file, no new start-up input): (a) a running app (window or headless) rereads `KEY_IDENTITY_SECRET` at a low rate or on store-touching events; on a mismatch with its in-memory identity it logs a prominent warning, or in headless mode exits so a supervisor restarts it with the new key. (b) Headless mode logs its peer ID at start (it already does, `runtime.rs:209`) and warns when `BUILTIN` is non-empty and its peer ID is not in `BUILTIN`; for a hard guarantee, an opt-in `--headless --expect-peer-id <hex>` refuses a generated or mismatched key.
Secondary liabilities: (a) adds polling or event-driven reads and depends on the same unverified `varaq` fact; reading through the already-open handle sees only its own writes, so it may detect nothing if `varaq` allows two openers (needs a spike row); a headless self-exit can become a restart loop under a supervisor if the stored key is bad. (b) is advisory unless `--expect-peer-id` is used, which adds one deploy-time value to keep in sync after any reset.

**C3 (Nit). Scope: `key reset` (line 96, pseudocode 231-235) and `key import --force` (95, 225-229).**
False compromise: both discard the old secret in one write with no way back, while the draft says a lost trusted key costs a release and unreadable exports (115, 138). A mistyped `--force` or `--yes` on the trusted node is unrecoverable; the prompt is the only protection, and `--yes` skips it.
Proposed resolution (beforehand cushioning): when `reset` or `import --force` would replace an existing key, require `--save-old <file>` (written owner-only like `export`) or an explicit `--discard-old` for the leaked-key case.
Secondary liabilities: one more flag and secret file; a leaked key has an extra copy on disk unless discarded, so `--discard-old` must be explicit. If C1 is adopted, the main reason to keep the old key (reading old exports) disappears and this becomes a convenience.

**C4 (Nit). Scope: Non-Goals rotation bullet (line 43), "BUILTIN and the trusted key" (111-115), `app/src/net/trusted.rs:17`.**
False compromise: "changing `BUILTIN` is a release" is stated as a law. Every shipped build keeps the dead entry forever, and a new-key release does not help installs that never update. Old builds fall back to remembered peers and peer exchange (ADR 0023), which the draft does not say softens the failure.
Proposed resolution (preliminary action): ship `BUILTIN` with a second, offline-held standby trusted key, generated and backed up now but not running, so losing the live key costs no release to restore bootstrap. The export recipient stays `BUILTIN[0]`, or the separate intake key from C1.
Secondary liabilities: a second secret to protect for the life of every shipped build; the dead entry is dialled and times out until the standby is activated (a little cold-start latency); whoever gets the standby secret is trusted by every shipped build; ticket 0008 and the README backup process grow.

**C5 (Nit). Scope: `export` pseudocode (line 219, "create if missing") and `import` (226-227); `runtime::public_key` (`app/src/runtime.rs:127-130`).**
False compromise: a read-only command (`key show`, `key export`) creates and stores a key if none exists, because the existing path uses `load_or_create`. An operator who runs `key show` on a fresh cloud volume creates a random identity, and a later `key import` then needs `--force`. The deploy story (line 172) depends on import happening before any other command touches the store, and the draft does not name that ordering.
Proposed resolution: separate the read path from the create path. `key show` and `key export` report "no key yet" without creating one; only a normal start creates. Import onto an empty store then needs no `--force`.
Secondary liabilities: `--print-public-key` as an alias loses create-on-first-use, so ticket 0008 and CLAUDE.md (which say to run it to get the first trusted key) need `--headless` run once or an explicit `key reset --yes` on an empty store; existing `public_key` tests change.
