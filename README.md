# Arcadian Dream — Horizon

An unofficial Rust-native rewrite of [Maxmani’s Arcadian Dream](https://github.com/Maxmani/arcadian-dream) 0.1.0 for Horizon 26.3. See [port coverage and native adaptations](docs/PORTING.md) for implementation details and validation limits.

Adds the **Arcadian Dream** creative tab, its ores/materials/equipment, fairies, danmaku and crafting systems. Ore features use dimension biome tags and the normal underground decoration step, so custom biomes and terrain remain eligible. Existing chunks are not regenerated.

## Build

With Horizon’s `cargo-horizon-mod` available:

```sh
python3 tools/build.py --packager /path/to/cargo-horizon-mod
```

Artifacts appear in `target/horizon-mod/`. `--install` copies both packages into the local Horizon mod folders. Then build the game through the launcher with trusted native mods enabled and restart it. Both client and server need matching packages.

The checked-in assets and behavior reference are sufficient to package the port; the Java original is not a build dependency. `tools/import_original.py --source /path/to/arcadian-dream` and `tools/update_reference.py /path/to/arcadian-dream` are maintainer import tools. `tools/modernize.py` updates the imported resource formats.

Accessories equip with right-click. Remove one with `/arcadian unequip life`, `/arcadian unequip charm` or `/arcadian unequip necklace`. The danmaku table crafts automatically when its inputs are valid; Faith Items repair shots and Star Items extend their duration.

Restart the game after installing or enabling the native port. Settings are in `config/arcadiandream.json` and Options → Mods → Arcadian Dream. Restart after edits; a dedicated server uses its own configuration. The original default disables drinking the Hourai Elixir.

## Credits

Original mod: **Maxmani and contributors**, including **Virtlux, Foxigami and Taiyou**. Touhou Project: **ZUN / Team Shanghai Alice**. Horizon adaptation: **eltanschauung**, with Codex assistance.

Code: **EUPL-1.2-or-later** ([LICENSE](LICENSE)). Original assets: **CC-BY-SA-4.0** ([LICENSE.ASSETS](LICENSE.ASSETS)), subject to original per-file notices. See [third-party notices](THIRD_PARTY_NOTICES.md) and [source provenance](upstream.json).

This unofficial fan port is not endorsed by the original authors, Mojang, Microsoft or Horizon. It distributes no Minecraft resources or JARs.
