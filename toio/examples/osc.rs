use ::toio::*;

use std::error::Error;
use std::io::{self, stdout};
use std::net::UdpSocket;
use std::process;
use std::sync::Arc;
use std::time::SystemTime;
use std::vec;

use futures::future::join_all;
use rosc::encoder;
use rosc::{OscMessage, OscPacket, OscType};
use tokio::sync::RwLock;

use crossterm::{
    cursor::Show,
    event::{self, Event, KeyCode},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::{
    prelude::*,
    widgets::{block::*, *},
};

const IDARR: [&str; 193] = [
    "Individual ID", //TOIO Num
    "0",             // #1
    "j1c",           // #2
    "r81",           // #3
    "26E",           // #4
    "76t",           // #5
    "broken",        // #6
    "k5k",           // #7
    "h41",           // #8
    "0",             // #9
    "0",             // #10
    "0",             // #11
    "Q3A",           // #12
    "03a",           // #13
    "0",             // #14
    "K0m",           // #15
    "0",             // #16
    "0",             // #17
    "p8B",           // #18
    "91B",           // #19
    "p75",           // #20
    "G1E",           // #21
    "k2L",           // #22
    "b5p",           // #23
    "J6C",           // #24
    "0",             // #25
    "b8T",           // #26
    "b6A",           // #27
    "01c",           // #28
    "0",             // #29
    "0",             // #30
    "E2N",           // #31
    "G7t",           // #32
    "L6T",           // #33
    "C0E",           // #34
    "t79",           // #35
    "J6k",           // #36
    "d6f",           // #37
    "0",             // #38
    "M75",           // #39
    "310",           // #40
    "M5p",           // #41
    "A4a",           // #42
    "M9J",           // #43
    "i01",           // #44
    "T5m",           // #45
    "j1G",           // #46
    "40G",           // #47
    "L6n",           // #48
    "a3F",           // #49
    "J8d",           // #50
    "227",           // #51
    "k4i",           // #52
    "J68",           // #53
    "90J",           // #54
    "k96",           // #55
    "0",             // #56
    "0",             // #57
    "0",             // #58
    "0",             // #59
    "0",             // #60
    "0",             // #61
    "0",             // #62
    "0",             // #63
    "0",             // #64
    "0",             // #65
    "0",             // #66
    "0",             // #67
    "0",             // #68
    "0",             // #69
    "0",             // #70
    "0",             // #71
    "0",             // #72
    "0",             // #73
    "0",             // #74
    "0",             // #75
    "0",             // #76
    "0",             // #77
    "0",             // #78
    "E7c",           // #79
    "P1B",           // #80
    "F2B",           // #81
    "L1H",           // #82
    "D5i",           // #83
    "m4Q",           // #84
    "m1k",           // #85
    "r52",           // #86
    "k89",           // #87
    "D2K",           // #88
    "65r",           // #89
    "f3K",           // #90
    "13c",           // #91
    "e1a",           // #92
    "0",             // #93
    "e6e",           // #94
    "07F",           // #95
    "m8k",           // #96
    "79H",           // #97
    "0",             // #98
    "i1M",           // #99
    "R3C",           // #100
    "D98",           // #101
    "m86",           // #102
    "a66",           // #103
    "0",             // #104
    "E8T",           // #105
    "J8n",           // #106
    "N0b",           // #107
    "586",           // #108
    "p50",           // #109
    "c9k",           // #110
    "N0N",           // #111
    "0",             // #112
    "B1m",           // #113
    "h7E",           // #114
    "c05",           // #115
    "K20",           // #116
    "32D",           // #117
    "F19",           // #118
    "r4d",           // #119
    "D2F",           // #120
    "D0m",           // #121
    "m6B",           // #122
    "M0j",           // #123
    "Q8G",           // #124
    "A1t",           // #125
    "p7J",           // #126
    "t0H",           // #127
    "M5i",           // #128
    "j1L",           // #129
    "e7i",           // #130
    "T1E",           // #131
    "85i",           // #132
    "71H",           // #133
    "20H",           // #134
    "T9n",           // #135
    "58B",           // #136
    "J4R",           // #137
    "93N",           // #138
    "t0F",           // #139
    "M7G",           // #140
    "r4P",           // #141
    "i1d",           // #142
    "a22",           // #143
    "M39",           // #144
    "C23",           // #145
    "816",           // #146
    "E0M",           // #147
    "T4b",           // #148
    "L1L",           // #149
    "i5m",           // #150
    "P2R",           // #151
    "t77",           // #152
    "A5E",           // #153
    "88e",           // #154
    "k1b",           // #155
    "m04",           // #156
    "41b",           // #157
    "B4k",           // #158
    "J1M",           // #159
    "H4M",           // #160
    "C1D",           // #161
    "12K",           // #162
    "822",           // #163
    "E1T",           // #164
    "Q4H",           // #165
    "k4d",           // #166
    "k4J",           // #167
    "L70",           // #168
    "31f",           // #169
    "G1P",           // #170
    "34e",           // #171
    "939",           // #172
    "24F",           // #173
    "43r",           // #174
    "M81",           // #175
    "01E",           // #176
    "A0N",           // #177
    "65f",           // #178
    "Q6p",           // #179
    "93R",           // #180
    "r0i",           // #181
    "A35",           // #182
    "P40",           // #183
    "G9R",           // #184
    "c7C",           // #185
    "P17",           // #186
    "76f",           // #187
    "99p",           // #188
    "96E",           // #189
    "p3E",           // #190
    "h6t",           // #191
    "n2L",           // #192
];

fn return_toio_id(name: &str) -> Option<usize> {
    return IDARR.iter().position(|&r| r == name);
}

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

    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

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
            disable_raw_mode()?;
            stdout().execute(Show)?;
            stdout().execute(LeaveAlternateScreen)?;
            process::exit(0);
        }
    }
}

