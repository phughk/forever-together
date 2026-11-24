# 0008. Supply the built-in trusted peer key

* **Status:** done for trusted1 (verified live 2026-11-24); trusted2 is not deployed yet, deploy it with the same unit and a different key
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

## Verified, 2026-11-24

trusted1 (`BUILTIN[0]`, `89.40.6.71`) is deployed and serving. From another network its key resolves through n0's DNS, the app connects in 3.1 s with a direct path, and two players with only the built-in list bootstrap through it and find each other (spike results, "trusted1 live"). trusted2 is not deployed yet. Problems found and fixed on the way: a service with no `HOME` used a relative data directory and a random key (the program now refuses to start without `HOME` or `--data-dir`), and `ufw` dropped the relay port and UDP.
