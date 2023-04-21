# nier-mod-loader
Loads CPK mods and DLL plugins into NieR:Automata

Requires nightly to build:  
`cargo +nightly build --release`

## Installation
Place rename "mod_loader.dll" to "d3d11.dll" and place in the same directory as the NieRAutomata.exe 

## SpecialK/FAR
If using SpecialK/FAR, place mod_loader.dll in the same directory as NieRAutomata.exe (but do not rename) and install SpecialK64.dll as dinput8.dll. Add the following to the end of dinput8.ini (this file is generated on first game launch)
```
[Import.ModLoader]
Architecture=x64
Role=ThirdParty
When=Early
Blacklist=
Filename=mod_loader.dll
```

## Installing Mods
The mod loader can load either CPK's or plugins (.dll). Place them into `data\mods\cpks\*` and `data\mods\plugins\*` respectively. The mod loader will search either folder recursively by file extension, so mods can be at any folder depth e.g.
  - `data\mods\cpks\weapons\longswords\custom_longsword.cpk`
  - `data\mods\plugins\tools\2BHook\2BHook.dll`
  
 *(THESE FOLDERS DO NOT EXIST BY DEFAULT)*
  
## Configuring Mods
The mod loader will generate a config.ini inside `data\` split into a [DLL] and [CPK] section. For [CPK] the format will be in `FILENAME=TRUE/FALSE` where `TRUE` means the mod will be loaded, and `FALSE` (or any other value) means the mod will *not* be loaded. For [DLL] the format will be in `FILENAME=EARLY/LATE` where `EARLY` means the plugin will be loaded immediately, and `LATE` will wait for the game to reach main menu and for [CPK]'s to finish mounting.

For The `FILENAME` will only be the stem -- no path, no extension.

## Mod load order
For each type of mod ([CPK], [DLL] (early), [DLL] (late)), they will be loaded in the order they are listed in the config. For example:
```
[DLL]
2BHook=EARLY
NASA=LATE

[CPK]
Kimono=TRUE
```
would load in the following order:
  1. 2BHook (immediately following game launch)
  2. Kimono (at main menu)
  3. NASA (at main menu following Kimono)

## Game load order
NieR:Automata loads files in the following order:
  1. Non-DLC cpk's ( < data100.cpk)
  2. DLC cpk's ( >= data100.cpk)
  3. nier-mod-loader cpk's
  4. Loose files (e.g. data\pl\pl000d.dat)
  
This means that any loose files will load *last* and if matching, will *overwrite* mod CPK's.

## Important
This mod loader is still in it's very early stages, expect updates to *break* support for config.ini 

## Planned Updates (in order of importance)
  - ~~Refactor codebase to use a proper rust hooking library and reduce stink of winapi calls.~~ (kinda, uses detour-rs, still stinky)
  - Add a `[DISABLED]` section to the config for mods no longer found
  - ~~Add support for mod load order~~ (send thanks to Petrarca for telling me to do this)
  - Add support for loading `core\` files from mod CPK's
  - Patch the 64 cpk mount limit
  - Add support for loading non-packed mod directories (non-cpk)
  
## Maybe Updates
  - Add support for proxying other DLL's (e.g. xinput)