fn handle_events() -> io::Result<bool> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind == event::KeyEventKind::Press && key.code == KeyCode::Char('q') {
                return Ok(true);
            }
        }
    }
    Ok(false)
}

fn handle_packet(packet: OscPacket) -> Option<(usize, Command)> {
    match packet {
        OscPacket::Message(msg) => {
            let mut vals: Vec<i32> = msg
                .args
                .iter()
                .flat_map(|val| match val {
                    OscType::Int(i) => Some(*i),
                    _ => None,
                })
                .collect();

            // extract command
            let cmd: Option<Command> = match msg.addr.as_ref() {
                "/motorbasic" => Some(Command::MotorControl {
                    left_direction: vals[1] as u8,
                    left_speed: vals[2] as u8,
                    right_direction: vals[3] as u8,
                    right_speed: vals[4] as u8,
                }),
                "/motorduration" => Some(Command::MotorDuration {
                    left_direction: vals[1] as u8,
                    left_speed: vals[2] as u8,
                    right_direction: vals[3] as u8,
                    right_speed: vals[4] as u8,
                    duration: vals[5] as u8,
                }),
                "/motortarget" => Some(Command::MotorTarget {
                    control: vals[1] as u8,
                    timeout: vals[2] as u8,
                    move_type: vals[3] as u8,
                    max_speed: vals[4] as u8,
                    speed_change: vals[5] as u8,
                    x_target: vals[6] as u16,
                    y_target: vals[7] as u16,
                    theta_target: vals[8] as u16,
                }),
                "/motoracceleration" => Some(Command::MotorAcceleration {
                    velocity: vals[1] as u8,
                    acceleration: vals[2] as u8,
                    rotational_velocity: vals[3] as u16,
                    rotational_direction: vals[4] as u8,
                    direction: vals[5] as u8,
                    priority: vals[6] as u8,
                    duration: vals[7] as u8,
                }),
                "/multitarget" => Some(Command::MultiTarget {
                    control: vals[1] as u8,
                    timeout: vals[2] as u8,
                    move_type: vals[3] as u8,
                    max_speed: vals[4] as u8,
                    speed_change: vals[5] as u8,
                    op_add: 1,
                    targets: vals
                        .split_off(6)
                        .chunks(3)
                        .map(|target| TargetCommand {
                            x_target: target[0] as u16,
                            y_target: target[1] as u16,
                            theta_target: target[2] as u16,
                        })
                        .collect(),
                }),
                "/led" => Some(Command::Led {
                    duration: vals[1] as u8,
                    red: vals[2] as u8,
                    green: vals[3] as u8,
                    blue: vals[4] as u8,
                }),
                "/multiLed" => Some(Command::MultiLed {
                    repetitions: vals[1] as u8,
                    lights: vals
                        .split_off(2)
                        .chunks(4)
                        .map(|light| LedCommand {
                            duration: light[0] as u8,
                            red: light[1] as u8,
                            green: light[2] as u8,
                            blue: light[3] as u8,
                        })
                        .collect(),
                }),
                "/sound" => Some(Command::Sound {
                    sound_effect: vals[1] as u8,
                    volume: vals[2] as u8,
                }),
                "/midi" => Some(Command::Midi {
                    repetitions: vals[1] as u8,
                    notes: vals
                        .split_off(2)
                        .chunks(3)
                        .map(|note| MidiCommand {
                            duration: note[0] as u8,
                            note: note[1] as u8,
                            volume: note[2] as u8,
                        })
                        .collect(),
                }),

                _ => None,
            };

            // Return pair of (toioID, pair)
            return cmd.map(|cmd| (vals[0] as usize, cmd));
        }
        _ => None,
    }
}

