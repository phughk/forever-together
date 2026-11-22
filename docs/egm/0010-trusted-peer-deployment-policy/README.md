# 0010-trusted-peer-deployment-policy: Who may run trusted peers and how keys are added

* **Status:** open (initial draft written 2026-11-22, no review round yet)
* **Ticket:** [0010](../../../tickets/0010-trusted-peer-deployment-policy.md)
* **ADRs:** [0023](../../adr/0023-no-supernodes.md) (extended by the result), [0009](../../adr/0009-bootstrap-and-peer-exchange.md), [0038](../../adr/0038-scan-log-and-opt-in-export.md), [0039](../../adr/0039-key-management.md), [0034](../../adr/0034-system-overview-goals-and-risks.md)
* **Rounds:** none yet

## Question

Beyond the author's first trusted peer, who may run one, how their public keys reach installs (a release that changes `net::trusted::BUILTIN`, the player's own settings, or something else), in what order installs dial them, and what operators agree to.

## Files

* `design_draft.md`
