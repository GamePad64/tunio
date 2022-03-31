use etherparse::PacketBuilder;
use log::debug;
use std::io::{Read, Write};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tunio::config::IfaceConfig;
use tunio::platform::wintun::WinTunPlatformIfaceConfig;
use tunio::traits::DriverT;
use tunio::{DefaultDriver, DefaultInterface};

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut driver = DefaultDriver::new().unwrap();

    let interface_config = IfaceConfig::default().set_name("name".into()).set_platform(
        |config: WinTunPlatformIfaceConfig| config.set_description("description".into()),
    );

    let interface = driver.new_interface_up(interface_config);

    for _ in 1..100 {
        let builder = PacketBuilder::ipv6(
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            5,
        )
        .udp(8080, 8080);

        let mut packet = Vec::with_capacity(builder.size(0));
        builder.write(&mut packet, &[]).unwrap();

        // interface.write(&*packet).await;

        sleep(Duration::from_secs(1));
    }

    let mut buf = vec![0u8; 4096];
    // while let Ok(n) = interface.read(buf.as_mut_slice()).await {
    //     println!("{buf:x?}");
    // }

    tokio::signal::ctrl_c().await;
}
