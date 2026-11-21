# 0011. Supply the addon page URL

* **Status:** open
* **Kind:** prerequisite
* **Source:** ADR 0031; ui audit
* **ADRs:** none
* **EGM:** none

## What

`ADDON_URL` in `webapp/src/lib/links.ts` is empty, so step 1 of the first run is plain text. Publish the addon page (CurseForge or GitHub release) and set it.

## Done when

The first-run step links to the addon page.
