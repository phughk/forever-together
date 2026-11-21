# Stage 6: Comprehension test (round 1)

Target: `../design_draft.md`, FeatureRange lines 1-190 and DetailedRange lines 191-282. Run 2026-11-21. Reviewer: `egm-comprehension` (Mode REVIEW). It read the document alone, with no context files. The reviewer could not write files; the orchestrator saved its result. Finding ids: `P<n>`.

## What this proposes

The design adds a `key` command group to the app's command line: `key show`, `key export <file>`, `key import <file>` and `key reset`. These let an operator back up, restore, deploy and replace the install's single ed25519 identity key. The key stays as 32 bytes in the local store, and export/import use an unencrypted OpenSSH-format private key file that tools like `ssh-keygen` and `age` can read. The design also encrypts the exported scan log with `age` to `BUILTIN[0]`, the first entry in a compiled-in trusted-peer list. The author decrypts it outside the app with `age -d -i <exported key file>`. Work is split into Part 1 (the commands and key file) and Part 2 (the scan-export encryption, which waits on tickets 0039, 0040 and 0008).

## How the existing system works, per this document

On first start the app generates one permanent ed25519 secret key. It stores that key in a local store under `KEY_IDENTITY_SECRET`, using code in `app/src/identity.rs`. The public half is the peer ID that other peers see, and the same bytes are the iroh endpoint key. A built-in trusted-peer list, `net::trusted::BUILTIN`, is compiled into the binary and is currently empty. The only key-related CLI today is `--print-public-key`, plus `--headless` and `--data-dir`. There is no backup, import, reset, or way to start a node with a chosen key. The document does not say what the app does, what a "trusted peer" is trusted to do, or what the "scan log" contains.

## Findings

**P1 (Major), lines 1-25, 37, 193: the document never says what the app is or what a scan log is.** The reader is never told what forever-together does. "Scan log", "scan export", "characters or codes", "players" and "the card" (line 108) are used without definition. Goal 37 and all of Part 2 depend on them. The privacy rationale (line 137, "players expose no characters or codes") cannot be judged without knowing what is in the log. Add a two-sentence description of the app and the scan log.

**P2 (Major), lines 13, 21, 24, 42, 47, 51: project-specific terms with no inline definition.** `varaq` first appears at line 42 and is never explained (evidently the store backend; Decision 6 depends on its locking). "iroh" (line 13) is undefined, and it is unclear whether "peer ID" is the hex of the public key. "Trusted peer / trusted node" and what trust grants are undefined, though the motivation (lines 21, 24) is that the trusted key must persist. `BUILTIN` is half-explained at line 21. `trusted_peers` (line 93) is never introduced; the list that "can actually be filled in" (line 15) is not described. "Headless" (line 59) and "always-on node" (line 113) are undefined. The `Store` trait, `MemStore` and `VaraqStore` (lines 200, 203) are not introduced. "wowft" appears in a filename (line 107) unexplained. The links to ADRs 0008, 0034, 0036, 0038 and 0039 carry content the document depends on without summarising it, including the ADR 0038 "amendment" and "custom header" (lines 107, 245) and ADR 0036 (lines 209, 211, 213).

**P3 (Major), lines 7, 126, 193, 209, 242-243: number collision "0039" and "0040".** Line 7 and line 209 use 0039 as the key-management ADR. Lines 193 and 242-243 use "scan log (0039, 0040)" and "ticket 0039" as tickets. A reader cannot tell ADR numbers from ticket numbers without opening other files. Prefix them. Line 126 does not say which kind of number each is.

**P4 (Major), lines 63, 94, 202, 218-223: `--stdout` is described two ways.** Decision 5 says `--stdout` prints the key "in addition to writing a file". The Commands section says "instead of a file". The pseudocode follows "instead". The `cli.rs` row (line 202) lists `export <file>` with no `--stdout` and no `--public`, so the final CLI surface is unclear.

**P5 (Major), lines 97, 203-204: who dispatches key commands.** Line 97 says "`runtime` dispatches before any network or window start-up". Line 203 says `lib.rs` `run` dispatches and the commands avoid `runtime::open_store`. Line 204 says `runtime.rs` has only a comment change and keeps `public_key`. The owner of dispatch, and whether `public_key` works with the concrete store, are unclear.

**P6 (Major), lines 38, 123-127, 193, 242: the ticket map is inconsistent.** Goal 38 says the five tickets "collapse into one tracked piece of work". Lines 123-127 keep 0003, 0008 and 0044 separate and add new ones. Line 193 treats 0043 and 0044 as separate parts. Line 126 says the export module is in ticket 0040; line 242 says it is "inside the module from ticket 0039".

**P7 (Major), lines 24, 113: "it is empty" is ambiguous.** It could mean the list is empty or that the first entry is empty. "Chosen recipient" is unexplained. Line 24 describes a private key "kept offline", while Decision 2 uses the live trusted node's own identity key. The problem statement and the decision describe different things, and the reader is not told the framing was superseded.

**P8 (Major), lines 51, 98, 119, 127, 210, 275-283: the running-app guard is under-specified and tracked in several places.** Line 98 says the commands "also print a one-line notice to stop the app first" until the `varaq` behaviour is verified; that appears in neither the Part 1 table nor the pseudocode. The lock verification is assigned to ticket 0003 (line 51), a "new small ticket" (line 127) and `spike-s0.md` (line 210), while line 119 says 0003 only records file modes. The owner of the check is unclear. Lines 275-283 refer to "the store's own lock" without restating it.

**P9 (Major), lines 268-283: three alternatives sit under Detailed Implementation after "Verification".** They are alternatives to Decisions 5 and 6 and belong in "Alternatives Considered" (line 129), which lists eight others, so a reader would think that list was complete.

**P10 (Major), lines 3, 4, 131: unclear voice and undefined process terms.** Line 131 says "I considered" and "the trade-offs I put to them". The author is named at line 3 but the text says "the author" in the third person throughout. A reader cannot tell who "I" is. `egm-review` (line 4) and "reconciled" are undefined.

**P11 (Nit), line 93:** "Same output as today's `--print-public-key`" is unverifiable because the output is never stated.

**P12 (Nit), lines 200, 215-235:** line 200 has a garbled sentence ("a result or a typed error (an enum) (`KeyError`: ...)", "Pure of the CLI"). Line 215 says "three file operations" but `reset` is not a file operation and `show` is omitted. The pseudocode prints "old peer id" for import and reset without saying what happens when there is no old key. `export` creates a key "if missing" (line 219), which the Commands section does not say.

**P13 (Nit), lines 201, 252:** `S: Store + ?Sized` and "atomic replace semantics (parse first, write once)" are unexplained; whether `Store::put` is atomic is not stated. "31 uses" of `dyn Store` has no source.

**P14 (Nit), line 55:** Windows gets the directory's default access rules for an unencrypted secret; the document does not say what that means for exposure, although the stated goal is "owner-only".

**P15 (Nit), line 138:** "The same key now does signing" does not say signing what (line 46 rules out release signing); presumably iroh connection authentication.

**P16 (Nit), lines 51, 263:** Decision 6 is referenced at line 51 before the decisions list at line 57. "The project's 'only test what you changed'" is an unexplained project rule.
