# Third-party components and notices

Telerust is distributed under GPL-3.0-or-later; see [LICENSE](LICENSE). The workspace implementation in this repository is original project code. The available Git history is one initial snapshot; source review found no copied implementation from Telegram Desktop or AyuGram. Telerust calls the documented TDLib JSON C API and does not contain source code from either client.

## Rust dependencies

Resolved registry packages and license expressions from their crate manifests (`cargo metadata`):

| Crate | Version | Declared license | Source |
|---|---:|---|---|
| `aes` | 0.8.4 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/aes/0.8.4) |
| `async-broadcast` | 0.7.2 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/async-broadcast/0.7.2) |
| `async-channel` | 2.5.0 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/async-channel/2.5.0) |
| `async-io` | 2.6.0 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/async-io/2.6.0) |
| `async-lock` | 3.5.0 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/async-lock/3.5.0) |
| `async-process` | 2.5.0 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/async-process/2.5.0) |
| `async-recursion` | 1.2.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/async-recursion/1.2.0) |
| `async-signal` | 0.2.14 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/async-signal/0.2.14) |
| `async-task` | 4.7.1 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/async-task/4.7.1) |
| `async-trait` | 0.1.92 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/async-trait/0.1.92) |
| `atomic-waker` | 1.1.2 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/atomic-waker/1.1.2) |
| `autocfg` | 1.5.1 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/autocfg/1.5.1) |
| `bitflags` | 2.13.2 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/bitflags/2.13.2) |
| `block-buffer` | 0.10.4 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/block-buffer/0.10.4) |
| `block-padding` | 0.3.3 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/block-padding/0.3.3) |
| `blocking` | 1.7.0 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/blocking/1.7.0) |
| `cairo-rs` | 0.21.5 | MIT | [crates.io](https://crates.io/crates/cairo-rs/0.21.5) |
| `cairo-sys-rs` | 0.21.5 | MIT | [crates.io](https://crates.io/crates/cairo-sys-rs/0.21.5) |
| `cbc` | 0.1.2 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/cbc/0.1.2) |
| `cfg-expr` | 0.20.10 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/cfg-expr/0.20.10) |
| `cfg-if` | 1.0.5 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/cfg-if/1.0.5) |
| `cfg_aliases` | 0.2.2 | MIT | [crates.io](https://crates.io/crates/cfg_aliases/0.2.2) |
| `cipher` | 0.4.4 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/cipher/0.4.4) |
| `concurrent-queue` | 2.5.0 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/concurrent-queue/2.5.0) |
| `cpufeatures` | 0.2.17 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/cpufeatures/0.2.17) |
| `crossbeam-utils` | 0.8.23 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/crossbeam-utils/0.8.23) |
| `crypto-common` | 0.1.7 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/crypto-common/0.1.7) |
| `dbus` | 0.9.12 | Apache-2.0/MIT | [crates.io](https://crates.io/crates/dbus/0.9.12) |
| `dbus-secret-service` | 4.1.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/dbus-secret-service/4.1.0) |
| `digest` | 0.10.7 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/digest/0.10.7) |
| `endi` | 1.1.1 | MIT | [crates.io](https://crates.io/crates/endi/1.1.1) |
| `enumflags2` | 0.7.12 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/enumflags2/0.7.12) |
| `enumflags2_derive` | 0.7.12 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/enumflags2_derive/0.7.12) |
| `equivalent` | 1.0.3 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/equivalent/1.0.3) |
| `errno` | 0.3.14 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/errno/0.3.14) |
| `event-listener` | 5.4.2 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/event-listener/5.4.2) |
| `event-listener-strategy` | 0.5.4 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/event-listener-strategy/0.5.4) |
| `fastrand` | 2.5.0 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/fastrand/2.5.0) |
| `field-offset` | 0.3.6 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/field-offset/0.3.6) |
| `futures-channel` | 0.3.34 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/futures-channel/0.3.34) |
| `futures-core` | 0.3.34 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/futures-core/0.3.34) |
| `futures-executor` | 0.3.34 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/futures-executor/0.3.34) |
| `futures-io` | 0.3.34 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/futures-io/0.3.34) |
| `futures-lite` | 2.6.1 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/futures-lite/2.6.1) |
| `futures-macro` | 0.3.34 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/futures-macro/0.3.34) |
| `futures-sink` | 0.3.34 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/futures-sink/0.3.34) |
| `futures-task` | 0.3.34 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/futures-task/0.3.34) |
| `futures-util` | 0.3.34 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/futures-util/0.3.34) |
| `gdk-pixbuf` | 0.21.5 | MIT | [crates.io](https://crates.io/crates/gdk-pixbuf/0.21.5) |
| `gdk-pixbuf-sys` | 0.21.5 | MIT | [crates.io](https://crates.io/crates/gdk-pixbuf-sys/0.21.5) |
| `gdk4` | 0.10.3 | MIT | [crates.io](https://crates.io/crates/gdk4/0.10.3) |
| `gdk4-sys` | 0.10.3 | MIT | [crates.io](https://crates.io/crates/gdk4-sys/0.10.3) |
| `generic-array` | 0.14.7 | MIT | [crates.io](https://crates.io/crates/generic-array/0.14.7) |
| `getrandom` | 0.2.17 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/getrandom/0.2.17) |
| `getrandom` | 0.3.4 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/getrandom/0.3.4) |
| `getrandom` | 0.4.3 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/getrandom/0.4.3) |
| `gio` | 0.21.5 | MIT | [crates.io](https://crates.io/crates/gio/0.21.5) |
| `gio-sys` | 0.21.5 | MIT | [crates.io](https://crates.io/crates/gio-sys/0.21.5) |
| `glib` | 0.21.5 | MIT | [crates.io](https://crates.io/crates/glib/0.21.5) |
| `glib-macros` | 0.21.5 | MIT | [crates.io](https://crates.io/crates/glib-macros/0.21.5) |
| `glib-sys` | 0.21.5 | MIT | [crates.io](https://crates.io/crates/glib-sys/0.21.5) |
| `gobject-sys` | 0.21.5 | MIT | [crates.io](https://crates.io/crates/gobject-sys/0.21.5) |
| `graphene-rs` | 0.21.5 | MIT | [crates.io](https://crates.io/crates/graphene-rs/0.21.5) |
| `graphene-sys` | 0.21.5 | MIT | [crates.io](https://crates.io/crates/graphene-sys/0.21.5) |
| `gsk4` | 0.10.3 | MIT | [crates.io](https://crates.io/crates/gsk4/0.10.3) |
| `gsk4-sys` | 0.10.3 | MIT | [crates.io](https://crates.io/crates/gsk4-sys/0.10.3) |
| `gtk4` | 0.10.1 | MIT | [crates.io](https://crates.io/crates/gtk4/0.10.1) |
| `gtk4-macros` | 0.10.3 | MIT | [crates.io](https://crates.io/crates/gtk4-macros/0.10.3) |
| `gtk4-sys` | 0.10.3 | MIT | [crates.io](https://crates.io/crates/gtk4-sys/0.10.3) |
| `hashbrown` | 0.17.1 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/hashbrown/0.17.1) |
| `heck` | 0.5.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/heck/0.5.0) |
| `hermit-abi` | 0.5.3 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/hermit-abi/0.5.3) |
| `hex` | 0.4.3 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/hex/0.4.3) |
| `hkdf` | 0.12.4 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/hkdf/0.12.4) |
| `hmac` | 0.12.1 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/hmac/0.12.1) |
| `indexmap` | 2.14.2 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/indexmap/2.14.2) |
| `inout` | 0.1.4 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/inout/0.1.4) |
| `itoa` | 1.0.19 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/itoa/1.0.19) |
| `keyring` | 3.6.3 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/keyring/3.6.3) |
| `libadwaita` | 0.8.0 | MIT | [crates.io](https://crates.io/crates/libadwaita/0.8.0) |
| `libadwaita-sys` | 0.8.1 | MIT | [crates.io](https://crates.io/crates/libadwaita-sys/0.8.1) |
| `libc` | 0.2.191 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/libc/0.2.191) |
| `libdbus-sys` | 0.2.7 | Apache-2.0/MIT | [crates.io](https://crates.io/crates/libdbus-sys/0.2.7) |
| `libloading` | 0.8.9 | ISC | [crates.io](https://crates.io/crates/libloading/0.8.9) |
| `linux-raw-sys` | 0.12.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/linux-raw-sys/0.12.1) |
| `log` | 0.4.35 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/log/0.4.35) |
| `memchr` | 2.8.3 | Unlicense OR MIT | [crates.io](https://crates.io/crates/memchr/2.8.3) |
| `memoffset` | 0.9.1 | MIT | [crates.io](https://crates.io/crates/memoffset/0.9.1) |
| `nix` | 0.29.0 | MIT | [crates.io](https://crates.io/crates/nix/0.29.0) |
| `num` | 0.4.3 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/num/0.4.3) |
| `num-bigint` | 0.4.8 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/num-bigint/0.4.8) |
| `num-complex` | 0.4.6 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/num-complex/0.4.6) |
| `num-integer` | 0.1.47 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/num-integer/0.1.47) |
| `num-iter` | 0.1.46 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/num-iter/0.1.46) |
| `num-rational` | 0.4.2 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/num-rational/0.4.2) |
| `num-traits` | 0.2.19 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/num-traits/0.2.19) |
| `once_cell` | 1.21.4 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/once_cell/1.21.4) |
| `ordered-stream` | 0.2.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/ordered-stream/0.2.0) |
| `pango` | 0.21.5 | MIT | [crates.io](https://crates.io/crates/pango/0.21.5) |
| `pango-sys` | 0.21.5 | MIT | [crates.io](https://crates.io/crates/pango-sys/0.21.5) |
| `parking` | 2.2.1 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/parking/2.2.1) |
| `pin-project-lite` | 0.2.17 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/pin-project-lite/0.2.17) |
| `piper` | 0.2.5 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/piper/0.2.5) |
| `pkg-config` | 0.3.34 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/pkg-config/0.3.34) |
| `polling` | 3.11.0 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/polling/3.11.0) |
| `ppv-lite86` | 0.2.21 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/ppv-lite86/0.2.21) |
| `proc-macro-crate` | 3.5.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/proc-macro-crate/3.5.0) |
| `proc-macro2` | 1.0.107 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/proc-macro2/1.0.107) |
| `qrcode` | 0.14.1 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/qrcode/0.14.1) |
| `quote` | 1.0.47 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/quote/1.0.47) |
| `r-efi` | 5.3.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | [crates.io](https://crates.io/crates/r-efi/5.3.0) |
| `r-efi` | 6.0.0 | MIT OR Apache-2.0 OR LGPL-2.1-or-later | [crates.io](https://crates.io/crates/r-efi/6.0.0) |
| `rand` | 0.8.8 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/rand/0.8.8) |
| `rand_chacha` | 0.3.1 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/rand_chacha/0.3.1) |
| `rand_core` | 0.6.4 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/rand_core/0.6.4) |
| `rustc_version` | 0.4.1 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/rustc_version/0.4.1) |
| `rustix` | 1.1.6 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/rustix/1.1.6) |
| `secret-service` | 4.0.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/secret-service/4.0.0) |
| `semver` | 1.0.28 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/semver/1.0.28) |
| `serde` | 1.0.229 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/serde/1.0.229) |
| `serde_core` | 1.0.229 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/serde_core/1.0.229) |
| `serde_derive` | 1.0.229 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/serde_derive/1.0.229) |
| `serde_json` | 1.0.154 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/serde_json/1.0.154) |
| `serde_repr` | 0.1.21 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/serde_repr/0.1.21) |
| `serde_spanned` | 0.6.9 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/serde_spanned/0.6.9) |
| `serde_spanned` | 1.1.3 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/serde_spanned/1.1.3) |
| `sha1` | 0.10.7 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/sha1/0.10.7) |
| `sha2` | 0.10.9 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/sha2/0.10.9) |
| `signal-hook-registry` | 1.4.8 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/signal-hook-registry/1.4.8) |
| `slab` | 0.4.12 | MIT | [crates.io](https://crates.io/crates/slab/0.4.12) |
| `smallvec` | 1.16.5 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/smallvec/1.16.5) |
| `static_assertions` | 1.1.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/static_assertions/1.1.0) |
| `subtle` | 2.6.1 | BSD-3-Clause | [crates.io](https://crates.io/crates/subtle/2.6.1) |
| `syn` | 2.0.119 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/syn/2.0.119) |
| `syn` | 3.0.10 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/syn/3.0.10) |
| `system-deps` | 7.0.8 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/system-deps/7.0.8) |
| `target-lexicon` | 0.13.5 | Apache-2.0 WITH LLVM-exception | [crates.io](https://crates.io/crates/target-lexicon/0.13.5) |
| `tempfile` | 3.27.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/tempfile/3.27.0) |
| `toml` | 0.8.23 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/toml/0.8.23) |
| `toml` | 1.2.1+spec-1.1.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/toml/1.2.1+spec-1.1.0) |
| `toml_datetime` | 0.6.11 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/toml_datetime/0.6.11) |
| `toml_datetime` | 1.1.3+spec-1.1.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/toml_datetime/1.1.3+spec-1.1.0) |
| `toml_edit` | 0.22.27 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/toml_edit/0.22.27) |
| `toml_edit` | 0.25.19+spec-1.1.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/toml_edit/0.25.19+spec-1.1.0) |
| `toml_parser` | 1.1.5+spec-1.1.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/toml_parser/1.1.5+spec-1.1.0) |
| `toml_write` | 0.1.2 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/toml_write/0.1.2) |
| `toml_writer` | 1.2.0+spec-1.1.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/toml_writer/1.2.0+spec-1.1.0) |
| `tracing` | 0.1.44 | MIT | [crates.io](https://crates.io/crates/tracing/0.1.44) |
| `tracing-attributes` | 0.1.31 | MIT | [crates.io](https://crates.io/crates/tracing-attributes/0.1.31) |
| `tracing-core` | 0.1.36 | MIT | [crates.io](https://crates.io/crates/tracing-core/0.1.36) |
| `typenum` | 1.20.1 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/typenum/1.20.1) |
| `uds_windows` | 1.2.1 | MIT | [crates.io](https://crates.io/crates/uds_windows/1.2.1) |
| `unicode-ident` | 1.0.27 | (MIT OR Apache-2.0) AND Unicode-3.0 | [crates.io](https://crates.io/crates/unicode-ident/1.0.27) |
| `version-compare` | 0.2.1 | MIT | [crates.io](https://crates.io/crates/version-compare/0.2.1) |
| `version_check` | 0.9.5 | MIT/Apache-2.0 | [crates.io](https://crates.io/crates/version_check/0.9.5) |
| `wasi` | 0.11.1+wasi-snapshot-preview1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/wasi/0.11.1+wasi-snapshot-preview1) |
| `wasip2` | 1.0.4+wasi-0.2.12 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/wasip2/1.0.4+wasi-0.2.12) |
| `windows-link` | 0.2.1 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/windows-link/0.2.1) |
| `windows-sys` | 0.52.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/windows-sys/0.52.0) |
| `windows-sys` | 0.59.0 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/windows-sys/0.59.0) |
| `windows-sys` | 0.61.2 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/windows-sys/0.61.2) |
| `windows-targets` | 0.52.6 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/windows-targets/0.52.6) |
| `windows_aarch64_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/windows_aarch64_gnullvm/0.52.6) |
| `windows_aarch64_msvc` | 0.52.6 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/windows_aarch64_msvc/0.52.6) |
| `windows_i686_gnu` | 0.52.6 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/windows_i686_gnu/0.52.6) |
| `windows_i686_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/windows_i686_gnullvm/0.52.6) |
| `windows_i686_msvc` | 0.52.6 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/windows_i686_msvc/0.52.6) |
| `windows_x86_64_gnu` | 0.52.6 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/windows_x86_64_gnu/0.52.6) |
| `windows_x86_64_gnullvm` | 0.52.6 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/windows_x86_64_gnullvm/0.52.6) |
| `windows_x86_64_msvc` | 0.52.6 | MIT OR Apache-2.0 | [crates.io](https://crates.io/crates/windows_x86_64_msvc/0.52.6) |
| `winnow` | 0.7.15 | MIT | [crates.io](https://crates.io/crates/winnow/0.7.15) |
| `winnow` | 1.0.5 | MIT | [crates.io](https://crates.io/crates/winnow/1.0.5) |
| `wit-bindgen` | 0.57.1 | Apache-2.0 WITH LLVM-exception OR Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/wit-bindgen/0.57.1) |
| `xdg-home` | 1.3.0 | MIT | [crates.io](https://crates.io/crates/xdg-home/1.3.0) |
| `zbus` | 4.4.0 | MIT | [crates.io](https://crates.io/crates/zbus/4.4.0) |
| `zbus_macros` | 4.4.0 | MIT | [crates.io](https://crates.io/crates/zbus_macros/4.4.0) |
| `zbus_names` | 3.0.0 | MIT | [crates.io](https://crates.io/crates/zbus_names/3.0.0) |
| `zerocopy` | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/zerocopy/0.8.63) |
| `zerocopy-derive` | 0.8.63 | BSD-2-Clause OR Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/zerocopy-derive/0.8.63) |
| `zeroize` | 1.9.1 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/zeroize/1.9.1) |
| `zeroize_derive` | 1.5.0 | Apache-2.0 OR MIT | [crates.io](https://crates.io/crates/zeroize_derive/1.5.0) |
| `zmij` | 1.0.23 | MIT | [crates.io](https://crates.io/crates/zmij/1.0.23) |
| `zvariant` | 4.2.0 | MIT | [crates.io](https://crates.io/crates/zvariant/4.2.0) |
| `zvariant_derive` | 4.2.0 | MIT | [crates.io](https://crates.io/crates/zvariant_derive/4.2.0) |
| `zvariant_utils` | 2.1.0 | MIT | [crates.io](https://crates.io/crates/zvariant_utils/2.1.0) |

## Native runtime components

| Component | Fedora version observed | License family | Source |
|---|---:|---|---|
| GTK4 | 4.24.1 | LGPL-2.1-or-later; RPM also records licenses for bundled components | [upstream](https://gitlab.gnome.org/GNOME/gtk) |
| libadwaita | 1.10.0 | LGPL-2.1-or-later / MIT | [upstream](https://gitlab.gnome.org/GNOME/libadwaita) |
| DBus | 1.16.2 | AFL-2.1 or GPL-2.0-or-later | [upstream](https://gitlab.freedesktop.org/dbus/dbus) |
| Pango | 1.58.2 | LGPL-2.0-or-later | [upstream](https://gitlab.gnome.org/GNOME/pango) |
| GdkPixbuf | 2.44.8 | LGPL-2.1-or-later | [upstream](https://gitlab.gnome.org/GNOME/gdk-pixbuf) |
| Cairo | 1.18.6 | LGPL-2.1-only or MPL-1.1 | [upstream](https://gitlab.freedesktop.org/cairo/cairo) |
| Graphene | 1.10.8 | MIT | [upstream](https://github.com/ebassi/graphene) |
| TDLib | 1.8.67 used for the recorded ABI smoke probe; runtime version is not pinned | BSL-1.0 | [upstream](https://github.com/tdlib/td) |

The native libraries are system build/runtime dependencies and are not bundled. Fedora RPMs contain their full per-package licenses and bundled-component notices. TDLib 1.8.67 is listed only as the earlier ABI probe version; Telerust does not currently enforce a runtime TDLib version.

## Compatibility review

Cargo metadata reports permissive Rust dependency terms (MIT, Apache-2.0, BSD-2-Clause/3-Clause, ISC, Unicode-3.0, LLVM-exception, and Unlicense, including dual/multi-license expressions); no registry dependency is GPL-only. Where an expression includes an LGPL option, a permissive alternative is available and selected. GTK/libadwaita and other native libraries are dynamically linked system dependencies under LGPL-family or permissive terms. TDLib is an external runtime dependency under Boost Software License 1.0. These terms are compatible with this original client under GPL-3.0-or-later. Retain upstream notices when redistributing dependency sources/binaries; if native libraries are bundled later, meet their applicable LGPL relinking obligations.
