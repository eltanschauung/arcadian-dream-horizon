# Native port coverage

Target: the complete Arcadian Dream 0.1.0 content and gameplay roster on Horizon 26.3. This is a native implementation, not a Java/Fabric bridge. Validation below distinguishes resource/generation checks from interactive gameplay checks.

| System | Native implementation | Validation |
|---|---|---|
| Content | All 98 items/block items, 16 blocks, Arcadian Dream creative tab, original textures/models, recipes, equipment, music and painting | Item names and item/recipe/biome tables load; creative tab observed in-game |
| Ores | Original placement ranges, vein sizes and counts; additive dimension biome tags and underground decoration | Decorated Overworld, Nether and End chunks contain the ores, including with BOP + Tectonic enabled; no skipped Arcadian features |
| Mining | Tool tiers, ordinary and Silk Touch drops, Fortune, original material attributes, Hihiirokane autosmelt | Normal mining and Silk Touch checked through the game’s drop loader |
| Abandoned shrine | Original structure, template, processors, forest/taiga tags, spacing and treasure | Structure start produces a nonempty piece list with BOP + Tectonic |
| Fairies | Three mob types; original variants and geometry/animation; all 22 spell configurations; flight, anger, daylight spawning and charm suppression | Spawned and rendered in-game; flight and saved-world reload observed |
| Danmaku | Six families/patterns, modifiers, original palette, icy shots, duration/gravity, Kunai piercing, amulet undead bonus, star armor bypass, enlargement and spin | Projectile power/color/icy/scale/age roundtrip, expiry and piercing checks pass |
| Crafting | Ritual shrine + 8/16 Onbashira arrangements and dimension restrictions; seven-slot danmaku table; repair, patterns, dye, ice and time-orb modifiers | In-game shot crafting checked; full ritual-recipe matrix remains |
| Special equipment | Chisel, scythe (including PvP/creative checks), both mallets, Nue trident, Hisou and icicle swords, gourd, bombs and healing charm | Native compilation; individual gameplay checks remain |
| Hourai / ethereal | Saved doses, harmful-effect immunity, resurrection/fatigue attributes, incoming/outgoing damage rules | Native compilation; death-protection integration |
| Accessories | Saved life, charm and necklace slots; Extra Life, Fairy Charm modes and Magatama bullet cancellation | Native compilation; persistent stack components retained |
| Trading | Antiquarian job site, original prices/XP/shot ranges, five trade levels, wandering-trader additions | Offer progression checked; interactive trading remains |
| Loot / progression | Original loot additions, modern Looting functions, enchantments, custom advancement triggers and cancellation statistics | Content loads; detailed progression checks remain |
| Configuration | Original defaults, editable JSON and Mods-menu settings | Mods-menu page opened in-game; changed numeric option persisted with its integer type |

## Deliberate Horizon adaptations

- Accessories equip by using the item. `/arcadian unequip life`, `charm` or `necklace` returns an equipped stack, preserving its components. No Trinkets dependency or separate Trinkets inventory is required.
- The danmaku table commits a valid craft to its output slot when its inputs are ready. It consumes inputs once, holds the result until taken, and rejects items from its output slot. This replaces the Java preview/on-take implementation.
- Antiquarians use Horizon’s librarian entity/profession template and a danmaku-table job site, with their own name and offers. Existing vanilla librarians keep their trades. Their appearance/profession wire type is the vanilla template.
- Natural fairy spawning uses a bounded native daylight pass with the original relative fairy weights and group sizes. Temperate custom biomes remain eligible; snow-fox biome tags select ice fairies.
- Horizon supplies the base menu/status-effect presentation and armor integration. These are not a pixel-for-pixel recreation of Fabric/GeckoLib/Trinkets screens or every original particle effect.

Player state lives in the world’s `arcadiandream/players.json`. Fairy variants and spells are entity attachments; shot parameters are saved on projectiles and synchronized to clients. Settings live in the game home’s `config/arcadiandream.json` and require a restart after editing. Server settings are authoritative.

Both client and dedicated server need matching content and native packages. This port does not distribute Minecraft resources, Java libraries or a Minecraft JAR. Testing uses isolated worlds; existing worlds are never regenerated.

The installed client was also rebuilt with Arcadian Dream, ChatHeads, Eating Animation, Lunatic, Minimap, Tectonic-Horizon and Zoom, then opened in an isolated world with BOP content enabled. Saved fairies loaded and an item-give command displayed the translated item name. No existing player world or account was used for this check.

## Fairy lighting correction

Fairy models sample sky and block light at the interpolated eye position rather than inheriting the Allay template's maximum block light. Burning fairies retain maximum block light; bodies and wings otherwise follow the environment. Native preflight/build validation is recorded separately from an in-game visual check.

## Fairy animation correction

The native renderer now applies GeckoLib's baked coordinate/rotation conventions to bones, cube pivots and cube rotations, and scales about each bone pivot. The original model files remain unchanged. The three original clips (`idle`, `idle.ground`, `move`) use all their authored easing curves, including parameterized elastic easing; per-entity clocks restart on clip changes and blend for five ticks. Movement uses Java's 0.015 horizontal-motion threshold plus nonzero limb speed; hurt fairies select ground/air idle. Head look overrides match the Java models. Death roll and death tint are handled by the native renderer.

Reference: original `BaseFairyEntity.predicate/registerControllers`, fairy model classes, and [GeckoLib 1.20.1 model baking/render transforms](https://github.com/bernie-g/geckolib/tree/af3a0bd93d3367b283c9c97ca288a0bcd84e1ca8/Fabric/src/main/java/software/bernie/geckolib). `python3 tools/test_fairy_animation.py` checks easing values, state selection, clip transitions/restarts/loops, baked axes and death-roll timing using the installed Horizon toolchain/cache. Five checks pass. The combined selected-mod build compiles; the rendered pose and attachments were inspected in an isolated world and confirmed by the user. A Java client was not run side by side.

## Wall-Passing Chisel emergence

Successful chisel passage queues Minecraft's item-frame item-removal pop and a 20-particle white-smoke puff after its teleport. Both resolve at the acting player's destination through the existing authoritative command queue and standard nearby sound/particle dispatch; disabled use, blocked passage and failed searches emit neither. No custom packets, assets or persistent state are introduced.
