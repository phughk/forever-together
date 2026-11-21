# EGM iterations

One folder per elephant goldfish model pass, named after its ticket: `docs/egm/NNNN-slug/`. The rules are in [ADR 0036](../adr/0036-how-designs-and-decisions-are-maintained.md).

Layout of a folder:

```
NNNN-slug/
  README.md       status (open or closed), rounds run, links to the ticket and ADRs
  brief.md        the concepts brief
  reviews/        r1-<reviewer>.md, r2-<reviewer>.md ... one file per reviewer per round
  resolutions.md  the author's answers, dated
```

Copy `_template/README.md` when starting. Nothing else goes in a folder, and nothing from an EGM run goes anywhere else in `docs/`.

| Iteration | Ticket | Status | ADRs |
|-----------|--------|--------|------|
| none yet | | | |
