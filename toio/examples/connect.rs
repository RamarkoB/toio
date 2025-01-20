use ::toio::*;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let scanner = ToioScanner::new().await?;
    let mut toios = scanner.search().await?;

    while let Some(toio) = toios.next().await {
        toio.connect().await;
        let mut updates = toio.updates().await?;

        tokio::spawn(async move {
            while let Some(update) = updates.next().await {
                match update {
                    Update::Position { .. } => {}
                    _ => println!("{:?}", update),
                }
            }
        });

        toio.send_command(toio::Command::Led {
            duration: 0,
            red: 255,
            green: 255,
            blue: 255,
        })
        .await;

        toio.send_command(toio::Command::MultiLed {
            repetitions: 2,
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
            repetitions: 2,
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
            move_type: 0,
            max_speed: 80,
            speed_change: 0,
            op_add: 1,
            targets: vec![
                toio::TargetCommand {
                    x_target: 100,
                    y_target: 100,
                    theta_target: 0,
                },
                toio::TargetCommand {
                    x_target: 200,
                    y_target: 100,
                    theta_target: 90,
                },
                toio::TargetCommand {
                    x_target: 200,
                    y_target: 200,
                    theta_target: 180,
                },
            ],
        })
        .await;

        // toio.send_command(toio::Command::MotorAcceleration {
        //     velocity: 50,
        //     acceleration: 5,
        //     rotational_velocity: 15,
        //     rotational_direction: 0,
        //     direction: 0,
        //     priority: 0,
        //     duration: 255,
        // })
        // .await;
    }

    Ok(())
}
