# QuiRQ

Open-core amateur radio control platform. QuiRQ provides a unified interface for
controlling transceivers across operating systems, abstracting over multiple radio
control backends.

## Repository layout

```
quirq/
├── app/                        Flutter cross-platform UI (Android, iOS, Linux, macOS, Windows)
│   ├── lib/main.dart           Application entry point
│   ├── android/  ios/          Platform runners
│   ├── linux/  macos/  windows/
│   ├── test/                   Flutter widget tests
│   └── pubspec.yaml
│
├── lib/                        Rust library workspace
│   ├── Cargo.toml              Workspace manifest (hamlib-sys, quirq-core, quirq-ffi)
│   ├── hamlib/                 Hamlib source (git submodule → Hamlib/Hamlib)
│   │
│   ├── hamlib-sys/             Raw FFI bindings to libhamlib
│   │   ├── build.rs            Locates system libhamlib via pkg-config; runs bindgen
│   │   └── src/lib.rs          Re-exports generated bindings
│   │
│   ├── quirq-core/             Safe Rust wrapper and radio abstraction layer
│   │   ├── src/
│   │   │   ├── error.rs        Error and Result types
│   │   │   └── transceiver/
│   │   │       ├── mod.rs      Transceiver trait + all shared types
│   │   │       │               (Vfo, Mode, Level, Func, Parm, VfoOp, …)
│   │   │       └── hamlib.rs   HamlibTransceiver: implements Transceiver via libhamlib
│   │   └── tests/
│   │       └── hamlib_dummy.rs Integration tests against Hamlib's built-in dummy rig
│   │
│   └── quirq-ffi/              flutter_rust_bridge API surface (stub — in progress)
│       └── src/lib.rs
│
└── .gitmodules                 Declares lib/hamlib submodule
```

## Crates

### `hamlib-sys`

Raw, unsafe Rust bindings to [libhamlib](https://hamlib.github.io/), generated at
compile time by [bindgen](https://github.com/rust-lang/rust-bindgen). The system
library is located via `pkg-config`; install `libhamlib-dev` (Debian/Ubuntu) or
`hamlib-devel` (Fedora) before building.

The `hamlib/` submodule is present as a code reference and for potential future
build-from-source support; the bindings currently target the installed system library.

### `quirq-core`

The radio abstraction layer. Key design points:

- **`Transceiver` trait** — driver-independent interface covering frequency, mode,
  PTT, signal levels, boolean functions, split operation, CTCSS/DCS, repeater
  offsets, RIT/XIT, VFO operations, scan, antenna, and power state.
- **`Vfo` struct** — opaque handle obtained at runtime from `vfo_list()` or
  `vfo_by_name()`; callers never hard-code driver-specific VFO identifiers.
- **Capability discovery** — `supported_modes()`, `supported_levels()`,
  `supported_funcs()`, `supported_parms()`, `supported_vfo_ops()`,
  `supported_scan_ops()` return what the connected radio actually supports.
- **Human-readable names** — all enum types implement `Display`.
- **`HamlibTransceiver`** — implements `Transceiver` for any radio supported by
  libhamlib. Thread-safe initialisation via a static mutex on `rig_init`.

### `quirq-ffi`

Stub crate that will expose `quirq-core` to Flutter via
[flutter_rust_bridge](https://github.com/fzyzcjy/flutter_rust_bridge). Not yet
implemented.

## Building

```sh
# Install system dependencies (Debian/Ubuntu)
sudo apt-get install libhamlib-dev libclang-dev pkg-config

# Check the Rust workspace
cd lib && cargo check

# Run tests (parallel-safe — init mutex serialises rig_init internally)
cd lib && cargo test
```

## App

The Flutter application lives in `app/`. It is not yet wired to the Rust library.

```sh
cd app && flutter run -d linux
```
