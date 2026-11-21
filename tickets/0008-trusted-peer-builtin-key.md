# 0008. Supply the built-in trusted peer key

* **Status:** open
* **Kind:** prerequisite
* **Source:** CLAUDE.md; ADR 0023

## What

`net::trusted::BUILTIN` is empty. Run an always-on copy with `--headless`, print its key with `--print-public-key` and add it. The private key is never committed.

## Done when

BUILTIN holds the key; a fresh instance with no remembered peers reaches the trusted peer as a last resort.
