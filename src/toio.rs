#[allow(dead_code)]
#[allow(unused_variables)]
#[allow(unused_imports)]
#[allow(unreachable_patterns)]
use btleplug::{
    api::{CharPropFlags, Characteristic, Peripheral, ValueNotification, WriteType},
    platform,
};
use tokio::sync::mpsc;
use tokio::sync::mpsc::Receiver;
use futures::stream::StreamExt;
use std::time::SystemTime;
use uuid::Uuid;
use std::error::Error;

use tokio::time::timeout;

pub const SERVICE: Uuid = Uuid::from_u128(0x10B20100_5B3B_4571_9508_CF3EFCD7BBAE);
pub const POSITION: Uuid = Uuid::from_u128(0x10B20101_5B3B_4571_9508_CF3EFCD7BBAE);
pub const MOTOR: Uuid = Uuid::from_u128(0x10B20102_5B3B_4571_9508_CF3EFCD7BBAE);
pub const LIGHT: Uuid = Uuid::from_u128(0x10B20103_5B3B_4571_9508_CF3EFCD7BBAE);
pub const SOUND: Uuid = Uuid::from_u128(0x10B20104_5B3B_4571_9508_CF3EFCD7BBAE);
pub const MOTION: Uuid = Uuid::from_u128(0x10B20106_5B3B_4571_9508_CF3EFCD7BBAE);
pub const BUTTON: Uuid = Uuid::from_u128(0x10B20107_5B3B_4571_9508_CF3EFCD7BBAE);
pub const BATTERY: Uuid = Uuid::from_u128(0x10B20108_5B3B_4571_9508_CF3EFCD7BBAE);
// pub const CONFIG: Uuid = Uuid::from_u128(0x10B201FF_5B3B_4571_9508_CF3EFCD7BBAE);

pub fn uuid_to_string(uuid: Uuid) -> String {
    return match uuid {
        SERVICE => "Service",
        POSITION => "Position",
        MOTOR => "Motor",
        LIGHT => "Light",
        SOUND => "Sound",
        MOTION => "Motion",
        BUTTON => "Button",
        BATTERY => "Battery",
        CONFIG => "Config",
        _ => "",
    }
    .to_owned();
}

/// Format for a target to plug into the MotorTarget varient 
/// of the Command enum. By putting multiple of these into a vector,
/// you can send a a series of targets for a toio to travel to in
/// a sequence.
#[derive(Clone)]
pub struct TargetCommand {
    pub xTarget: u16,
    pub yTarget: u16,
    pub thetaTarget: u16,
}

/// Format for a RGB color to plug into the LedRepeat varient 
/// of the Command enum. By putting multiple of these into a vector,
/// you can send a a series of colors for a toio to flash on its led
/// in a sequence.
#[derive(Clone)]
pub struct LedCommand {
    pub duration: u8,
    pub red: u8,
    pub blue: u8,
    pub green: u8,
}

/// Format for a MIDI note to plug into the Midi varient 
/// of the Command enum. By putting multiple of these into a vector,
/// you can send a a series of notes for a toio to play  in
/// a sequence.
#[derive(Clone)]
pub struct MidiCommand {
    pub duration: u8,
    pub note: u8,
    pub volume: u8,
}

#[derive(Clone)]
pub enum Command {
    //Request Commands
    MotionRequest,
    MagneticRequest,
    PostureRequest {
        format: u8,
    },

    //Motor Commands
    MotorControl {
        leftDirection: u8,
        leftSpeed: u8,
        rightDirection: u8,
        rightSpeed: u8,
    },
    MotorDuration {
        leftDirection: u8,
        leftSpeed: u8,
        rightDirection: u8,
        rightSpeed: u8,
        duration: u8,
    },
    MotorTarget {
        control: u8,
        timeout: u8,
        moveType: u8,
        maxSpeed: u8,
        speedChange: u8,
        xTarget: u16,
        yTarget: u16,
        thetaTarget: u16,
    },
    MultiTarget {
        control: u8,
        timeout: u8,
        moveType: u8,
        maxSpeed: u8,
        speedChange: u8,
        opAdd: u8,
        targets: Vec<TargetCommand>,
    },
    MotorAcceleration {
        velocity: u8,
        acceleration: u8,
        rotationalVelocity: u16,
        rotationalDirection: u8,
        direction: u8,
        priority: u8,
        duration: u8,
    },

    //Light Commands
    LedOff,
    Led {
        length: u8,
        red: u8,
        green: u8,
        blue: u8,
    },
    LedRepeat {
        repetitions: u8,
        lights: Vec<LedCommand>,
    },

    //Sound Commands
    SoundOff,
    Sound {
        soundEffect: u8,
        volume: u8,
    },
    Midi {
        repetitions: u8,
        notes: Vec<MidiCommand>,
    },
}

