# 0048. Build the DHT lookup, published addresses and the headless relay

* **Status:** open
* **Kind:** feature
* **Priority:** P1
* **Blocked by:** tickets 49 (the rate and the unknowns)
* **Source:** ADR 0041
* **ADRs:** [ADR 0041](../docs/adr/0041-address-lookup-and-hole-punching-relay.md)
* **EGM:** [0038-self-host-iroh-relay](../docs/egm/0038-self-host-iroh-relay/design_draft.md)

## What

The "Detailed Implementation" list in the 0038 draft: the `IrohConfig::Default` branch rebuilt from `presets::Minimal` with unfiltered DNS and DHT lookup and the project's relay list, `app/src/net/relay_server.rs`, `--relay` (headless only), tests, README and CLAUDE.md, and the ADR 0034 risk rows.

## Done when

A node started with `--headless --relay` serves, a window install cannot, both lookups run on every endpoint, and the loopback tests pass.
