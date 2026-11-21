# TASing

## Introduction

This article was inherited from [SAR](https://wiki.portal2.sr/TASing) with some modifications to match BURT.

It serves as a complete documentation of the Portal 2 Beta TASing environment implemented into BURT and the `.p2btas` scripting language used by the TAS playback. It also intends to be a guide for people interested in Portal 2 Beta TASing.

For now, the only supported beta build is `July 2009 (852_0)`.

## Legitimacy

It's worth noting that instead of simulating virtual controller like SAR do, BURT directly injects movement with `CUserCmd` and angles with `CInput`. The reason behind that is simple: SAR hooks the Steam Controller API (specifically `CInput::SteamControllerMove`). The 2009 engine has absolutely no idea what the Steam Controller API is.

However, if you add Corehub build as a non-Steam game to your library, launch it and plug in any controller, you'll notice that it works almost flawlessly. The reason behind it is that it uses `XInput` (Xbox Input) as a compatibility layer, which is older, less precise system that literally stores how you moved your joystick using a signed 16-bit integer (range `[-32768; 32767]`). Also in this build by default your `cl_forwardspeed`/`cl_sidespeed`/`cl_backspeed` is set to 450, but the game clamps it to 175 afterwards (so you don't even have to move joystick all the way to get maximum speed).

All of that effectively means that you can only "legally" set movement values that are multiple of `450.0 / 32768.0 = 0.01373291015625`. However, if you want to move at maximum speed (175 ups), you can extend the wish movement vector to be 450 in length (which is possible with joystick), so after clamping we get way more precision. This means that for vectorial strafe tool this is not really much of an issue, but it is for absmov with strength less than 1.

BURT implements quantization, so if TAS player wants to adjust move analog in certain way, provided numbers will be set to the nearest "legitimate" ones. It's only used when you change your angles, because if you don't, it's possible to just use `+strafe`, which would give us infinite precision.

Quantization can be disabled with `bvar_tas_quantize_movement 0`.

## Setup

