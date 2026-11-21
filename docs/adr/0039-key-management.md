# 0039. The app's keys can be exported, imported and reset from the command line

* **Status:** Accepted for the requirements (decided by the author 2026-11-21); details open in ticket 0043 and not built. Extends [0008](0008-iroh-and-local-storage.md) (permanent keys in the `varaq` store), [0016](0016-peers-identified-by-key.md) and [0023](0023-no-supernodes.md). Used by [0038](0038-scan-log-and-opt-in-export.md).
* **Date:** 2026-11-21
* **Deciders:** Hugh Kaznowski

## Context

The app generates an iroh secret key on first start and stores it ([0008](0008-iroh-and-local-storage.md)). Today the only key tool is `--print-public-key`. The author will run trusted nodes on cloud instances, needs backups of the trusted key, will hold the private key that decrypts exported scan logs ([0038](0038-scan-log-and-opt-in-export.md)), and needs to reset a key that leaked or was lost.

## Decision

The command line gets three operations, besides the existing `--print-public-key`:

* **Export** the secret key to a file, for backup and deployment.
* **Import** a secret key from a file, so a node starts with a chosen, stable identity and prints the same public key every time.
* **Reset** the key: discard the stored key and generate a new one.

Rules, *defaults* where the author did not specify:

* The secret key never goes to the web app, the API or a log, and never to standard output unless asked for with an explicit flag. These are command-line operations only.
* An exported file is written with owner-only permissions and refuses to overwrite an existing file. Import refuses to replace an existing key unless told to, and says what will change.
* Reset asks for confirmation, and a node refuses to reset or import while the app is running on that data directory.
* Any key change changes the peer ID ([0016](0016-peers-identified-by-key.md)): the app's remembered peers stay, but other nodes see a new peer. Resetting a trusted peer's key means a new release with the new `BUILTIN` key ([0034](0034-system-overview-goals-and-risks.md) risk table). The command says so.
* The same key file format is used by export and import. The format, whether it is protected by a passphrase, and the exact command names are open (ticket 0043).
* The private key that decrypts scan exports is an ordinary exported key held offline by the author; the app only ever uses the public half.

## Consequences

* A cloud node can be rebuilt with the same identity from a backup.
* Losing the trusted key without a backup means new key, new release, and unreadable old scan files.
* Permission checks for the stored key (ticket 0003) matter more now that keys can be exported.
