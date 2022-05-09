use std::io::{Read, Write};
use tunio::traits::DriverT;
use tunio::DefaultDriver;

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
    let mut interface = driver.new_interface_up(if_config).unwrap();

    // The interface is created and running.

    // Write to interface using Write trait
    let buf = [0u8; 4096];
    let _ = interface.write(&buf);

    // Read from interface using Read trait
    let mut mut_buf = [0u8; 4096];
    let _ = interface.read(&mut mut_buf);
}
