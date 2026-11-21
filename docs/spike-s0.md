# Spike S0 results

Only the rows filled in so far are listed. The other S0 outputs (see the implementation plan, item 0) are still open.

## Seeded simulation (D1)

Source: `src-tauri/src/engine/sim.rs`. Run: `cargo test --release --lib -- --ignored --nocapture engine::sim::tests::report` from `src-tauri/`. Seed 1, one topic, same faction, stage 0. Model and assumptions are in the module comment.

Metric: of the peers that have a compatible five in their own sample at the start, the share committed within 3 rounds. A round is one 10 s epoch for converging announcements and one 8 s proposal cycle (4 s response window plus 4 s decision window) for the proposer model. Floor: 70% (plan).

| Scenario | Peers with a compatible five | Converging announcements, 3 epochs | Proposer model, 3 cycles | Proposer model, 4 cycles |
|----------|------------------------------|------------------------------------|--------------------------|--------------------------|
| 3000 peers, sample 100, delivery 0.95 | 2062 / 3000 | 1.9% | 12.1% | 15.8% |
| 3000 peers, sample 100, delivery 0.80 | 2156 / 3000 | 0.0% | 18.9% | 25.0% |
| 3000 peers, sample 100, delivery 1.00 | 2027 / 3000 | 56.3% | 8.6% | 11.0% |
| 3000 peers, sample 300, delivery 0.95 | 2212 / 3000 | 1.9% | 11.0% | 15.3% |
| 300 peers, sample 100, delivery 0.95 | 196 / 300 | 0.0% | 25.5% | 39.3% |

Proposer share (chance a free peer starts a proposal per cycle) from 0.05 to 0.6 moves the 3 cycle result only between 10.8% and 12.1% at 3000 peers, delivery 0.95.

Reading: **neither mechanism reaches the 70% floor in this model.** Converging announcements need every member to build the same party from its own sample, so any divergence (delivery below 1.0) collapses them, and even with perfect delivery they reach 56% in 3 epochs because the level-biased sample drops the same far-level ads for some members but not others. The proposer model is far less sensitive to divergence but slow, since a group gains members only one cycle at a time and fails when a cycle adds nobody. The design should be reopened per the plan ("if neither reaches the floor, reopen the design"), or the model assumptions (role mix, level spread, one swarm of 3000 for one topic, proposer share, no group merging in the proposer model) should be revisited first.
