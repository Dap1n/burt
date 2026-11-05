# Commands & Variables

All commands are prefixed with `burt_` or `bvar_`. Run any of them with `?` as the first argument for a built-in description and usage (e.g., `burt_sr_start ?`).

Note that if certain parameter accepts boolean, you may write it either with `0` & `1`, or with `true`/`t` & `false`/`f`.

If it accepts coordinates, they should be comma-separated: `123,456,789` (you can wrap them in quotes as well: `"123 456 789"`). It's done to improve readability of the command and to reduce parameters count.

### General & Utility
| Command | Description |
| --- | --- |
| `burt_echo <color> <text>` | Print colored text to the console. |
| `burt_exit` | Safely unload the plugin. |
| `burt_clear_mdmp` | Remove all `.mdmp` crash dump files in the game directory. |

### Speedrunning & Recording
| Command | Description |
| --- | --- |
| `burt_sr_start [use_vault_save]` | Start a speedrun. By default loads `vault.sav` if available, otherwise starts a new game. |
| `burt_sr_stop` | End the current speedrun manually. |
| `burt_rec [0\|1]` | Toggle automatic demo recording on level load. |

### TAS Playback
| Command | Description |
| --- | --- |
| `burt_tas_play <filename>` | Play a TAS script with given name from the `/tas` folder. |
| `burt_tas_stop` | Stop the current TAS. |
| `burt_tas_replay` | Replay the last played TAS. |
| `burt_tas_pause` | Pause the TAS playback (and the whole game). |
| `burt_tas_resume` | Resume the TAS playback (and the whole game). |
| `burt_tas_advance [ticks]` | Advance TAS playback by the specified amount of ticks (default is 2). |

### Automation Tools
| Command | Description |
| --- | --- |
| `burt_strafe [mode] [vel] [dir] [no_pitch_lock] [let_speed_lock]` | TAS autostrafe tool. |
| `burt_absmov <angle> [strength] [enabled]` | Absolute movement toward a world angle. |
| `burt_autojump [ducked] [enabled]` | Toggle automatic jumping when getting grounded. |
| `burt_decel [target_speed] [enabled]` | Stop as fast as possible via ground friction. |
| `burt_setang <pitch> <yaw> [time]` | Rotate camera to exact angles. |
| `burt_autoaim_point <x> <y> <z> [time] [enabled]` | Automatic aim at certain coordinates. |
| `burt_autoaim_entity <entity> [offset] [enabled]` | Automatic aim at a specific entity or index. |

### Debugging & Info
| Command | Description |
| --- | --- |
| `burt_getpos_exact` | Get exact position and angles of the player entity. |
| `burt_portal_report` | Get positions and angles of portals you're currently inside. |
| `burt_sg_dbg <mode>` | Manage Save Glitch debugging (`0` normal, `1` ghost, `2` body). |

---

### BURT Variables (`bvar_`)
BURT Variables are a nicer alternative to standard ConVars. They only accept valid inputs and display their current value when queried.

| Command | Description |
| --- | --- |
| `bvar_tas_check_disable <0\|1>` | Globally disable the TAS check tool. |
| `bvar_tas_check_max_replays <n>` | How many times the check tool will restart playback. |
| `bvar_tas_commands_disable <0\|1>` | Globally disable commands in TAS script. |
| `bvar_tas_interpolate <0\|1>` | Preserve client interpolation in TAS playback. |
| `bvar_tas_pause_at <tick>` | Automatically pause the TAS at the given tick. |
| `bvar_tas_playback_rate <rate>` | The rate at which to play back TAS scripts. |
| `bvar_tas_quantize_movement <0\|1>` | Whether to quantize movement to 16-bit XInput limits. |
| `bvar_tas_skip_to <tick>` | Quickly forward the TAS up to the given tick. |
| `bvar_tas_tools_disable <0\|1>` | Globally disable all TAS tools. |
