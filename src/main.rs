#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(unused_variables)]
#[allow(unused_imports)]
#[allow(unreachable_patterns)]
mod toio;

use btleplug::api::{Central, CentralEvent, Manager as _, Peripheral, ScanFilter};
use btleplug::platform::Manager;
use futures::stream::StreamExt;
use std::error::Error;
use std::time::Duration;
use tokio::sync::mpsc;
use tokio::time::sleep;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let manager = Manager::new().await?;

    // get the first bluetooth adapter
    // connect to the adapter
    let central = manager
        .adapters()
        .await
        .unwrap()
        .into_iter()
        .nth(0)
        .unwrap();

    // Each adapter has an event stream, we fetch via events(),
    // simplifying the type, this will return what is essentially a
    // Future<Result<Stream<Item=CentralEvent>>>.
    let mut events = central.events().await?;

    // start scanning for devices
    central
        .start_scan(ScanFilter {
            services: vec![toio::SERVICE],
        })
        .await?;

    let (tx, mut rx) = mpsc::channel(32);

    //Discovery Async Task
    tokio::spawn(async move {
        while let Some(event) = events.next().await {
            match event {
                CentralEvent::DeviceDiscovered(id) => {
                    tx.send(id).await.unwrap();
                }
                // CentralEvent::DeviceUpdated(id) => {
                //     tx.send(id).await.unwrap();
                // }
                CentralEvent::DeviceDisconnected(id) => {
                    println!("Device Disconnected! {:?}", id);
                }
                _ => {}
            }
        }
    });

    //Connection Async Task
    tokio::spawn(async move {
        while let Some(id) = rx.recv().await {
            let peripheral = central.peripheral(&id).await.unwrap();

            if let Some(properties) = peripheral.properties().await.unwrap() {
                if !properties.services.contains(&toio::SERVICE) {
                    continue;
                }

                let name = properties.local_name.unwrap_or("".to_string());

                let toio = toio::Toio::new(name, peripheral);
                toio.connect().await;
                toio.start_update_stream().await;

                // toio.send_command(toio::Command::Sound {
                //     soundEffect: 3,
                //     volume: 255,
                // })
                // .await;
                toio.send_command(toio::Command::Led {
                    length: 0,
                    red: 255,
                    green: 255,
                    blue: 255,
                })
                .await;

                toio.send_command(toio::Command::LedRepeat {
                    repetitions: 0,
                    lights: vec![
                        toio::LedCommand {
                            duration: 0x1E,
                            red: 0x00,
                            blue: 0xFF,
                            green: 0x00,
                        },
                        toio::LedCommand {
                            duration: 0x1E,
                            red: 0x00,
                            blue: 0x00,
                            green: 0xFF,
                        },
                    ],
                })
                .await;

                toio.send_command(toio::Command::Midi {
                    repetitions: 0,
                    notes: vec![
                        toio::MidiCommand {
                            duration: 0x1e,
                            note: 0x3c,
                            volume: 0x1e,
                        },
                        toio::MidiCommand {
                            duration: 0x1e,
                            note: 0x3e,
                            volume: 0xff,
                        },
                        toio::MidiCommand {
                            duration: 0x1e,
                            note: 0x40,
                            volume: 0xff,
                        },
                    ],
                })
                .await;

                toio.send_command(toio::Command::MultiTarget {
                    control: 0,
                    timeout: 5,
                    moveType: 0,
                    maxSpeed: 80,
                    speedChange: 0,
                    opAdd: 1,
                    targets: vec![
                        toio::TargetCommand {
                            xTarget: 100,
                            yTarget: 100,
                            thetaTarget: 0,
                        },
                        toio::TargetCommand {
                            xTarget: 200,
                            yTarget: 100,
                            thetaTarget: 90,
                        },
                        toio::TargetCommand {
                            xTarget: 200,
                            yTarget: 200,
                            thetaTarget: 180,
                        },
                    ],
                })
                .await;

                sleep(Duration::from_secs(2)).await;

                toio.send_command(toio::Command::SoundOff).await;
                toio.send_command(toio::Command::LedOff).await;

                // toio.send_command(toio::Command::MotorAcceleration { velocity: 50, acceleration: 5, rotationalVelocity: 15, rotationalDirection: 0, direction: 0, priority: 0, duration: 255 }).await;
                // toio.send_command(toio::Command::LedRepeat { repetitions: 0, operations: 2, lights: vec![0x1E, 0x3C, 255, 0x1E, ] }).await;
            }
        }
    });

    loop {
        // println!("{}", vec.read().unwrap().len());
    }
}
