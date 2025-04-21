mod osc;
mod toio;
mod ui;

use osc::*;
use toio::*;
use ui::*;

use std::error::Error;
use std::net::UdpSocket;
use std::process;
use std::sync::Arc;
use std::time::SystemTime;
use std::vec;

use futures::future::join_all;
use tokio::sync::RwLock;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let scanner = ToioScanner::new().await?;
    let mut toios = scanner.search().await?;
    let connected: Arc<RwLock<Vec<ToioUI>>> = Arc::new(RwLock::new(vec![]));

    let host_addr = "0.0.0.0:3334";
    let to_addr = "0.0.0.0:3333";

    let socket = Arc::new(UdpSocket::bind(host_addr).unwrap());
    let mut buf = [0u8; rosc::decoder::MTU];

    let sock = socket.clone();
    let connected_clone = connected.clone();
    tokio::spawn(async move {
        // let mut now = SystemTime::now();
        while let Ok(size) = sock.recv(&mut buf) {
            let (_, packet) = rosc::decoder::decode_udp(&buf[..size]).unwrap();
            if let Some((toionum, cmd)) = handle_packet(packet) {
                let connected_read = connected_clone.read().await;
                if toionum < connected_read.len() {
                    let last_command = connected_read[toionum].get_last_command();
                    let mut last_command_write = last_command.write().await;
                    *last_command_write = Some(SystemTime::now());

                    connected_read[toionum].toio.send_command(cmd).await;
                }
            }
        }
    });

    let connected_clone = connected.clone();
    tokio::spawn(async move {
        while let Some(toio) = toios.next().await {
            toio.connect().await;

            let mut updates = toio.updates().await.unwrap();

            let sock = socket.clone();

            let mut connected_write = connected_clone.write().await;
            let toio_ui = ToioUI::new(toio);

            let id = connected_write.len();
            let battery = toio_ui.get_battery();
            let last_update = toio_ui.get_last_update();
            tokio::spawn(async move {
                while let Some(update) = updates.next().await {
                    if let Update::Battery { level } = update {
                        let mut battery = battery.write().await;
                        *battery = Some(level);
                    }

                    let mut last_update = last_update.write().await;
                    *last_update = Some(SystemTime::now());

                    send_packet(&sock, to_addr, id, update);
                }
            });

            connected_write.push(toio_ui);
        }
    });

    let mut terminal = setup_terminal()?;

    let connected_clone = connected.clone();
    loop {
        let connected_read = connected_clone.read().await;

        let toio_info = join_all(connected_read.iter().map(|toio| async {
            let name = toio.name.clone();
            let id = toio.id.clone();
            let connected = toio.is_connected().await;

            let battery_string = if let Some(level) = *toio.battery.read().await {
                format!("{}", level)
            } else {
                "N/A".to_string()
            };

            let last_update_string = if let Some(last) = *toio.last_update.read().await {
                if let Ok(time) = last.elapsed() {
                    if time.as_millis() < 50 {
                        "<50ms".to_string()
                    } else if time.as_secs() < 1 {
                        format!(">{}ms", time.as_millis() - (time.as_millis() % 100))
                    } else {
                        format!("{}s", time.as_secs())
                    }
                } else {
                    "N/A".to_string()
                }
            } else {
                "N/A".to_string()
            };

            let last_command_string = if let Some(last) = *toio.last_command.read().await {
                if let Ok(time) = last.elapsed() {
                    if time.as_millis() < 50 {
                        "<50ms".to_string()
                    } else if time.as_secs() < 1 {
                        format!(">{}ms", time.as_millis() - (time.as_millis() % 100))
                    } else {
                        format!("{}s", time.as_secs())
                    }
                } else {
                    "N/A".to_string()
                }
            } else {
                "N/A".to_string()
            };

            (
                name,
                id,
                battery_string,
                last_update_string,
                last_command_string,
                connected,
            )
        }))
        .await;

        terminal.draw(ui(toio_info))?;

        if handle_events()? {
            exit_terminal()?;
            process::exit(0);
        }
    }
}
