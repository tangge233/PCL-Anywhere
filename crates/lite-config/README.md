# lite-config

Simple config file management: single-file transactions, version migration, atomic replacement.

## Usage

```rust
use lite_config::{Config, DocAccess, MigrateError, Migration, Schema, value};
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(default)]
struct AppConfig {
    max_memory_gib: u32,
    theme: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            max_memory_gib: 15,
            theme: "system".into(),
        }
    }
}

impl Schema for AppConfig {
    const VERSION: u32 = 2;

    /// v1 stored MiB; convert to GiB when upgrading to v2.
    fn migrations() -> &'static [Migration] {
        &[Migration {
            note: "max_memory_mb (MiB) becomes max_memory_gib (GiB)",
            apply: |doc| {
                if let Some(item) = doc.get_path(&["max_memory_mb"]) {
                    let mib = item
                        .as_integer()
                        .ok_or(MigrateError::Shape("max_memory_mb is not an integer"))?;
                    doc.remove_path(&["max_memory_mb"]);
                    doc.set_path(&["max_memory_gib"], value(mib / 1024))?;
                }
                Ok(())
            },
        }]
    }
}

fn main() -> Result<(), lite_config::Error> {
    let path = "/tmp/lite-config-demo/config.toml";
    // Corrupt contents are replaced by a fresh default, so one bad edit can't stop the program.
    let cfg = Config::<AppConfig>::open_or_default_replace(path)?;

    // Read: one atomic load, no lock and no IO.
    println!("max memory: {} GiB", cfg.read(|c| c.max_memory_gib));

    // Write: visible to readers immediately; the writer thread persists it after a debounce.
    cfg.mutate(|c| {
        c.max_memory_gib = 8;
        c.theme = "dark".into();
    });

    // Wait for the disk. Use this before exiting, and in tests.
    cfg.flush(Duration::from_secs(2))
}
```

The `config.toml` it writes:

```toml
version = 2
max_memory_gib = 8
theme = "dark"
```

## Guarantees

- **Crash safety**: process killed, disk full, serialization failure — the file holds either the
  complete old contents or the complete new contents, never a truncated mix.
- **Lock-free reads**: `snapshot` and `read` take no lock and do no IO.
- **Bounded writes**: one `Mutex` per file covers only "clone → mutate → publish snapshot";
  `fsync` runs on the writer thread.

## Non-guarantees

- A power loss may drop the last write (`rename` is atomic, but there is no `F_FULLFSYNC`).
- No cross-process exclusion: two processes each rewrite the whole file, so the later writer wins.
- No cross-file atomicity: changing two files is two independent commits.
- Writes rewrite the whole file, so hand-written comments and unknown keys are discarded.

The full contract is on [docs.rs](https://docs.rs/lite-config).

## License

[MIT](LICENSE).
