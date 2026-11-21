> **Archived 2026-11-21.** Its decisions are contained in ADRs, mainly 0034 and 0035. History only; the ADRs win.

# Reconciliation: concepts_brief.md against design_draft.md

Compared on 2026-11-21. The brief is the predecessor context (Step 1). The draft is the document being finalized. Contradictions come first, then additions.

## Contradictions (the brief decided a question and the draft decides it differently)

| # | Brief says | Draft says | Where the draft's answer came from |
|---|-----------|-----------|-----------------------------------|
| 1 | "Group decision rule (the review's recommendation). ... Each peer holds at most one lease ... Per-peer leases and a total order" | Peers agree by converging announcements and a human Confirm (Technical Plan 6, `adr/0010`) | The author chose option 3 at the agreement gate. The brief was not updated. |
| 2 | "Rust crates: `p2prdf-code`, `p2prdf-core`, `p2prdf-net`, `p2prdf-lan`, and a thin `src-tauri` shell" | One Rust crate with modules `code`, `engine`, `net`, `store`, `server` (`adr/0013`) | The author called the crate split excessive. |
| 3 | "One frontend with two transports (Tauri calls and HTTP)" | One web app and one transport: the desktop window loads the local server over HTTP, with no Tauri IPC (`adr/0012`) | The author said the desktop and the phone use the same web app and API. |
| 4 | "The addon sends raw instance IDs" and the code keeps "instance ID" | The code carries the zone name only. No instance ID and no dungeon field (`adr/0018`, `adr/0006`) | The author said users never search from inside a dungeon. |
| 5 | "name" is one field, and names are unique | `first name` and `surname` are two separate fields, and the realm is never read from the name call (`adr/0022`) | The author confirmed two-part names. |
| 6 | Overview: the code holds "professions, a gear summary such as item level" | The code has no professions. Item level only. Professions are an Open Question | Not decided by the author. The brief's own item 4 says professions come later, so the Overview line is stale. |
| 7 | "shared test vectors in both repos, a CI check that they match" | One repository, vectors once under `docs/`, no copy check (`adr/0020`) | The author said the addon, app and web app go into one new repository. |
| 8 | Goals: "A seeded multi-peer simulator and property tests for the group rule" | Functional tests only, with a handful to a few dozen simulated peers (`adr/0014`) | The author called a 10,000-peer test absurd. |
| 9 | Overview: the app "works out the dungeon from the location ... under the key `dungeon + region + level`", and Goals: "A dungeon picker defaulted from location" | A dungeon table with a level and zone-name recommendation, the player picks, topics are `region + realm + faction + dungeon` (`adr/0005`, `adr/0006`) | The author's decisions about the dungeon table and level-free topics. The brief's later "Revised" bullets agree with the draft, but the Overview and Goals text was not changed. |
| 10 | "Bootstrap order ... (2) the DHT noticeboard ... (3) the hard-coded seed", "Network options still open" | Remembered peers, then peer exchange, then trusted peers. The noticeboard is dropped (`adr/0009`) | The author replaced the noticeboard. The brief keeps the old bullet next to its "Revised" bullet. |

## Additions (the draft decides something the brief did not raise, and it changes the architecture, an interface or a dependency)

*   **Signed gossip messages** with ed25519 and `postcard` serialization (`adr/0019`, Proposed). New protocol rule and dependency.
*   **Loopback-only API routes** (pairing, token reset, LAN mode, trusted peers) and live updates read with `fetch` streaming (`adr/0012`, Proposed details).
*   **Test VM as a git submodule** (`vendor/wow-forever-addons`), with VM additions committed in `wow-forever-addons` (`adr/0021`, Accepted).
*   **`qrcode` library** in the web app, for the pairing QR. New dependency.
*   **Fixed defaults** (epoch 10 s, ad keep-alive 45 s, confirm timeout 60 s, six widening stages, 100 known peers). Starting values kept in settings, tuned from use.