fn send_packet(socket: &UdpSocket, to_addr: &str, id: usize, update: Update) {
    let vals: Option<(&str, Vec<i32>)> = match update {
        Update::Position {
            x_center,
            y_center,
            theta,
            ..
        } => Some((
            "/position",
            vec![x_center as i32, y_center as i32, theta as i32],
        )),
        Update::Battery { level } => Some(("/battery", vec![level as i32])),
        Update::Button { pressed } => Some(("/button", vec![if pressed { 0x00 } else { 0x80 }])),
        Update::Motion {
            horizontal,
            collision,
            double_tap,
            posture,
            shake,
        } => Some((
            "/motion",
            vec![
                horizontal as i32,
                collision as i32,
                double_tap as i32,
                posture as i32,
                shake as i32,
            ],
        )),
        Update::MotorTargetResponse { control, response } => {
            Some(("/motorresponse", vec![control as i32, response as i32]))
        }
        Update::MultiTargetResponse { control, response } => {
            Some(("/motorresponse", vec![control as i32, response as i32]))
        }
        Update::Standard { standard, theta } => {
            Some(("/standard", vec![standard as i32, theta as i32]))
        }
        Update::PositionMissed => Some(("/positionMissed", vec![])),
        Update::StandardMissed => Some(("/standardMissed", vec![])),
        Update::MotorSpeed {
            left_speed,
            right_speed,
        } => Some(("/motorSpeed", vec![left_speed as i32, right_speed as i32])),
        Update::PostureEuler { roll, pitch, yaw } => {
            Some(("/postureEuler", vec![roll as i32, pitch as i32, yaw as i32]))
        }
        Update::PostureQuaternion { w, x, y, z } => Some((
            "/PostureQuaternion",
            vec![w as i32, x as i32, y as i32, z as i32],
        )),
        // Update::PostureHighPrecisionEuler { .. } => todo!(),
        Update::Magnetic {
            state,
            strength,
            forcex,
            forcey,
            forcez,
        } => Some((
            "/magnetic",
            vec![
                state as i32,
                strength as i32,
                forcex as i32,
                forcey as i32,
                forcez as i32,
            ],
        )),
        _ => None,
    };

    if let Some((addr, args)) = vals {
        let msg = encoder::encode(&OscPacket::Message(OscMessage {
            addr: addr.to_string(),
            args: vec![id as i32]
                .iter()
                .chain(args.iter())
                .map(|x| OscType::Int(*x))
                .collect(),
        }))
        .unwrap();

        socket.send_to(&msg, to_addr).unwrap();
    }
}

struct ToioUI {
    pub toio: Toio,
    name: String,
    id: String,
    battery: Arc<RwLock<Option<u8>>>,
    last_update: Arc<RwLock<Option<SystemTime>>>,
    last_command: Arc<RwLock<Option<SystemTime>>>,
}

impl ToioUI {
    pub fn new(toio: Toio) -> ToioUI {
        return ToioUI {
            name: toio.name.clone(),
            id: if let Some(id) = return_toio_id(&toio.name) {
                format!("{}", id)
            } else {
                "N/A".to_owned()
            },
            battery: Arc::new(RwLock::new(None)),
            toio,
            last_update: Arc::new(RwLock::new(None)),
            last_command: Arc::new(RwLock::new(None)),
        };
    }

    pub fn get_battery(&self) -> Arc<RwLock<Option<u8>>> {
        return self.battery.clone();
    }

    pub fn get_last_update(&self) -> Arc<RwLock<Option<SystemTime>>> {
        return self.last_update.clone();
    }

    pub fn get_last_command(&self) -> Arc<RwLock<Option<SystemTime>>> {
        return self.last_command.clone();
    }

    pub async fn is_connected(&self) -> bool {
        return self.toio.is_connected().await;
    }
}

fn ui(toio_info: Vec<(String, String, String, String, String, bool)>) -> impl Fn(&mut Frame) {
    return move |frame| {
        let area = frame.size();

        let rows: Vec<Row> = toio_info
            .iter()
            .enumerate()
            .map(|(i, val)| {
                let battery = val.2.clone();
                let battery_color = if let Ok(level) = battery.parse::<i32>() {
                    if level == 10 {
                        Style::new().red()
                    } else if level < 50 {
                        Style::new().yellow()
                    } else {
                        Style::new().green()
                    }
                } else {
                    Style::new().white()
                };

                Row::new(vec![
                    Span::raw(format!("{}", i)),
                    Span::raw(val.0.clone()),
                    Span::raw(val.1.clone()),
                    Span::raw(battery).style(battery_color),
                    Span::raw(val.3.clone()),
                    Span::raw(val.4.clone()),
                ])
            })
            .collect();

        let instructions = Title::from(Line::from(vec![" Quit ".into(), "<Q> ".blue().bold()]));

        let widths = [
            Constraint::Length(2),
            Constraint::Length(4),
            Constraint::Length(3),
            Constraint::Length(7),
            Constraint::Length(12),
            Constraint::Length(12),
        ];

        let table = Table::new(rows, widths)
            .column_spacing(7)
            .header(
                Row::new(vec![
                    "",
                    "Name",
                    "ID",
                    "Battery",
                    "Last Update",
                    "Last Command",
                ])
                .style(Style::new().bold()),
            )
            .highlight_style(Style::new().reversed())
            .highlight_symbol(">>");

        frame.render_widget(
            table.block(
                Block::default()
                    .title(" Laptop Toio ")
                    .title(
                        instructions
                            .alignment(Alignment::Center)
                            .position(Position::Bottom),
                    )
                    .borders(Borders::ALL),
            ),
            area,
        );
    };
}
