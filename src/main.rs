use crate::tun::{WinTunDriver, WinTunInterface};
use etherparse::PacketBuilder;
use std::thread::sleep;
use std::time::Duration;

mod tun;
mod waiter;
mod wintun_raw;

#[tokio::main]
async fn main() {
    env_logger::init();
    let driver = WinTunDriver::new();
    // let interface = driver.create_iface();
    // let interface = WinTunInterface::new(driver.wintun, "name", "type");

    for _ in 1..100 {
        let builder = PacketBuilder::ipv6(
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
            5,
        )
        .udp(8080, 8080);

        let mut packet = Vec::with_capacity(builder.size(0));
        builder.write(&mut packet, &[]).unwrap();

        // interface.send(&*packet);
        sleep(Duration::from_secs(1))
    }

    tokio::signal::ctrl_c().await;
}
