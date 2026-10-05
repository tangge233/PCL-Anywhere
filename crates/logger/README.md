# logger

Application-wide logging for PCL-Anywhere.

Messages are recorded with the `log` macros and written to files by
`flexi_logger`. gpui and gpui-kit use the same `log` API, so their messages are
written as well.

This crate covers the launcher process only. Game process output and crash
reports are not handled here.

## Quick start

Call `init` once during startup. After that, any thread can log.

```rust
fn main() {
    // Bind the guard to a variable. Dropping it shuts the writer down, so
    // `let _ = ...` would drop it right away and no further records are written.
    let _log = logger::init(logger::Options::default()).expect("failed to initialize logging");

    // ... open the main window ...
}
```

`Options::default()`:

| Field | Default | Meaning |
| --- | --- | --- |
| `dir` | `None` | `logger::log_dir()`: `$XDG_CONFIG_HOME/pcl-anywhere/logs`, or `~/.config/pcl-anywhere/logs` when `XDG_CONFIG_HOME` is unset |
| `level` | `Info` in debug builds, `Warn` in release builds | Default level for every target |
| `targets` | empty | Per-target level overrides |
| `rotation` | `BySession` | When to start a new file |
| `keep` | `20` | Maximum number of log files left in the directory |
| `console` | `Warn` | Lowest level that is also written to stderr |
| `buffered` | `false` | Write each record immediately. `true` batches writes, so the last batch can be lost if the process dies |
| `latest_link` | `true` | Maintain a `latest.log` symlink (Unix only) |
| `utc` | `false` | Use UTC instead of local time |

## Writing messages

`logger` re-exports `log`, so callers do not need it as a separate dependency.

```rust
use logger::log;

log::info!(target: "Setup", "setup page initialized");
log::warn!(target: "Download", "checksum mismatch, retrying from the mirror");
log::error!(target: "Instance", "failed to open the settings window: {error}");
```

Every line has the form `[timestamp] [level] [target] message`:

```
[2026-10-04 12:00:00.123] [INFO ] [Setup] setup page initialized
[2026-10-04 12:00:01.004] [WARN ] [Download] checksum mismatch, retrying from the mirror
```

The level is padded to five characters so the remaining fields line up.

`target` is the third field, and it is also the string used for level
filtering (next section). If you omit `target:`, the `log` crate fills it with
the module path instead, for example `pcl_ui::pages::setup`.

Pick one short noun per area and use it consistently: `App`, `Setup`,
`Instance`, `Download`, `Launch`, `Config`.

## Per-target levels

Level filtering compares the key against the `target` string. A key applies to
every target that starts with it, and the comparison is case-sensitive.

```rust
let options = logger::Options {
    level: log::LevelFilter::Warn, // all targets default to warn
    targets: vec![("Download".into(), log::LevelFilter::Debug)], // except Download
    ..Default::default()
};
```

To raise the level of one target without restarting, set `PCL_LOG`:

```bash
PCL_LOG="warn,Download=debug" ./pcl-app
```

`PCL_LOG` takes precedence over `Options`. The syntax is the same as
`RUST_LOG`; names are case-sensitive, so the key must match the `target`
spelling exactly.

## File rotation

Exactly one mode is active at a time.

| Mode | A new file starts |
| --- | --- |
| `Rotation::BySession` (default) | On every process start |
| `Rotation::BySize { max_bytes }` | When the current file exceeds `max_bytes` |
| `Rotation::ByDate` | On every new calendar day |

The resulting names, all inside `dir`:

| Mode | Files |
| --- | --- |
| `BySession` | `pcl-anywhere_2026-10-04_12-00-00.log` |
| `BySize` | `pcl-anywhere_2026-10-04_12-00-00_rCURRENT.log` while writing, plus `pcl-anywhere_2026-10-04_12-00-00_r00000.log`, `..._r00001.log`, ... for the finished ones |
| `ByDate` | `pcl-anywhere_rCURRENT.log` for the current day; finished days are renamed, for example `pcl-anywhere_r2026-10-04_23-59-59.log` |
| all modes | `latest.log`, a symlink to the file being written; `tail -F latest.log` follows it across rotations |

`keep` is a hard limit. On every `init`, files whose names start with
`pcl-anywhere` are deleted oldest first until `keep` remain. Symlinks and files
with other names are not touched.

## Full options

```rust
// Assumes the surrounding function returns Result.
let _log = logger::init(logger::Options {
    dir: Some(logger::log_dir()?), // passing it explicitly; same value as the default
    level: log::LevelFilter::Info,
    rotation: logger::Rotation::BySize { max_bytes: 8 * 1024 * 1024 },
    keep: 5,
    console: logger::Console::Off, // do not write to stderr
    ..Default::default()
})?;
```

## Changing the level at runtime

Once the settings page or the config file is wired up, use these functions
instead of keeping a reference to the guard:

```rust
// Changes the default level; Options::targets is kept.
logger::set_level(log::LevelFilter::Debug)?;

// Replaces the whole specification, so Options::targets no longer applies.
// A later set_level rebuilds the specification from Options and brings it back.
logger::set_spec("warn,Instance=info")?;
```

## Other methods

```rust
let guard = logger::init(options)?;

guard.flush(); // no-op in the default write mode; use it to push buffered records out
guard.rotate_now()?; // start a new file now
guard.log_files()?; // log files that exist right now, including the open one
logger::log_dir()?; // the log directory, for an "open log folder" action
```

## Pitfalls

- Bind the guard to a variable. `let _log = logger::init(..)` works;
  `let _ = logger::init(..)` drops the guard immediately and stops logging.
- `init` succeeds once per process. A second call returns
  `Error::AlreadyInitialized`; it does not replace the first logger.
- Do not call `init` from tests. All tests in one binary share a process, and
  the global logger can be installed only once. This crate keeps its unit tests
  on a constructor that installs nothing globally, and puts the real global path
  in `tests/global.rs`, one test per file.
- Do not make `Debug` the default level. gpui, wgpu and cosmic-text log on every
  frame at DEBUG; a measured 20-second session wrote 8 MB. Raise a single target
  with `targets` or `PCL_LOG` instead.

## Guarantees

- Exactly one logger per process. A second `init` fails instead of silently
  replacing the first.
- All three rotation modes write into the same directory with the same file name
  prefix, and `keep` is enforced at startup.
- In the default (unbuffered) write mode a record reaches the kernel before the
  macro returns, so killing the process does not lose lines that were logged.

## Non-guarantees

- Two processes that share a directory write independently, and the cleanup in
  one may delete a file the other is still writing to. On Linux deleting an open
  file succeeds, and the other process keeps writing to the unlinked inode.
- Low-level failures such as a full disk or an unwritable directory are reported
  by flexi_logger on stderr; the caller does not see them.
- No crash reports, and no capture of child process output.

## License

[GPL-3.0](../../LICENSE).
