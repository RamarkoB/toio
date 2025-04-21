use crate::toio::*;

use std::error::Error;
use std::io::stdout;
use std::sync::Arc;
use std::time::SystemTime;
use std::vec;

use tokio::sync::RwLock;

use crossterm::{
    cursor::Show,
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

pub struct ToioUI {
    pub toio: Toio,
    pub name: String,
    pub id: String,
    pub battery: Arc<RwLock<Option<u8>>>,
    pub last_update: Arc<RwLock<Option<SystemTime>>>,
    pub last_command: Arc<RwLock<Option<SystemTime>>>,
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

fn return_toio_id(name: &str) -> Option<usize> {
    return IDARR.iter().position(|&r| r == name);
}

pub fn ui(toio_info: Vec<(String, String, String, String, String, bool)>) -> impl Fn(&mut Frame) {
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

pub fn setup_terminal() -> Result<Terminal<CrosstermBackend<std::io::Stdout>>, Box<dyn Error>> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let terminal = Terminal::new(CrosstermBackend::new(stdout()))?;
    Ok(terminal)
}

pub fn exit_terminal() -> Result<(), Box<dyn Error>> {
    disable_raw_mode()?;
    stdout().execute(Show)?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