fn parseTargetCommand(vals: Vec<TargetCommand>) -> Vec<u8> {
    let mut cmd = vec![];

    for target in vals.iter() {
        cmd.push((target.xTarget & 0x00FF) as u8);
        cmd.push(((target.xTarget & 0xFF00) >> 8) as u8);
        cmd.push((target.yTarget & 0x00FF) as u8);
        cmd.push(((target.yTarget & 0xFF00) >> 8) as u8);
        cmd.push((target.thetaTarget & 0x00FF) as u8);
        cmd.push(((target.thetaTarget & 0xFF00) >> 8) as u8);
    }

    return cmd;
}

fn parseLedCommand(repetitions: u8, vals: Vec<LedCommand>) -> Vec<u8> {
    let mut cmd = vec![0x04, repetitions, vals.len() as u8];

    for led in vals.iter() {
        cmd.push(led.duration);
        cmd.push(0x01);
        cmd.push(0x01);
        cmd.push(led.red);
        cmd.push(led.green);
        cmd.push(led.blue);
    }

    return cmd;
}

fn parseMidiCommand(repetitions: u8, vals: Vec<MidiCommand>) -> Vec<u8> {
    let mut cmd = vec![0x03, repetitions, vals.len() as u8];

    for note in vals.iter() {
        cmd.push(note.duration);
        cmd.push(note.note);
        cmd.push(note.volume);
    }

    return cmd;
}

#[derive(Debug)]
pub enum Update {
    Position {
        xCenter: u16,
        yCenter: u16,
        theta: u16,
        xSensor: u16,
        ySensor: u16,
    },
    Standard {
        standard: u32,
        theta: u16,
    },
    PositionMissed,
    StandardMissed,
    MotorTargetResponse {
        control: u8,
        response: u8,
    },
    MultiTargetResponse {
        control: u8,
        response: u8,
    },
    MotorSpeed {
        leftSpeed: u8,
        rightSpeed: u8,
    },
    Motion {
        horizontal: u8,
        collision: u8,
        double_tap: u8,
        posture: u8,
        shake: u8,
    },
    PostureEuler {
        roll: u16,
        pitch: u16,
        yaw: u16,
    },
    PostureQuaternion {
        w: f32,
        x: f32,
        y: f32,
        z: f32,
    },
    PostureHighPrecisionEuler {
        roll: f32,
        pitch: f32,
        yaw: f32,
    },
    Magnetic {
        state: u8,
        strength: u8,
        forcex: i8,
        forcey: i8,
        forcez: i8,
    },
    Button {
        pressed: bool,
    },
    Battery {
        level: u8,
    },
}

pub struct Toio {
    name: String,
    peripheral: platform::Peripheral,
    lastUpdate: Option<SystemTime>,
}

impl Toio {
    pub fn new(name: String, peripheral: platform::Peripheral) -> Toio {
        Toio {
            name,
            peripheral,
            lastUpdate: None,
        }
    }

    pub async fn connect(&self) {
        if let Err(err) = self.peripheral.connect().await {
            eprintln!("Error connecting to {}", err);
        } else {
            println!("Connected to {}", self.name);
        }

        self.peripheral.discover_services().await.unwrap();

        for characteristic in self.peripheral.characteristics().into_iter() {
            if !characteristic.properties.contains(CharPropFlags::NOTIFY) {
                continue;
            }

            if let Err(err) = self.peripheral.subscribe(&characteristic).await {
                eprintln!("Error connecting to characteristic, skipping: {}", err);
                continue;
            } else {
                // println!(
                //     "Connected to {} characteristic",
                //     uuid_to_string(characteristic.uuid)
                // );
            }
        }
    }

    pub async fn updates(&self) ->  Result<Receiver<Update>, Box<dyn Error>> {
        let (tx, rx) = mpsc::channel(32);

        let mut notification_stream = self.peripheral.notifications().await?;
        tokio::spawn(async move {
            // let notification_steam = notification_stream;

            while let Ok(possible_event) = timeout(
                std::time::Duration::from_secs(5),
                notification_stream.next(),
            )
            .await
            {
                if let Some(event) = possible_event {
                    if let Some(update) = Toio::get_update(event) {
                        tx.send(update).await.unwrap();
                    }
                }
            }
        });

        return Ok(rx);
    }

