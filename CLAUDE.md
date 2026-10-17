# CLAUDE.md

Bevy 0.20 game.

## Build

Compile and run: `bevy run web`

## Code style

- No useless comments. Only doc comments on members (types, fields, functions) and comments that add info the code can't. Prefer simple, clear code over explaining it.
- Extremely minimalistic code: fewest lines, types, and abstractions that do the job.
- Always use BSN (`bsn!` / `bsn_list!`) for spawning entities and building scenes/UI when possible, instead of tuple bundles and `children![]`.
- Name major entities (`#Name` in BSN). Skip decorative parts like pins or details.
- Avoid marker components (anti-pattern).