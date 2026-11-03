# Arcadian Dream for Horizon

- Native Rust rewrite of the whole original Arcadian Dream 0.1.0; keep `docs/PORTING.md` honest about parity.
- Code: EUPL-1.2-or-later. Original assets: CC-BY-SA-4.0; preserve credits and notices.
- `python3 tools/build.py` packages `.hmod` + `.hnmod`; `--install` installs them locally. A launcher native build is then required.
- Use the sibling Horizon checkout's toolchain/cache. Test only with temporary homes/worlds; preserve accounts and existing saves.
- Push to `eltanschauung/arcadian-dream-horizon`. Never distribute Minecraft resources/JARs, credentials, logs or build caches.
- Keep build artifacts below 5 GB. Add a `Co-Authored-By:` trailer to commits.
