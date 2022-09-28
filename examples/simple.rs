use etherparse::PacketBuilder;
use futures::AsyncReadExt;
use futures::AsyncWriteExt;
use std::thread::sleep;
use std::time::Duration;
use tunio::traits::{DriverT, InterfaceT};
use tunio::{DefaultAsyncInterface, DefaultDriver};

#[tokio::main]
async fn main() {
    env_logger::init();
    let mut driver = DefaultDriver::new().unwrap();

    let mut interface_config = DefaultAsyncInterface::config_builder();
    interface_config.name("name".into());
    #[cfg(target_os = "windows")]
    interface_config
        .platform(|mut b| b.description("description".into()).build())
        .unwrap();
    let interface_config = interface_config.build().unwrap();

    let mut interface = DefaultAsyncInterface::new_up(&mut driver, interface_config).unwrap();
    let iff = interface.handle();

    iff.add_address("18.3.5.6/24".parse().unwrap()).unwrap();
    iff.add_address("20.3.5.6/24".parse().unwrap()).unwrap();
    iff.remove_address("18.3.5.6/24".parse().unwrap()).unwrap();
    iff.add_address("fd3c:dea:7f96:2b14::/64".parse().unwrap())
        .unwrap();

    for _ in 1..10 {
        let builder = PacketBuilder::ipv6(
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            5,
        )
        .udp(8080, 8080);

        let mut packet = Vec::with_capacity(builder.size(0));
        builder.write(&mut packet, &[]).unwrap();

        interface.write(&*packet).await;

        sleep(Duration::from_secs(1));
    }

    let mut buf = vec![0u8; 4096];
    while let Ok(n) = interface.read(buf.as_mut_slice()).await {
        buf.truncate(n);
        println!("{buf:x?}");
        buf.resize(4096, 0u8);
    }

    tokio::signal::ctrl_c().await;
}