    fn get_update(notification: ValueNotification) -> Option<Update> {
        let vals = notification.value;
        match notification.uuid {
            POSITION => match vals[0] {
                0x01 => Some(Update::Position {
                    xCenter: vals[1] as u16 | (vals[2] as u16) << 8,
                    yCenter: vals[3] as u16 | (vals[4] as u16) << 8,
                    theta: vals[5] as u16 | (vals[6] as u16) << 8,
                    xSensor: vals[7] as u16 | (vals[8] as u16) << 8,
                    ySensor: vals[9] as u16 | (vals[10] as u16) << 8,
                }),
                0x02 => Some(Update::Standard {
                    standard: vals[1] as u32
                        | (vals[2] as u32) << 8
                        | (vals[3] as u32) << 16
                        | (vals[4] as u32) << 24,
                    theta: vals[5] as u16 | (vals[6] as u16) << 8,
                }),
                0x03 => Some(Update::PositionMissed),
                0x04 => Some(Update::StandardMissed),
                _ => {
                    println!(
                        "Unkown {} Update: {:?}",
                        uuid_to_string(notification.uuid),
                        vals
                    );
                    None
                }
            },
            MOTOR => match vals[0] {
                0x83 => Some(Update::MotorTargetResponse {
                    control: vals[1],
                    response: vals[2],
                }),
                0x84 => Some(Update::MultiTargetResponse {
                    control: vals[1],
                    response: vals[2],
                }),
                0xe0 => Some(Update::MotorSpeed {
                    leftSpeed: vals[1],
                    rightSpeed: vals[2],
                }),
                _ => {
                    println!(
                        "Unkown {} Update: {:?}",
                        uuid_to_string(notification.uuid),
                        vals
                    );
                    None
                }
            },
            MOTION => match vals[0] {
                0x01 => Some(Update::Motion {
                    horizontal: vals[1],
                    collision: vals[2],
                    double_tap: vals[3],
                    posture: vals[4],
                    shake: vals[5],
                }),
                // 0x02 => Some(Update::Magnetic {
                //     state: vals[1],
                //     strength: vals[2],
                //     forcex: vals[3] as i8,
                //     forcey: vals[4] as i8,
                //     forcez: vals[5] as i8,
                // }),
                // 0x03 => match vals[1] {
                //     0x01 => Some(Update::PostureEuler {
                //         roll: vals[2] as u16 | (vals[3] as u16) << 8,
                //         pitch: vals[4] as u16 | (vals[5] as u16) << 8,
                //         yaw: vals[5] as u16 | (vals[6] as u16) << 8,
                //     }),
                //     0x02 => Some(Update::PostureQuaternion {
                //         w: 0.0,
                //         // vals[2] as f32
                //         //     | (vals[3] as f32) << 8
                //         //     | (vals[4] as f32) << 16
                //         //     | (vals[5] as f32) << 24,
                //         x: 0.0,
                //         // vals[6] as f32
                //         //     | (vals[7] as f32) << 8
                //         //     | (vals[8] as f32) << 16
                //         //     | (vals[9] as f32) << 24,
                //         y: 0.0,
                //         // vals[10] as f32
                //         //     | (vals[11] as f32) << 8
                //         //     | (vals[12] as f32) << 16
                //         //     | (vals[13] as f32) << 24,
                //         z: 0.0,
                //         // vals[14] as f32
                //         //     | (vals[15] as f32) << 8
                //         //     | (vals[16] as f32) << 16
                //         //     | (vals[17] as f32) << 24,
                //     }),
                //     0x03 => Some(Update::PostureHighPrecisionEuler {
                //         roll: 0.0,
                //         // vals[2] as f32
                //         //     | (vals[3] as f32) << 8
                //         //     | (vals[4] as f32) << 16
                //         //     | (vals[5] as f32) << 24,
                //         pitch: 0.0,
                //         // vals[6] as f32
                //         //     | (vals[7] as f32) << 8
                //         //     | (vals[8] as f32) << 16
                //         //     | (vals[9] as f32) << 24,
                //         yaw: 0.0,
                //         // vals[10] as f32
                //         //     | (vals[11] as f32) << 8
                //         //     | (vals[12] as f32) << 16
                //         //     | (vals[13] as f32) << 24,
                //     }),
                //     _ => {
                //         println!(
                //             "Unkown {} Update: {:?}",
                //             uuid_to_string(notification.uuid),
                //             vals
                //         );
                //         None
                //     }
                // },
                _ => {
                    println!(
                        "Unkown {} Update: {:?}",
                        uuid_to_string(notification.uuid),
                        vals
                    );
                    None
                }
            },
            BATTERY => Some(Update::Battery { level: vals[0] }),
            BUTTON => Some(Update::Button {
                pressed: vals[1] == 0x80,
            }),
            _ => {
                println!(
                    "Unkown {} Update: {:?}",
                    uuid_to_string(notification.uuid),
                    vals
                );
                None
            }
        }
    }