- Install [BURT](https://github.com/Dap1n/burt/releases) if you don't already have it.
- Create a new folder called `tas` in your Portal 2 Beta directory. This is where all your TAS scripts will live.
- The script files will need to have a `.p2btas` extension and can be played using the command `burt_tas_play`. For example, if your script file is `game/tas/mytas.p2btas`, then to play it in-game you would use the command `burt_tas_play mytas`.

## The .p2btas file format

The `.p2btas` format is almost the same thing as SAR `.p2tas` with small additional features like support of slowtime, origin offsets in autoaim tool and nested comments.

### Header

Every `.p2btas` script needs to start with a header, which will give TAS player essential info for script playback.

#### Version indicator

At the beginning of every script, you have to include a `version` command. This number lets the TAS tools know which version of the tools to use in order to maintain compatibility with older scripts. The latest version number is `1`.

#### Start commands

After the `version` command, you have to include a `start` command. It can have five parameters:

- `now`: Starts playing the TAS immediately, or as soon as a session is started if you are not in one (i.e. in the main menu).
- `save <save>`: Loads the given save and then immediately starts playing the TAS.
- `map <map>`: Loads the given map and then immediately starts playing the TAS.
- `next`: Emulates one of the above start types, but waits until the next session load to start TAS playback. If left blank, BURT presumes a start type of `now`.
  - BURT will wait until the next session load to start TAS playback, but in the case of `save` and `map`, BURT will immediately load the save/map requested. This means that BURT will load the save/map, wait until the next load, and then start TAS playback.

#### Header examples

Below you can see some examples of how a header file may look like in your script.

Loads Prehub 2 and immediately starts the TAS:
```
version 1
start map p2_lab_prehub_2
```

Immediately loads the save `tas_aquarium` and starts the TAS:
```
version 1
start save tas_aquarium
```

Loads the save `catapult_pre`, waits until the next session load, then starts the TAS:
```
version 1
start next save catapult_pre
```

In this example, the save `catapult_pre` was made on the last tick of a TAS for the map `p2_lab_hub_2`, meaning that it loads into a save on Hub 2, then a tick later the level transition completes, and then the TAS starts playback at the beginning of `p2_lab_catapult`. The end result looks very similar to `start map p2_lab_catapult` (Catapult), but is arguably "more legitimate".

### Comments

Comments are sections that will be ignored and can be expressed in 2 ways:

- Single line comments with `//`
- Multi-line comments starting with `/*` and ending with `*/`

They are useful for adding notes in your script or to temporarily prevent a line from being executed.

### Framebulk

Framebulks store the inputs and commands that should be executed by the plugin at a given tick. Every line that isn’t a header line, a repeat block (explained later) or a comment is considered to be a framebulk. They’re formatted like this:

```
tick>movement|angles|buttons|commands|tools
```

Below is an explanation of each field in a framebulk:

#### Tick

The number of ticks since the beginning of the TAS. This dictates when the framebulk will be executed. It can also be written with a `+` in front, in which case it will indicate the number of ticks compared to the previous framebulk instead.

Examples:
- If a tick value is `20`, the framebulk will be executed 20 ticks after TAS script has started.
- If a tick value is `+10`, the framebulk will be executed 10 ticks after the previous framebulk.

#### Movement

Two floating point values from `-1.0` and `1.0` representing horizontal and vertical movement analog values respectively.

Example: `0 1` as movement field will move the player forward.

These values are multiplied by 175, which is real maximum player speed you can get that way. In actual controller they're multiplied by 450, which makes things buggy because they're still get clamped to 175 afterwards, so we mimic perfect controller in that specific case for simplicity.

#### Angles

Two floating point values representing horizontal and vertical view analog values respectively. The values correspond to view rotation in degrees in one tick.

Example: `5 0` as analog field will turn the player 5 degrees to the right in the given tick.

#### Buttons

A list of digital inputs that should be pressed or released. Each digital action has a letter assigned. These actions are <u>J</u>ump, <u>D</u>uck, <u>U</u>se, <u>S</u>lowtime, <u>B</u>lue portal and <u>O</u>range portal. Uppercase letter presses down the input, lowercase letter releases it. Additionally, by putting a number after an uppercase button, you can specify a number of ticks after which it will be released automatically.

Examples:
- `D` will press the duck input. `d` will release the duck input.
- `D8` will press and hold the duck input for 8 ticks, then release it.
- `dU1J` will release the duck input, press the use input for 1 tick, and press the jump input. All other inputs will remain at their previous state.

Note that during slowtime ticks count in the same way, just with duration between them changed based on `slowtime_speed` ConVar. This means that after using the slowtime you should write your TAS like it wasn't ever executed in the first place.

#### Commands

A string representing a console command or multiple console commands separated by semicolons. Everything typed here will be executed in the console at the given tick.

Example: `say I think we're in a cereal factory; record factory` will print "I think we're in a cereal factory" in game’s chat and record a demo named "factory".

#### Tools

A string representing an automation tool command or a set of tool commands separated by semicolons. The usage of automation tools will be explained later.

Everything connected together gives us an example of a fully populated framebulk:

```
20>0 1|5 0|dU1J|say I think we're in a cereal factory; record factory|strafe max
```

### Preserving rules

The TAS script player will keep the previously set state of inputs if no information about it was given for current tick. This allows you to skip certain parts of a framebulk or even entire framebulks if you want specific input to continue for a specific amount of ticks.

Examples below are just random framebulks. Dots at the top and the bottom are there to show that it’s only a part of a full script, and you should think about these examples as such.

**Example 1:** There are no framebulks for ticks 11 to 19, the player will just continue going forward until the tick 20.
```
...
10>0 1|0 0|jduzbo||
20>0 0|0 0|jduzbo||
...
```

**Example 2:** Framebulk 15 is missing information about button states. Jump will be held for that tick.
```
...
14>0 0|0 0|Jduzbo||
15>0 0|0 0|||
...
```

**Example 3:** Framebulk 21 has only movement field and 22 has only button field with lowercase d. Player will move for 11 ticks while trying to crouch, stop moving at tick 21 and stop crouching at tick 22.
```
...
10>0 1||D
21>0 0
22>||d
...
```

Notice that, even though fields are not mandatory, you have to use the separator `|` characters to reach the correct field. Button field is the third one, so even if you don’t use movement and analog fields, you have to put two separators before it.

Additionally, commands and tools fields are not preserved. In other words, they won't be repeatedly executed every tick until the field is “cleaned up”. They will execute only once in a tick specified by a framebulk.

### Tools bulk

When TASing, you'll find yourself commonly writing framebulks that only contain information in the tools field. For instance:
```
+7>||||strafe vec max 0deg
```

To improve readability of such bulks, BURT has 'tools bulks', which only contain the tools field:

```
tick>>tools
```

The previous example as a tool bulk would be:
```
+7>>strafe vec max 0deg
```

### Loops

By putting a part of your script in a `repeat` block, you can repeat it X amount of times.

Example:
```
repeat 10
  +2>||U1
end
```

This block will spam the use key ten times. The indentation is not required.

These blocks can be nested. Also, keep in mind that both `repeat` and `end` keywords are case-sensitive (they have to be lower-case to work).

## Automation tools

Automation tools are controlled through the tools field in a framebulk, which works in a similar way to the commands field. They work by modifying the view/movement analog and button fields in the framebulk every tick.

### Tools execution order

Just like in SAR, tools have static execution order: `check`, `setang`, `autoaim`, `autojump`, `absmov`, `strafe` and `decel`.

For instance, if the tools field is `strafe max; setang 10 20`, the `setang` tool will be run before the `strafe` tool, which means the autostrafer is able to "predict" the angle change caused by the `setang` (which is important since angle changing is the first thing to happen in a tick). In general, angle-changing tools (`setang`, `autoaim`) are executed before the movement tools (`absmov`, `strafe`, `decel`) so they can work accurately.

### Strafe tool

The `strafe` tool will adjust player input to get a different kind of strafing depending on parameters.

Syntax: `strafe [parameters]`

Possible parameters:

**Strafing type control:**
- `none` or `off` - disables strafing entirely.
- `vec` - enables vectorial strafing (movement analog is adjusted to get desired movement direction). (default)
- `ang` - enables angular strafing (view analog is adjusted to get desired movement direction). This isn't particularly recommended as it doesn't look appealing.
- `veccam` - enables special vectorial strafing that rotates you towards your current moving direction.

**Velocity control:**
- `max` - makes autostrafer aim for the greatest acceleration. (default)
- `keep` - makes autostrafer maintain the current velocity.
- `[number]ups` - sets a target velocity of [number] units per second for the autostrafer.

**Strafe direction control:**
- `forward` - autostrafer will try to strafe in a straight line, towards the current view angle. (default)
- `forwardvel` - autostrafer will try to strafe in a straight line, towards the current velocity angle.
- `left` - autostrafer will try to strafe left.
- `right` - autostrafer will try to strafe right.
- `[number]deg` - sets a target yaw angle of [number] degrees autostrafer should strafe towards.

**Other:**
- `nopitchlock` - Make the autostrafer not clamp the pitch. The autostrafer will always clamp your pitch angle (up and down) between -30 and 30 when midair, as it gives the fastest possible acceleration (forward movement is being scaled by a cosine of that angle while being airborne). This argument will tell the autostrafer that you wish to enable sub-optimal strafing (this is useful when you need to hit a shot while strafing for example).
- `letspeedlock` - By default, autostrafer will attempt to not deviate too much right or left when the velocity is higher than or equal 300 ups, to avoid speedlock and keep acceleration at its maximum. Sometimes that behaviour is not wanted, in which case this parameter should be used.

Examples:
- `strafe 299.999ups left veccam` - strafes left while facing current view angle and trying to reach the velocity of 299.999ups (the highest velocity you can get without being affected by the limited air control).
- `strafe vec max forward` - strafes with maximum acceleration on a straight line without changing view angles.

### Autojump tool

Autojump tool will change the jump button state depending on whether the player is grounded or not, resulting in automatically jumping on the earliest contact with a ground.

Syntax: `autojump [on|ducked]`

Anything other than `on` or `ducked` will disable the autojump.

Examples:
- `autojump on` - enables autojump.
- `autojump ducked` - enables autojump while also ducking. Ducking slightly increases your jump height.
- `autojump off` - disables autojump.

### Absolute movement tool

Absolute movement tool will generate movement values depending on the absolute move direction you provide in degrees.

Syntax: `absmov <angle> [strength]`

Giving `off` as an argument will disable the tool. The strength parameter must be between 0 and 1 (default) and controls how fast the player will move.

Examples:
- `absmov 90deg` - enables absolute movement towards positive Y.
- `absmov 90deg 0.5` - enables absolute movement towards positive Y, but half as fast.
- `absmov off` - disables absolute movement.

### Set view angles tool

This tool works basically the same as `setang` console command. It will adjust the view analog in a way so the camera is looking towards given angles.

Syntax: `setang <pitch> <yaw> [time] [easing]`

The tool will set the pitch and yaw according to given angles in degrees. `time` specifies how long a transition to new angles takes, in ticks. If omitted, transition happens instantly. `easing` specifies which algorithm to use for interpolation. Options are `cubic`, `exp`/`exponential`, `linear` and `sin`/`sine`. If omitted, `linear` is used.

Examples:
- `setang 69 42.0` - instantly rotates the camera to angles 69 42.
- `setang 0 0 20` - slowly rotates the camera towards angles 0 0 over the next 20 ticks.
- `setang 89 0 10 exp` - rotates the camera towards angles 89 0 over 10 ticks using the exponential easing.

### Auto aim tool

The Auto Aim tool will automatically aim towards a specified point in 3D space.

Syntax: `autoaim <x> <y> <z> [time] [easing]` OR `autoaim ent <name> [offset_x] [offset_y] [offset_z] [time] [easing]`

Giving `off` as an argument will disable the tool. Time specifies how long a transition to aiming at the point takes, in ticks. If omitted, transition happens instantly. When using this command to place portals, the command `burt_portal_report` can be helpful.

`autoaim ent` will aim at the absolute origin of a named entity with optional offsets from the origin. If multiple entities have the same name you can use `autoaim ent name[n]` (with square brackets) to select the n'th entity with that name or classname (starting at 0), or `autoaim ent <index>` to aim at an entity with a certain index. To find the name or index of an entity, do `developer 1; ent_text` in console while looking at it. The first line is `(<index>) Name: <name> (<classname>)`.

The easing types are the same as the `setang` tool above.

Examples:
- `autoaim 0 0 0` - aims at world origin.
- `autoaim 0 0 0 20` - aims at world origin, but smoothed over 20 ticks.
- `autoaim 0 0 0 20 sin` - aims at world origin, but smoothed over 20 ticks using the sine easing.
- `autoaim -900 -2200 2525` - aims at dual portal gun in Prehub 1.
- `autoaim ent neckSpark 0 0 0 10` - aims at GLaDOS neck, smoothed over 10 ticks.
- `autoaim ent neckSpark 0 0 0 10 exp` - aims at GLaDOS neck, smoothed over 10 ticks using the exponential easing.
- `autoaim ent GlaDOS_slideshow_2[1] 30 100 30 20` - aims at the **2nd** entity with the name 'GlaDOS_slideshow_2' (presented in Prehub 2), with offset x:30 y:100 z:30 from it's origin, smoothed over 20 ticks.
- `autoaim ent 18 0 0 0 30` - aims at entity index 18 (Laser Emitter in p2_lab_lasers_1), smoothed over 30 ticks.

### Deceleration tool

The deceleration tool will slow down as quickly as possible to the given speed.

Syntax: `decel <speed>`

Giving `off` as an argument will disable the tool.

Examples:
- `decel 100ups` - decelerates until it reaches 100ups and then turns off.
- `decel off` - turns off the decel tool.

### Check tool

The check tool will check the player position and angle, and if it doesn't match what is specified in the arguments, it will automatically restart TAS playback. This is useful for inconsistent scripts, as the check tool can automatically retry TAS playback until it is successful. The bvar `bvar_tas_check_max_replays` (default value 15) defines how many times the check tool will restart playback until it gives up.

Syntax: `check pos <x> <y> <z> ang <pitch> <yaw> vel <velocity> posepsilon <value> angepsilon <value> velepsilon <value> holding [entity selector]`

- x, y, and z are the desired x, y, and z positions of the player (NOT the player camera).
- pitch and yaw are the desired pitch and yaw of the player.
- velocity is the desired velocity vector length of the player.
- posepsilon, angepsilon and velepsilon values are the deviations from actual player position that the check tool will tolerate. If left unspecified, default values will be 0.5 for posepsilon value, 0.2 for angepsilon value and 1.0 for velepsilon value.
- entity selector is an optional specifier for which object the player should be holding. See the [Autoaim](#auto-aim-tool) section for more information.

**IMPORTANT NOTE:** The check tool runs before all of the other tools and movement in the tick, so it effectively checks the player position and angle of the previous tick. That means that when you are getting your data for the check tool (by writing `burt_getpos_exact` as tick command, for instance), you should get the data from TAS tick `n`, and then insert the check command on framebulk `n+1`.

None of the arguments is required (but at least one must be specified), and their order in the tool does not matter.

Examples:
- `60>||||check pos 315 598 2145` - Checks if the player position in tick 59 is within 0.5 (default) units of 315, 598, 2145.
- `142>||||check pos 4105.78 150.17 2617.03 ang 1.60 178.54 vel 220 posepsilon 0.05 angepsilon 1 velepsilon 0.3` - Checks if the player position in tick 141 is within 0.05 units of 4105.78, 150.17, 2617.03, the player angle is within 1 degree of 1.60, 178.54 and the player velocity is within 0.3 ups of 220.
- `71>>check holding` - Check if the player is holding anything on tick 70.
- `421>>check holding box` - Check if the player is holding an entity named "box" (see `ent_text`) on tick 420.

## Raw script

After each tools script playback (a playback of script which contains tools), a new script with the same name followed by `_raw` suffix is created in the script's directory, which is essentially a dump of all player inputs during TAS playback. This means that it should lead to the same result, but without using any tools. The playback of any script with `_raw` suffix will use simplified and more legitimate input injection which does not process any automation tool.

This has several purposes:
- our current verification method - tools playback will inject inputs while they're processed, potentially making the game behave in an unintentional way, while raw playback will behave almost as it was a custom hardware plugged into the computer, injecting inputs while they're created.
- raw player input generation, which we can potentially use in the future for better verification methods.
- universal script format - raw script will yield similar results regardless of script version or operating system.

Once your TAS is finished, it is strongly recommended to record a demo of a raw script instead of the tools script.

## Script example

Here's an example of a full TAS script, launching off the catapult in Catapult course.

```
version 1
start map p2_lab_catapult

103>>absmov 90deg // going to the back a little
+20>>autojump ducked; strafe max // strafing to the catapult
+10>
repeat 20
  +2>||B1 // spam portal shots (these betas have no cooldown!)
end
350>
```

## Other useful things

The TASing experience can be aided by a small number of other tools such as:

## In-game commands

You can see a list of the TAS commands by typing `find burt_tas` / `find bvar_tas` in the in-game console, however here is a list of them:

- `burt_tas_play` - plays the given TAS script.
- `burt_tas_stop` - Stop the currently playing TAS.
- `burt_tas_replay` - Replay the last played TAS.
- `burt_pause` - Pause the whole game (and the TAS playback).
- `burt_resume` - Resume the whole game (and the TAS playback).
- `burt_tas_advance [ticks]` - Advances TAS playback by the specified amount of ticks, 2 by default.

------

- `bvar_tas_check_disable` - Globally disables the TAS check tool.
- `bvar_tas_check_max_replays` - How many times the check tool will restart playback.
- `bvar_tas_commands_disable` - Globally disables commands in TAS script.
- `bvar_tas_interpolate` - Preserves client interpolation in TAS playback.
- `bvar_tas_pause_at` - Automatically pauses the TAS at the given tick.
- `bvar_tas_playback_rate` - The rate at which to play back TAS scripts.
- `bvar_tas_quantize_movement` - Whether to quantize movement.
- `bvar_tas_skip_to` - Quickly forwards the TAS up to the given tick.
- `bvar_tas_tools_disable` - Globally disables all TAS tools.

## Version history

As updates to the TASing tools and environment continue to be made, scripts made on previous versions should retain their functionality, i.e. breaking changes should be kept to an absolute minimum.

| Version number | Changes |
|---|---|
| 1 | N/A |
