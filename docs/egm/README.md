# EGM iterations

One folder per elephant goldfish model pass, named after its ticket: `docs/egm/NNNN-slug/`. The rules are in [ADR 0036](../adr/0036-how-designs-and-decisions-are-maintained.md).

Layout of a folder:

```
NNNN-slug/
  README.md       status (open or closed), rounds run, links to the ticket and ADRs
  brief.md        the concepts brief (when the pass has one)
  design_draft.md the design draft (when the pass drafts one)
  reviews/        r1-<reviewer>.md, r2-<reviewer>.md ... one file per reviewer per round
  resolutions.md  the author's answers, dated
```

Copy `_template/README.md` when starting. Nothing else goes in a folder, and nothing from an EGM run goes anywhere else in `docs/`.

| Iteration | Ticket | Status | ADRs |
|-----------|--------|--------|------|
| [0043-key-management](0043-key-management/README.md) | [0043](../../tickets/0043-key-management.md) | open (Part 1 built, round 2 not run) | 0039, 0038 |
| [0001-verify-live-client-calls](0001-verify-live-client-calls/README.md) | [0001](../../tickets/0001-verify-live-client-calls.md) | open (initial draft, no review round) | |
| [0002-verify-qr-phone-scan](0002-verify-qr-phone-scan/README.md) | [0002](../../tickets/0002-verify-qr-phone-scan.md) | open (initial draft, no review round) | |
| [0004-verify-iroh-live-network](0004-verify-iroh-live-network/README.md) | [0004](../../tickets/0004-verify-iroh-live-network.md) | open (initial draft, no review round) | |
| [0005-verify-agreement-live](0005-verify-agreement-live/README.md) | [0005](../../tickets/0005-verify-agreement-live.md) | open (initial draft, no review round) | |
| [0006-verify-addon-installer-real-install](0006-verify-addon-installer-real-install/README.md) | [0006](../../tickets/0006-verify-addon-installer-real-install.md) | open (initial draft, no review round) | |
| [0007-addon-installer-stage-2](0007-addon-installer-stage-2/README.md) | [0007](../../tickets/0007-addon-installer-stage-2.md) | open (initial draft, no review round) | |
| [0010-trusted-peer-deployment-policy](0010-trusted-peer-deployment-policy/README.md) | [0010](../../tickets/0010-trusted-peer-deployment-policy.md) | open (initial draft, no review round) | |
| [0011-addon-page-url](0011-addon-page-url/README.md) | [0011](../../tickets/0011-addon-page-url.md) | open (initial draft, no review round) | |
| [0012-addon-releases-and-updates](0012-addon-releases-and-updates/README.md) | [0012](../../tickets/0012-addon-releases-and-updates.md) | open (initial draft, no review round) | |
| [0013-blocking-and-trust](0013-blocking-and-trust/README.md) | [0013](../../tickets/0013-blocking-and-trust.md) | open (initial draft, no review round) | |
| [0014-chat-after-party](0014-chat-after-party/README.md) | [0014](../../tickets/0014-chat-after-party.md) | open (initial draft, no review round) | |
| [0015-join-party-qr](0015-join-party-qr/README.md) | [0015](../../tickets/0015-join-party-qr.md) | open (initial draft, no review round) | |
| [0016-other-matchmaking-uses](0016-other-matchmaking-uses/README.md) | [0016](../../tickets/0016-other-matchmaking-uses.md) | open (initial draft, no review round) | |
| [0017-location-xy-in-code](0017-location-xy-in-code/README.md) | [0017](../../tickets/0017-location-xy-in-code.md) | open (initial draft, no review round) | |
| [0018-zone-matching-localisation](0018-zone-matching-localisation/README.md) | [0018](../../tickets/0018-zone-matching-localisation.md) | open (initial draft, no review round) | |
| [0019-shared-zone-dungeons](0019-shared-zone-dungeons/README.md) | [0019](../../tickets/0019-shared-zone-dungeons.md) | open (initial draft, no review round) | |
| [0020-professions-and-gear-in-code](0020-professions-and-gear-in-code/README.md) | [0020](../../tickets/0020-professions-and-gear-in-code.md) | open (initial draft, no review round) | |
| [0021-topic-sharding](0021-topic-sharding/README.md) | [0021](../../tickets/0021-topic-sharding.md) | open (initial draft, no review round) | |
| [0022-encrypted-lan-link](0022-encrypted-lan-link/README.md) | [0022](../../tickets/0022-encrypted-lan-link.md) | open (initial draft, no review round) | |
| [0023-cross-faction-groups](0023-cross-faction-groups/README.md) | [0023](../../tickets/0023-cross-faction-groups.md) | open (initial draft, no review round) | |
| [0024-re-audit-revised-design](0024-re-audit-revised-design/README.md) | [0024](../../tickets/0024-re-audit-revised-design.md) | open (initial draft, no review round) | |
| [0025-diagnosis-attempt-counters](0025-diagnosis-attempt-counters/README.md) | [0025](../../tickets/0025-diagnosis-attempt-counters.md) | open (initial draft, no review round) | |
| [0026-diagnosis-hysteresis-and-hints](0026-diagnosis-hysteresis-and-hints/README.md) | [0026](../../tickets/0026-diagnosis-hysteresis-and-hints.md) | open (initial draft, no review round) | |
| [0027-history-estimator](0027-history-estimator/README.md) | [0027](../../tickets/0027-history-estimator.md) | open (initial draft, no review round) | |
| [0028-history-clear-action](0028-history-clear-action/README.md) | [0028](../../tickets/0028-history-clear-action.md) | open (initial draft, no review round) | |
| [0029-history-checkpoints](0029-history-checkpoints/README.md) | [0029](../../tickets/0029-history-checkpoints.md) | open (initial draft, no review round) | |
| [0030-per-phone-count](0030-per-phone-count/README.md) | [0030](../../tickets/0030-per-phone-count.md) | open (initial draft, no review round) | |
| [0031-auto-off-timer](0031-auto-off-timer/README.md) | [0031](../../tickets/0031-auto-off-timer.md) | open (initial draft, no review round) | |
| [0032-remote-flag-from-server](0032-remote-flag-from-server/README.md) | [0032](../../tickets/0032-remote-flag-from-server.md) | open (initial draft, no review round) | |
| [0033-ui-scroll-edge-cases](0033-ui-scroll-edge-cases/README.md) | [0033](../../tickets/0033-ui-scroll-edge-cases.md) | open (initial draft, no review round) | |
| [0034-empty-space-components](0034-empty-space-components/README.md) | [0034](../../tickets/0034-empty-space-components.md) | open (initial draft, no review round) | |
| [0035-web-app-only-mock-tested](0035-web-app-only-mock-tested/README.md) | [0035](../../tickets/0035-web-app-only-mock-tested.md) | open (initial draft, no review round) | |
| [0036-mise-test-addon-submodule](0036-mise-test-addon-submodule/README.md) | [0036](../../tickets/0036-mise-test-addon-submodule.md) | open (initial draft, no review round) | |
| [0037-adrs-mention-old-names](0037-adrs-mention-old-names/README.md) | [0037](../../tickets/0037-adrs-mention-old-names.md) | open (initial draft, no review round) | |
| [0038-self-host-iroh-relay](0038-self-host-iroh-relay/README.md) | [0038](../../tickets/0038-self-host-iroh-relay.md) | open (initial draft, no review round) | |
| [0039-scan-log-store](0039-scan-log-store/README.md) | [0039](../../tickets/0039-scan-log-store.md) | open (initial draft, no review round) | |
| [0040-scan-log-export-api](0040-scan-log-export-api/README.md) | [0040](../../tickets/0040-scan-log-export-api.md) | open (initial draft, no review round) | |
| [0041-scan-log-ui](0041-scan-log-ui/README.md) | [0041](../../tickets/0041-scan-log-ui.md) | open (initial draft, no review round) | |
| [0042-collect-region-and-realm-values](0042-collect-region-and-realm-values/README.md) | [0042](../../tickets/0042-collect-region-and-realm-values.md) | open (initial draft, no review round) | |
| [0045-replace-dyn-store-with-generics](0045-replace-dyn-store-with-generics/README.md) | [0045](../../tickets/0045-replace-dyn-store-with-generics.md) | open (initial draft, no review round) | |
| [0046-key-actions-in-settings-and-windows](0046-key-actions-in-settings-and-windows/README.md) | [0046](../../tickets/0046-key-actions-in-settings-and-windows.md) | open (initial draft, no review round) | |
