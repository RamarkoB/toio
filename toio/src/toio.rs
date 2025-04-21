use btleplug::{
    api::{
        Central, CentralEvent, CharPropFlags, Characteristic, Manager as _, Peripheral, ScanFilter,
        ValueNotification, WriteType,
    },
    platform,
    platform::{Adapter, Manager},
};
use futures::stream::StreamExt;
use std::error::Error;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::time::timeout;
use uuid::Uuid;

pub const SERVICE: Uuid = Uuid::from_u128(0x10B20100_5B3B_4571_9508_CF3EFCD7BBAE);
pub const POSITION: Uuid = Uuid::from_u128(0x10B20101_5B3B_4571_9508_CF3EFCD7BBAE);
pub const MOTOR: Uuid = Uuid::from_u128(0x10B20102_5B3B_4571_9508_CF3EFCD7BBAE);
pub const LIGHT: Uuid = Uuid::from_u128(0x10B20103_5B3B_4571_9508_CF3EFCD7BBAE);
pub const SOUND: Uuid = Uuid::from_u128(0x10B20104_5B3B_4571_9508_CF3EFCD7BBAE);
pub const MOTION: Uuid = Uuid::from_u128(0x10B20106_5B3B_4571_9508_CF3EFCD7BBAE);
pub const BUTTON: Uuid = Uuid::from_u128(0x10B20107_5B3B_4571_9508_CF3EFCD7BBAE);
pub const BATTERY: Uuid = Uuid::from_u128(0x10B20108_5B3B_4571_9508_CF3EFCD7BBAE);
pub const CONFIG: Uuid = Uuid::from_u128(0x10B201FF_5B3B_4571_9508_CF3EFCD7BBAE);

/// matches UUIDs of toios a string of their coresponding service
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
#[derive(Clone, Debug)]
pub struct TargetCommand {
    pub x_target: u16,
    pub y_target: u16,
    pub theta_target: u16,
}

/// Format for a RGB color to plug into the MultiLed varient
/// of the Command enum. By putting multiple of these into a vector,
/// you can send a a series of colors for a toio to flash on its led
/// in a sequence.
#[derive(Clone, Debug)]
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
#[derive(Clone, Debug)]
pub struct MidiCommand {
    pub duration: u8,
    pub note: u8,
    pub volume: u8,
}

/// An enum to list out all possible commands to send to a toio
#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum Command {
    //Request Commands
    MotionRequest,
    MagneticRequest,
    PostureRequest {
        format: u8,
    },

    //Motor Commands
    MotorControl {
        left_direction: u8,
        left_speed: u8,
        right_direction: u8,
        right_speed: u8,
    },
    MotorDuration {
        left_direction: u8,
        left_speed: u8,
        right_direction: u8,
        right_speed: u8,
        duration: u8,
    },
    MotorTarget {
        control: u8,
        timeout: u8,
        move_type: u8,
        max_speed: u8,
        speed_change: u8,
        x_target: u16,
        y_target: u16,
        theta_target: u16,
    },
    MultiTarget {
        control: u8,
        timeout: u8,
        move_type: u8,
        max_speed: u8,
        speed_change: u8,
        op_add: u8,
        targets: Vec<TargetCommand>,
    },
    MotorAcceleration {
        velocity: u8,
        acceleration: u8,
        rotational_velocity: u16,
        rotational_direction: u8,
        direction: u8,
        priority: u8,
        duration: u8,
    },

    //Light Commands
    LedOff,
    Led {
        duration: u8,
        red: u8,
        green: u8,
        blue: u8,
    },
    MultiLed {
        repetitions: u8,
        lights: Vec<LedCommand>,
    },

    //Sound Commands
    SoundOff,
    Sound {
        sound_effect: u8,
        volume: u8,
    },
    Midi {
        repetitions: u8,
        notes: Vec<MidiCommand>,
    },
}

fn parse_target_command(vals: Vec<TargetCommand>) -> Vec<u8> {
    let mut cmd = vec![];

    for target in vals.iter() {
        cmd.push((target.x_target & 0x00FF) as u8);
        cmd.push(((target.x_target & 0xFF00) >> 8) as u8);
        cmd.push((target.y_target & 0x00FF) as u8);
        cmd.push(((target.y_target & 0xFF00) >> 8) as u8);
        cmd.push((target.theta_target & 0x00FF) as u8);
        cmd.push(((target.theta_target & 0xFF00) >> 8) as u8);
    }

    return cmd;
}

