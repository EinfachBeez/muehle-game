use std::io::{Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::str::FromStr;
use std::sync::mpsc::Receiver;
use std::time::Instant;
use muehle_game::datastructures::movement::Move;
use muehle_game::datastructures::phase::GamePhase;
use muehle_game::datastructures::position::Position;

#[derive(PartialEq)]
enum AgentState {
    Idle,
    Computing,
    Error
}

pub struct Agent {
    process: Child,
    receive_output: Receiver<String>,
    receive_err: Receiver<String>,
    input: ChildStdin,
    state: AgentState,
    pub time: TimeTracker,
    pub log: String,
}

impl Agent {
    pub fn new(exe: &str) -> anyhow::Result<Self> {
        let mut child = Command::new(exe)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .stdin(Stdio::piped())
            .spawn()?;

        let output = child.stdout.take().unwrap();
        let err = child.stderr.take().unwrap();
        let input = child.stdin.take().unwrap();

        let (send_output, receive_output) = std::sync::mpsc::channel();
        let (send_err, receive_err) = std::sync::mpsc::channel();

        // Spawn thread for stdout
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(output);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if send_output.send(line).is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        // Spawn thread for stderr
        std::thread::spawn(move || {
            use std::io::{BufRead, BufReader};
            let reader = BufReader::new(err);
            for line in reader.lines() {
                if let Ok(line) = line {
                    if send_err.send(line).is_err() {
                        break;
                    }
                } else {
                    break;
                }
            }
        });

        Ok(Self {
            process: child,
            receive_output,
            receive_err,
            input,
            state: AgentState::Idle,
            time: TimeTracker::default(),
            log: String::with_capacity(1000000),
        })
    }

    pub fn update_log(&mut self) {
        while let Ok(line) = self.receive_err.try_recv() {
            self.log.push_str("STDERR: ");
            self.log.push_str(&line);
            self.log.push('\n');
        }
    }

    pub fn query(&mut self, conf: &Position, phase: GamePhase) -> anyhow::Result<Option<Move>> {
        if self.state == AgentState::Idle {
            self.time.begin_query();
            let phase = match phase {
                GamePhase::Placing => format!("P {}", (9 - conf.pieces_to_place_for_player(conf.active_player())) + 1),
                GamePhase::Moving => "M".to_owned(),
                GamePhase::Flying => "M".to_owned()
            };

            let cmd = format!("{} {}", phase, conf.to_string());
            writeln!(self.input, "{}", &cmd)?;
            self.log.push_str("QUERY: ");
            self.log.push_str(&cmd);
            self.log.push('\n');

            self.state = AgentState::Computing;
        } else if self.state == AgentState::Computing {
            while let Ok(line) = self.receive_output.try_recv() {
                self.log.push_str("REPLY: ");
                self.log.push_str(&line);
                self.log.push('\n');
                if let Ok(m) = Move::from_str(&line) {
                    self.time.complete_query();
                    self.state = AgentState::Idle;
                    return Ok(Some(m));
                };
                self.state = AgentState::Error;
                break;
            }
        }
        Ok(None)
    }

    pub fn kill(&mut self) -> anyhow::Result<()> {
        self.process.kill()?;
        Ok(())
    }
}

pub struct TimeTracker {
    data: Vec<u32>,
    start: Instant,
}

impl Default for TimeTracker {
    fn default() -> Self {
        Self {
            data: Default::default(),
            start: Instant::now(),
        }
    }
}

impl TimeTracker {
    fn begin_query(&mut self) {
        self.start = Instant::now();
    }

    fn complete_query(&mut self) {
        let duration = self.start.elapsed().as_millis() as u32;
        self.push(duration);
    }

    fn push(&mut self, duration: u32) {
        self.data.push(duration);
    }

    pub fn max(&self) -> f32 {
        self.data.iter().copied().max().unwrap_or(0) as f32 * 0.001
    }

    pub fn avg(&self) -> f32 {
        let sum = self.data.iter().copied().sum::<u32>() as f32 * 0.001;
        let mut cnt = self.data.len();
        if cnt == 0 {
            cnt = 1;
        }
        sum / cnt as f32
    }

    fn last(&self) -> u32 {
        self.data.last().copied().unwrap_or(0)
    }
}