    pub async fn send_command(&self, command: Command) {
        let uuid = match command {
            Command::MotionRequest | Command::MagneticRequest | Command::PostureRequest { .. } => {
                MOTION
            }
            Command::MotorControl { .. }
            | Command::MotorDuration { .. }
            | Command::MotorTarget { .. }
            | Command::MultiTarget { .. }
            | Command::MotorAcceleration { .. } => MOTOR,
            Command::LedOff | Command::Led { .. } | Command::LedRepeat { .. } => LIGHT,
            Command::SoundOff | Command::Sound { .. } | Command::Midi { .. } => SOUND,
        };

        let (responseFlag, responseType) = match uuid {
            LIGHT | SOUND => (CharPropFlags::WRITE, WriteType::WithResponse),
            _ => (
                CharPropFlags::WRITE_WITHOUT_RESPONSE,
                WriteType::WithoutResponse,
            ),
        };

        let cmd: Vec<u8> = match command {
            Command::MotionRequest => {
                vec![0x81]
            }
            Command::MagneticRequest => {
                vec![0x82]
            }
            Command::PostureRequest { format } => {
                vec![0x83, format]
            }
            Command::MotorControl {
                leftDirection,
                leftSpeed,
                rightDirection,
                rightSpeed,
            } => {
                vec![
                    0x01,
                    0x01,
                    leftDirection,
                    leftSpeed,
                    0x02,
                    rightDirection,
                    rightSpeed,
                ]
            }
            Command::MotorDuration {
                leftDirection,
                leftSpeed,
                rightDirection,
                rightSpeed,
                duration,
            } => {
                vec![
                    0x02,
                    0x01,
                    leftDirection,
                    leftSpeed,
                    0x02,
                    rightDirection,
                    rightSpeed,
                    duration,
                ]
            }
            Command::MotorTarget {
                control,
                timeout,
                moveType,
                maxSpeed,
                speedChange,
                xTarget,
                yTarget,
                thetaTarget,
            } => {
                vec![
                    0x03,
                    control,
                    timeout,
                    moveType,
                    maxSpeed,
                    speedChange,
                    0x00,
                    (xTarget & 0x00FF) as u8,
                    ((xTarget & 0xFF00) >> 8) as u8,
                    (yTarget & 0x00FF) as u8,
                    ((yTarget & 0xFF00) >> 8) as u8,
                    (thetaTarget & 0x00FF) as u8,
                    ((thetaTarget & 0xFF00) >> 8) as u8,
                ]
            }
            Command::MultiTarget {
                control,
                timeout,
                moveType,
                maxSpeed,
                speedChange,
                opAdd,
                targets,
            } => {
                let mut cmd = vec![
                    0x04,
                    control,
                    timeout,
                    moveType,
                    maxSpeed,
                    speedChange,
                    0x00,
                    opAdd,
                ];
                cmd.append(&mut parseTargetCommand(targets));
                cmd
            }

            Command::MotorAcceleration {
                velocity,
                acceleration,
                rotationalVelocity,
                rotationalDirection,
                direction,
                priority,
                duration,
            } => {
                vec![
                    0x05,
                    velocity,
                    acceleration,
                    (rotationalVelocity & 0x00FF) as u8,
                    ((rotationalVelocity & 0xFF00) >> 8) as u8,
                    rotationalDirection,
                    direction,
                    priority,
                    duration,
                ]
            }
            Command::LedOff => {
                vec![0x01]
            }
            Command::Led {
                length,
                red,
                green,
                blue,
            } => {
                vec![0x03, length, 0x01, 0x01, red, blue, green]
            }
            Command::LedRepeat {
                repetitions,
                lights,
            } => parseLedCommand(repetitions, lights),
            Command::SoundOff => {
                vec![0x01]
            }
            Command::Sound {
                soundEffect,
                volume,
            } => {
                vec![
                    0x02,        //sound
                    soundEffect, //sound effect ID
                    volume,      //volume
                ]
            }
            Command::Midi { repetitions, notes } => parseMidiCommand(repetitions, notes),
            _ => {
                vec![]
            }
        };

        self.write(uuid, cmd, responseFlag, responseType).await;
    }

    pub async fn write(
        &self,
        uuid: Uuid,
        cmd: Vec<u8>,
        responseFlag: CharPropFlags,
        responseType: WriteType,
    ) {
        let characteristic = Characteristic {
            uuid: uuid,
            service_uuid: SERVICE,
            properties: responseFlag,
        };

        println!("{} : {:?}", uuid_to_string(uuid), cmd);
        self.peripheral
            .write(&characteristic, &cmd, responseType)
            .await
            .unwrap();
    }
}

pub struct Updates {
    receiver: Receiver<Update>
}

impl Updates {
    fn new(receiver: Receiver<Update>) -> Updates {
        return Updates {receiver} 
    }
}

impl Iterator for Updates {
    type Item = Update;

    fn next(&mut self) -> Option<Self::Item> {
        self.receiver.try_recv().ok()
    }
}