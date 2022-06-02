# Tunio
[![Crates.io](https://img.shields.io/crates/v/tunio?style=flat-square)](https://crates.io/crates/tunio)
[![docs.rs](https://img.shields.io/docsrs/tunio/latest?style=flat-square)](https://docs.rs/tunio)

Create TUN/TAP interfaces in cross-platform and idiomatic Rust!

## Features ⭐
- [Tokio](https://tokio.rs/) support (optional).
- TUN/TAP support.
- Extensible architecture for adding other platforms later.

## Short example 📜
```rust,no_run
use std::io::{Read, Write};
use tunio::traits::{DriverT, InterfaceT};
use tunio::{DefaultDriver, DefaultInterface};

fn main() {
    // DefaultDriver is an alias for a supported driver for current platform.
    // It may be not optimal for your needs (for example, it can lack support of TAP),
    // but it will work in some cases. If you need another driver, then import and use it instead.
    let mut driver = DefaultDriver::new().unwrap();
    // Preparing configuration for new interface. We use `Builder` pattern for this.
    let if_config = DefaultDriver::if_config_builder()
        .name("iface1".to_string())
        .build()
        .unwrap();

    // Then, we create the interface using config and start it immediately.
    let mut interface = DefaultInterface::new_up(&mut driver, if_config).unwrap();

    // The interface is created and running.

    // Write to interface using Write trait
    let buf = [0u8; 4096];
    let _ = interface.write(&buf);

    // Read from interface using Read trait
    let mut mut_buf = [0u8; 4096];
    let _ = interface.read(&mut mut_buf);
}

```

## Supported platforms 🖥️
- **Windows**, TUN only (using [`Wintun`] driver).
  - [`Wintun`] driver requires a prebuilt DLL inside application folder. Please, refer to [`Wintun`] documentation for more details.
- **Linux**

[`Wintun`]: https://www.wintun.net/

macOS support for utun and feth drivers is planned. Feel free to post a PR, it is always greatly appreciated 😉

## Related projects 🔗
- [`netconfig`]: A high-level abstraction for gathering and changing network interface configuration.

[`netconfig`]: https://github.com/GamePad64/netconfig
