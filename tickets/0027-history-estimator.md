# 0027. Search history estimator (stage 8b)

* **Status:** open
* **Kind:** feature
* **Source:** ADR 0027; docs/design-process/archive brief (search diagnosis)
* **ADRs:** none
* **EGM:** none

## What

Rates by state, not regression: bottleneck role at the checkpoint plus own role; share of similar past searches that found a group with an interval; dependable means 3 successes and 3 failures and an interval under 40 points. Output only as a banded clause with provenance, never a bare number, never reassuring against a live 'not connected' or 'no tank'. Build only after a live capture shows the fields are stable.

## Done when

Pure estimator module, skip-on-corrupt tests, no API field without a decision.
