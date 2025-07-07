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

use clap::Parser;
use futures::future::join_all;
use futures::future::Either::{Left, Right};
use tokio::sync::RwLock;

#[derive(Parser)]
#[command(name = "toio")]
struct Args {
    /// Set receiving port
    #[arg(short, long)]
    port: Option<usize>,

    /// Set remote port
    #[arg(short, long)]
    remote: Option<usize>,

    /// Show terminal UI
    #[arg(short, long)]
    terminal: bool,

    /// Use unfiltered search
    #[arg(short, long)]
    search: bool,

    /// Use ordered search
    #[arg(short, long)]
    ordered: bool,

    /// Filter toios by IDs (comma-separated list e.g. 1,2,3)
    #[arg(short, long, value_delimiter = ',')]
    axlab_id: Option<Vec<usize>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();

    // create scanner and array of toios
    let scanner = match args.axlab_id.clone() {
        Some(filter) => {
            if args.terminal {
                if args.ordered {
                    println!("Running Ordered Search for Toios: {:?}", filter.clone());
                } else {
                    println!("Running Unordered Search for Toios: {:?}", filter.clone());
                }
            }
            ToioScanner::new_with_filter(args.ordered, filter.clone()).await?
        }
        None => {
            if !args.search {
                println!("You must provide IDs when not in search mode");
                process::exit(0);
            }

            if args.terminal {
                println!("Running Unfiltered Search for Toios");
            }
            ToioScanner::new().await?
        }
    };
    // let scanner = ToioScanner::new_with_filter(true, vec![3, 100]).await?;
    let mut toios = scanner.search().await?;
    let connected: Arc<RwLock<Vec<Arc<RwLock<Toio>>>>> = Arc::new(RwLock::new(vec![]));

    // server and client address
    let host_addr = format!("0.0.0.0:{}", args.port.unwrap_or(3334));
    let to_addr = format!("0.0.0.0:{}", args.remote.unwrap_or(3333));
    if args.terminal {
        println!(
            "Listening on port {} and sending to port {}",
            host_addr, to_addr
        )
    }

    // open socket and create buffer
    let socket = Arc::new(UdpSocket::bind(&host_addr)?);
    let mut buf = [0u8; rosc::decoder::MTU];

    // whenever a message is recieved through OSC, forward to toio
    let sock = socket.clone();
    let connected_clone = connected.clone();
    tokio::spawn(async move {
        // let mut now = SystemTime::now();
        while let Ok(size) = sock.recv(&mut buf) {
            if let Ok((_, packet)) = rosc::decoder::decode_udp(&buf[..size]) {
                if let Some((toionum, cmd)) = handle_packet(packet) {
                    let connected_read = connected_clone.read().await;
                    if toionum < connected_read.len() {
                        let toio = connected_read[toionum].read().await;

                        let last_command = toio.get_last_command();
                        let mut last_command_write = last_command.write().await;
                        *last_command_write = Some(SystemTime::now());

                        toio.toio.send_command(cmd).await;
                    }
                }
            }
        }
    });

    // whenever we connect to a toio, add it to the list
    let connected_clone = connected.clone();
    tokio::spawn(async move {
        while let Some(peripheral_update) = toios.next().await {
            match peripheral_update {
                Left(toio_peripheral) => {
                    // clone socket
                    let sock = socket.clone();

                    // listen for updates from toio
                    let mut updates = toio_peripheral.updates().await.unwrap();

                    // create instance of Toio to record toio info
                    let mut toio = Toio::new(toio_peripheral);
                    if args.terminal {
                        println!("Toio Connected: {}", toio.id);
                    }

                    let battery = toio.get_battery();
                    let last_update = toio.get_last_update();

                    // request permission to write to list of connected toios
                    let mut connected_write = connected_clone.write().await;
                    let id = connected_write.len();

                    // start process to listen for messages from toio
                    let toio_channel = tokio::spawn({
                        let to_addr = to_addr.clone();
                        async move {
                            while let Some(update) = updates.next().await {
                                // if it is a battery update, record it in the Toio
                                if let Update::Battery { level } = update {
                                    let mut battery = battery.write().await;
                                    *battery = Some(level);
                                }

                                // record time of update
                                let mut last_update = last_update.write().await;
                                *last_update = Some(SystemTime::now());

                                send_packet(&sock, &to_addr, id, update);
                            }
                        }
                    });

                    toio.add_channel(toio_channel);
                    connected_write.push(Arc::new(RwLock::new(toio)));
                }
                Right(peripheral_id) => {
                    // request permission to write to list of connected toios
                    let connected_write = connected_clone.write().await;

                    // Collect all peripheral_ids with their indices
                    let ids = join_all(connected_write.iter().map(|x| async {
                        let toio = x.read().await;
                        toio.toio.peripheral_id.clone()
                    }))
                    .await;

                    let toio_id = ids.iter().position(|id| *id == peripheral_id);

                    // Find the index of the matching peripheral_id
                    if let Some(idx) = toio_id {
                        let mut toio = connected_write[idx].write().await;
                        toio.disconnect();
                        if args.terminal {
                            println!("Toio Disconnected: {}", toio.id);
                        }
                    };
                }
            }
        }
    });

    // // start TUI process
    let mut terminal: ToioUI = None;
    if !args.terminal {
        terminal = setup_terminal()?;
    }

    // update UI from all of the toios
    let connected_clone = connected.clone();
    loop {
        let connected_read = connected_clone.read().await;

        // get info from all of the toios
        let toio_info = join_all(connected_read.iter().map(|toio_guard| async {
            let toio = toio_guard.read().await;
            let name = toio.name.clone();
            let id = toio.id.clone();
            let connected = toio.is_connected().await;

            // get battery level
            let battery_string = if let Some(level) = *toio.battery.read().await {
                format!("{}", level)
            } else {
                "N/A".to_string()
            };

            // get time of last update
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

            // get time of last command
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

        //  update UI
        if let Some(ref mut toio_ui) = terminal {
            toio_ui.draw(ui(toio_info, args.axlab_id.clone()))?;
        }

        // // exit terminal if "Q" key is pressed
        if handle_events()? {
            exit_terminal()?;
            process::exit(0);
        }
    }
}
