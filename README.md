# nier-mod-loader
Loads CPK mods and DLL plugins into NieR:Automata

Requires nightly to build:  
`cargo +nightly build --release`

## Installation
Place rename "mod_loader.dll" to "d3d11.dll" and place next to NieRAutomata.exe, OR inject/sideload by any other means early on game launch.

## Known Issues
If using SpecialK/FAR do not install as d3d11, instead configure SpecialK to load nier-mod-loader.

## Installing Mods
The mod loader can load either CPK's or plugins (.dll). Place them into `data\mods\cpks\*` and `data\mods\plugins\*` respectively. The mod loader will search either folder recursively by file extension, so mods can be at any folder depth e.g.
  - `data\mods\cpks\weapons\longswords\custom_longsword.cpk`
  - `data\mods\plugins\tools\2BHook\2BHook.dll`
  
## Configuring Mods
The mod loader will generate a config.ini inside `data\` split into a [DLL] and [CPK] section. The format will be in `FILENAME=TRUE/FALSE` where `TRUE` means the mod will be loaded, and `FALSE` (or any other value) means the mod will *not* be loaded. The `FILENAME` will only be the stem -- no path, no extension -- so 2 CPK's or 2 plugins under the same name in but installed in different paths will be checked by the same config entry.

## Important
This mod loader is still in it's very early stages, expect updates to *break* support for config.ini 

## Planned Updates (in order of importance)
  - ~~Refactor codebase to use a proper rust hooking library and reduce stink of winapi calls.~~ (kinda, uses detour-rs, still stinky)
  - Add a `[DISABLED]` section to the config for mods no longer found
  - Add support for mod load order
  - Add support for loading `core\` files from mod CPK's
  - Patch the 64 cpk mount limit
  - Add support for loading non-packed mod directories (non-cpk)
  
## Maybe Updates
  - Add support for proxying other DLL's (e.g. xinput)
