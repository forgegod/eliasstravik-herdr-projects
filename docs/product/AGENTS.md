# Product capability records

## Purpose

Own current material behaviour and executable evidence, not future plans.

## Ownership

`README.md` defines shape/lifecycle; `index.md` links current CAPs; `capabilities/` holds their contracts. Canonical `wireframes/` is created only when a recorded capability owns a primary human-facing surface.

## Local Contracts

- CAPs use present-tense falsifiable claims and existing implementation/test paths.
- Declare `Primary surface: human` or `none` honestly. `human` requires generated HTML and rendered PNG under the canonical wireframe contract.
- Configuration, hook protocols and automatic prompt delivery are headless contracts. This initial inventory does not claim to document the entire popup/sidebar UI.
- CAPs never link proposals or carry phase progress. Read `../changes/AGENTS.md` before visual work.

## Work Guidance

Read `README.md`; update CAPs with their implementation and tests. Extend the initial inventory when a material request touches another capability.

## Verification

`sh scripts/check-records.sh` and the affected Rust tests. Visual freshness is a separate gate when activated.

## Child DOX Index

No children. Create/index a wireframe contract on activation. Parent: `../AGENTS.md`.
