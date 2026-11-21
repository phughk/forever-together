# 0005. Verify leader invitations on a live network

* **Status:** open
* **Kind:** verification
* **Source:** ADR 0025; docs/spike-s0.md

## What

Leader invitations pass only in simulation (78.4% mean within 30 s). Run real instances and check the 70% floor, the tank-scarcity cap, and the engine's lack of retransmission for lost direct messages.

## Done when

Live numbers recorded next to the simulation numbers; thresholds marked verified or revised.
