use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::str::FromStr;
use std::sync::mpsc::{Receiver, TryRecvError};
use std::time::Instant;
use anyhow::anyhow;
use muehle_game::datastructures::movement::Move;
use muehle_game::datastructures::phase::GamePhase;
use muehle_game::datastructures::position::Position;

#[derive(PartialEq)]
enum AgentState {
    Idle,
    Computing
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

        let receive_output = spawn_line_reader(child.stdout.take().unwrap());
        let receive_err = spawn_line_reader(child.stderr.take().unwrap());
        let input = child.stdin.take().unwrap();

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
            self.log_line("STDERR", &line);
        }
    }

    fn log_line(&mut self, prefix: &str, line: &str) {
        self.log.push_str(prefix);
        self.log.push_str(": ");
        self.log.push_str(line);
        self.log.push('\n');
    }

    pub fn query(&mut self, conf: &Position, phase: GamePhase) -> anyhow::Result<Option<Move>> {
        if self.state == AgentState::Idle {
            self.time.begin_query();
            let phase = match phase {
                GamePhase::Placing => format!("P {}", (9 - conf.pieces_to_place_for_player(conf.active_player())) + 1),
                GamePhase::Moving | GamePhase::Flying => "M".to_owned()
            };

            let cmd = format!("{} {}", phase, conf.to_string());
            writeln!(self.input, "{}", &cmd)?;
            self.log_line("QUERY", &cmd);

            self.state = AgentState::Computing;
        } else {
            match self.receive_output.try_recv() {
                Ok(line) => {
                    self.log_line("REPLY", &line);
                    self.state = AgentState::Idle;
                    let m = Move::from_str(&line)?;
                    self.time.complete_query();
                    return Ok(Some(m));
                }
                Err(TryRecvError::Disconnected) => return Err(anyhow!("agent process exited")),
                Err(TryRecvError::Empty) => ()
            }
        }
        Ok(None)
    }

    pub fn kill(&mut self) -> anyhow::Result<()> {
        self.process.kill()?;
        Ok(())
    }
}

fn spawn_line_reader(stream: impl Read + Send + 'static) -> Receiver<String> {
    let (sender, receiver) = std::sync::mpsc::channel();
    std::thread::spawn(move || {
        for line in BufReader::new(stream).lines().map_while(Result::ok) {
            if sender.send(line).is_err() {
                break;
            }
        }
    });
    receiver
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
