#[allow(dead_code)]
#[allow(non_snake_case)]
#[allow(unused_variables)]
#[allow(unused_imports)]
#[allow(unreachable_patterns)]
mod toio;
mod scanner;

use scanner::ToioScanner;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let scanner = ToioScanner::new().await?;
    let mut toios = scanner.search().await?;

    let mut toio_list : Vec<toio::Toio> = vec![];


    loop {
        if let Some(toio) = toios.next() {
            toio.connect().await;
    
            toio.send_command(toio::Command::Led {
                length: 0,
                red: 255,
                green: 255,
                blue: 255,
            })
            .await;
    
            toio.send_command(toio::Command::LedRepeat {
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
    
            toio.send_command(toio::Command::MotorAcceleration {
                velocity: 50,
                acceleration: 5,
                rotationalVelocity: 15,
                rotationalDirection: 0,
                direction: 0,
                priority: 0,
                duration: 255,
            })
            .await;

            toio_list.push(toio);
        }
    }

    Ok(())
}