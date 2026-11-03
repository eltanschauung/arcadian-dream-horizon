# Arcadian Dream for Horizon

- `eltanschauung/arcadian-dream-horizon`: whole 0.1.0 Rust port for Horizon 26.3.
- Start: `README.md`; coverage/status: `docs/PORTING.md`. Keep verified/remaining work current at handoff.
- Map: `assets/`, `data/`; Rust `native/mixins/arcadiandream/`; generators/probes `tools/`; original behavior `reference/`.
- `python3 tools/build.py [--install]` packages/installs `.hmod`/`.hnmod`; then launcher native rebuild.
- Follow `../horizon/AGENTS.md` workflow, caches/safety; test in temporary homes/worlds.
- Code EUPL-1.2-or-later; assets CC-BY-SA-4.0; preserve credits. No Minecraft/credentials/logs/caches in Git. Packages <5 GB; commits need `Co-Authored-By:`; report new commit HEADs.
