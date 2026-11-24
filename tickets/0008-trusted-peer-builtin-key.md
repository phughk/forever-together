# 0008. Supply the built-in trusted peer key

* **Status:** in progress (BUILTIN filled 2026-11-23 with trusted1 and trusted2, backed up by the author; no node is running on them yet, so the cold-start fallback is not verified live)
* **Kind:** prerequisite
* **Priority:** P0
* **Source:** CLAUDE.md; ADR 0023
* **ADRs:** none
* **EGM:** [0043-key-management](../docs/egm/0043-key-management/README.md)

**Also needed by ticket 0044:** the first `BUILTIN` key is the recipient of encrypted scan exports. Back up its private key (ticket 0043) before shipping it.

## What

`net::trusted::BUILTIN` is empty. Run an always-on copy with `--headless`, print its key with `forever-together key show` (stop the node first) and add it. Back up the secret with `key export` first (README, "Keys"). The private key is never committed.

## Done when

BUILTIN holds the key; a fresh instance with no remembered peers reaches the trusted peer as a last resort.

## Check from outside, 2026-11-24

After the author reported trusted1 deployed: SSH on `89.40.6.71` answers; TCP 3340, 80, 443 and 7842 time out (filtered, not refused); n0's DNS has no record for either `BUILTIN` key; dialing both keys with the app's own endpoint setup fails with "No addressing information available" (`live_tests::trusted_peers_are_reachable`, ignored test). So from here trusted1 is not resolvable by key and the relay port is not reachable. Likely causes, none confirmed: the node is not running; it runs with a different key than `BUILTIN[0]` (check `key show` against `7c4b430a…`); or a firewall blocks TCP 3340 and the node's outbound UDP.