fn parse_led_command(repetitions: u8, vals: Vec<LedCommand>) -> Vec<u8> {
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

fn parse_midi_command(repetitions: u8, vals: Vec<MidiCommand>) -> Vec<u8> {
    let mut cmd = vec![0x03, repetitions, vals.len() as u8];

    for note in vals.iter() {
        cmd.push(note.duration);
        cmd.push(note.note);
        cmd.push(note.volume);
    }

    return cmd;
}

/// An enum to list out all possible updates to recieve from a toio
#[allow(dead_code)]
#[derive(Debug)]
pub enum Update {
    Position {
        x_center: u16,
        y_center: u16,
        theta: u16,
        x_sensor: u16,
        y_sensor: u16,
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
        left_speed: u8,
        right_speed: u8,
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
    pub name: String,
    peripheral: platform::Peripheral,
}

impl Toio {
    pub fn new(name: String, peripheral: platform::Peripheral) -> Toio {
        Toio { name, peripheral }
    }

    pub async fn connect(&self) -> bool {
        if let Err(_) = self.peripheral.connect().await {
            return false;
        } else if let Err(_) = self.peripheral.discover_services().await {
            return false;
        }

        for characteristic in self.peripheral.characteristics().into_iter() {
            if !characteristic.properties.contains(CharPropFlags::NOTIFY) {
                continue;
            }

            if let Err(err) = self.peripheral.subscribe(&characteristic).await {
                eprintln!("Error connecting to characteristic, skipping: {}", err);
                continue;
            }
        }

        return true;
    }

    pub async fn updates(&self) -> Result<Updates, Box<dyn Error>> {
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

        return Ok(Updates::new(rx));
    }

    pub async fn is_connected(&self) -> bool {
        return self.peripheral.is_connected().await.unwrap();
    }

    fn get_update(notification: ValueNotification) -> Option<Update> {
        let vals = notification.value;
        match notification.uuid {
            POSITION => match vals[0] {
                0x01 => Some(Update::Position {
                    x_center: vals[1] as u16 | (vals[2] as u16) << 8,
                    y_center: vals[3] as u16 | (vals[4] as u16) << 8,
                    theta: vals[5] as u16 | (vals[6] as u16) << 8,
                    x_sensor: vals[7] as u16 | (vals[8] as u16) << 8,
                    y_sensor: vals[9] as u16 | (vals[10] as u16) << 8,
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
                    left_speed: vals[1],
                    right_speed: vals[2],
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
            Command::LedOff | Command::Led { .. } | Command::MultiLed { .. } => LIGHT,
            Command::SoundOff | Command::Sound { .. } | Command::Midi { .. } => SOUND,
        };

        let (response_flag, response_type) = match uuid {
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
                left_direction,
                left_speed,
                right_direction,
                right_speed,
            } => {
                vec![
                    0x01,
                    0x01,
                    left_direction,
                    left_speed,
                    0x02,
                    right_direction,
                    right_speed,
                ]
            }
            Command::MotorDuration {
                left_direction,
                left_speed,
                right_direction,
                right_speed,
                duration,
            } => {
                vec![
                    0x02,
                    0x01,
                    left_direction,
                    left_speed,
                    0x02,
                    right_direction,
                    right_speed,
                    duration,
                ]
            }
            Command::MotorTarget {
                control,
                timeout,
                move_type,
                max_speed,
                speed_change,
                x_target,
                y_target,
                theta_target,
            } => {
                vec![
                    0x03,
                    control,
                    timeout,
                    move_type,
                    max_speed,
                    speed_change,
                    0x00,
                    (x_target & 0x00FF) as u8,
                    ((x_target & 0xFF00) >> 8) as u8,
                    (y_target & 0x00FF) as u8,
                    ((y_target & 0xFF00) >> 8) as u8,
                    (theta_target & 0x00FF) as u8,
                    ((theta_target & 0xFF00) >> 8) as u8,
                ]
            }
            Command::MultiTarget {
                control,
                timeout,
                move_type,
                max_speed,
                speed_change,
                op_add,
                targets,
            } => {
                let mut cmd = vec![
                    0x04,
                    control,
                    timeout,
                    move_type,
                    max_speed,
                    speed_change,
                    0x00,
                    op_add,
                ];
                cmd.append(&mut parse_target_command(targets));
                cmd
            }

            Command::MotorAcceleration {
                velocity,
                acceleration,
                rotational_velocity,
                rotational_direction,
                direction,
                priority,
                duration,
            } => {
                vec![
                    0x05,
                    velocity,
                    acceleration,
                    (rotational_velocity & 0x00FF) as u8,
                    ((rotational_velocity & 0xFF00) >> 8) as u8,
                    rotational_direction,
                    direction,
                    priority,
                    duration,
                ]
            }
            Command::LedOff => {
                vec![0x01]
            }
            Command::Led {
                duration,
                red,
                green,
                blue,
            } => {
                vec![0x03, duration, 0x01, 0x01, red, green, blue]
            }
            Command::MultiLed {
                repetitions,
                lights,
            } => parse_led_command(repetitions, lights),
            Command::SoundOff => {
                vec![0x01]
            }
            Command::Sound {
                sound_effect,
                volume,
            } => {
                vec![
                    0x02,         //sound
                    sound_effect, //sound effect ID
                    volume,       //volume
                ]
            }
            Command::Midi { repetitions, notes } => parse_midi_command(repetitions, notes),
        };

        self.write(uuid, cmd, response_flag, response_type).await;
    }

    pub async fn write(
        &self,
        uuid: Uuid,
        cmd: Vec<u8>,
        response_flag: CharPropFlags,
        response_type: WriteType,
    ) {
        let characteristic = Characteristic {
            uuid: uuid,
            service_uuid: SERVICE,
            properties: response_flag,
        };

        // println!("{} : {:?}", uuid_to_string(uuid), cmd);
        self.peripheral
            .write(&characteristic, &cmd, response_type)
            .await
            .unwrap();
    }
}

pub struct ToioScanner {
    central: Adapter,
}

impl ToioScanner {
    pub async fn new() -> Result<ToioScanner, Box<dyn Error>> {
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

        Ok(ToioScanner { central })
    }

    pub async fn search(&self) -> Result<Toios, Box<dyn Error>> {
        // Each adapter has an event stream, we fetch via events(),
        // simplifying the type, this will return what is essentially a
        // Future<Result<Stream<Item=CentralEvent>>>.
        let central = self.central.clone();
        let mut events = central.events().await?;

        // start scanning for devices
        central
            .start_scan(ScanFilter {
                services: vec![SERVICE, CONFIG],
            })
            .await?;

        let (tx, rx) = mpsc::channel(32);

        //Discovery Async Task
        tokio::spawn(async move {
            while let Some(event) = events.next().await {
                match event {
                    CentralEvent::DeviceDiscovered(id) => {
                        let peripheral = central.peripheral(&id).await.unwrap();
                        Self::try_connect(peripheral, &tx).await;
                    }
                    // CentralEvent::DeviceUpdated(id) => {
                    //     let peripheral = central.peripheral(&id).await.unwrap();
                    //     Self::try_connect(peripheral, &tx).await;
                    // }
                    CentralEvent::DeviceDisconnected(_) => {
                        // println!("Device Disconnected! {:?}", id);
                    }
                    _ => {}
                }
            }
        });

        return Ok(Toios::new(rx));
    }

    async fn try_connect(peripheral: platform::Peripheral, tx: &Sender<Toio>) {
        if let Some(properties) = peripheral.properties().await.unwrap() {
            let fullname = properties.local_name.unwrap_or("".to_string());
            if peripheral.is_connected().await.unwrap() || !fullname.contains(&"toio") {
                return;
            }

            let name: Vec<&str> = fullname.split('-').collect();

            // println!("{} : {:?}", name, properties.services);
            tx.send(Toio::new(
                name.last().unwrap_or(&&" ").to_string(),
                peripheral,
            ))
            .await
            .unwrap();
        }
    }
}

pub struct Toios {
    receiver: Receiver<Toio>,
}

impl Toios {
    fn new(receiver: Receiver<Toio>) -> Toios {
        return Toios { receiver };
    }

    pub async fn next(&mut self) -> Option<Toio> {
        return self.receiver.recv().await;
    }
}

pub struct Updates {
    receiver: Receiver<Update>,
}

impl Updates {
    fn new(receiver: Receiver<Update>) -> Updates {
        return Updates { receiver };
    }

    pub async fn next(&mut self) -> Option<Update> {
        return self.receiver.recv().await;
    }
}
