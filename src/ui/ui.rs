use crate::agent_helper::Agent;
use ggegui::egui::{Color32, Ui, Window};
use ggegui::Gui;
use ggez::event::MouseButton;
use ggez::glam::Vec2;
use ggez::graphics::{Canvas, Color, DrawMode, DrawParam, Drawable, Image, Mesh, PxScale, Rect, Text};
use ggez::input::keyboard::{KeyCode, KeyInput};
use ggez::mint::Point2;
use ggez::*;
use rfd::FileDialog;
use std::f32::consts::PI;
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use muehle_game::calculate_next_position;
use muehle_game::datastructures::movement::Move;
use muehle_game::datastructures::phase::GamePhase;
use muehle_game::datastructures::player::Player;
use muehle_game::datastructures::point::Point;
use muehle_game::datastructures::position::Position;

// pixel size of a point
const POINT_SIZE: f32 = 10.0;
const SCREEN_SIZE: (f32, f32) = (1500.0, 900.0);
const CENTER: (f32, f32) = (SCREEN_SIZE.0 / 2.0, SCREEN_SIZE.1 / 2.0);
const BOARD_CIRCLES: (f32, f32, f32) = (300.0, 200.0, 100.0);

const BACKGROUND_COLOR_TOP_RGB: (u8, u8, u8) = (243, 186, 126);
const BACKGROUND_COLOR_BOTTOM_RGB: (u8, u8, u8) = (209, 129, 61);


const WHITE_STONES_PATH: &str = "/white_piece.png";
const BLACK_PIECE_PATH: &str = "/black_piece.png";


const FPS: u32 = 60;

#[derive(Copy, Clone, Eq, PartialEq)]
enum GuiPhase {
    Initial,
    Active,
    Finished(Option<Player>)
}

#[derive(Copy, Clone, Debug)]
enum Action {
    Choose,
    Move(u8),
    Kill(u8, Option<u8>) // kill is also possible in place phase
}

struct AgentService {
    agent_active: bool,
    agent: Option<Agent>,
    exe: String
}

impl AgentService {
    fn new() -> Self {
        AgentService {
            agent_active: false,
            agent: None,
            exe: String::new()
        }
    }

    pub fn agent_active(&self) -> bool {
        self.agent_active
    }

    pub fn agent(&self) -> &Option<Agent> {
        &self.agent
    }

    pub fn agent_mut(&mut self) -> &mut Option<Agent> {
        &mut self.agent
    }

    pub fn exe(&self) -> &str {
        &self.exe
    }

    pub fn set_agent_active(&mut self, agent_active: bool) {
        self.agent_active = agent_active;
    }

    pub fn set_agent(&mut self, agent: Option<Agent>) {
        self.agent = agent;
    }

    pub fn set_exe(&mut self, exe: String) {
        self.exe = exe;
    }
}

pub struct State {
    gui: Gui,

    position: Position,
    board: [(f32, f32); 24],
    gui_phase: GuiPhase,
    game_timer: Option<Duration>,

    white_stone_image: Image,
    black_stone_image: Image,

    current_action: Action,

    agent_white: AgentService,
    agent_black: AgentService,
    agent_error: Option<String>
}

impl State {
    pub fn new(ctx: &mut Context) -> State {
        let default_position = Position::new(Player::White);
        //let default_position = Position::from_str("W-WWEEEWWWBEEEEEBEEEEEEEWE").unwrap();
        State {
            gui: Gui::new(ctx),

            position: default_position,
            board: generate_board(),
            gui_phase: GuiPhase::Initial,
            game_timer: None,
            white_stone_image: Image::from_path(ctx, WHITE_STONES_PATH).unwrap(),
            black_stone_image: Image::from_path(ctx, BLACK_PIECE_PATH).unwrap(),

            current_action: Action::Choose,

            agent_white: AgentService::new(),
            agent_black: AgentService::new(),
            agent_error: None
        }
    }

    fn start_game(&mut self) {
        self.agent_error = None;

        for (agent_service, player) in [(&mut self.agent_white, Player::White), (&mut self.agent_black, Player::Black)] {
            if !agent_service.agent_active {
                continue;
            }
            match Agent::new(agent_service.exe.as_str()) {
                Ok(agent) => agent_service.set_agent(Some(agent)),
                Err(err) => {
                    self.agent_error = Some(format!("Could not start {} agent ({}): {}", player, agent_service.exe, err));
                    break;
                }
            }
        }
        if self.agent_error.is_some() {
            self.kill_agents();
            return;
        }

        self.position = Position::new(Player::White);
        self.reset_action();
        self.gui_phase = GuiPhase::Active;

        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap();
        self.game_timer = Some(now);
    }

    fn stop_game(&mut self, player_won: Option<Player>) {
        self.gui_phase = GuiPhase::Finished(player_won);
        self.kill_agents();
    }

    fn kill_agents(&mut self) {
        if let Some(mut agent) = self.agent_white.agent_mut().take() {
            let _ = agent.kill();
        }

        if let Some(mut agent) = self.agent_black.agent_mut().take() {
            let _ = agent.kill();
        }
    }

    fn elapsed_time_in_sec(&self) -> Option<u64> {
        if let Some(start) = self.game_timer {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap();
            let elapsed = now - start;
            Some(elapsed.as_secs())
        } else {
            None
        }
    }

    fn piece_at(&self, x: f32, y: f32) -> Option<u8> {
        let radius = 35.0;
        for (i, &(px, py)) in self.board.iter().enumerate() {
            let dx = x - px;
            let dy = y - py;
            if dx*dx + dy*dy <= radius*radius {
                return Some(i as u8);
            }
        }
        None
    }

    fn is_active_player_agent(&self) -> bool {
        match self.position.active_player() {
            Player::White => self.agent_white.agent_active,
            Player::Black => self.agent_black.agent_active
        }
    }

    fn handle_piece_action(&mut self, _ctx: &mut Context, piece_pos: u8) {
        let current_position = &self.position;
        let game_phase = current_position.get_game_phase_for_current_player();

        let is_agent = self.is_active_player_agent();

        if is_agent || self.gui_phase != GuiPhase::Active {
            return;
        }

        match self.current_action {
            Action::Choose => {
                match game_phase {
                    GamePhase::Placing => {
                        if current_position.is_mill_closed(None, piece_pos, current_position.active_player()) {
                            self.update_action(Action::Kill(piece_pos, None));
                            return;
                        }

                        let move_action = Move::Place {position: piece_pos, kill: None};
                        let result = Self::execute_action(current_position, move_action);
                        match result {
                            Some(position) => {
                                self.update_position(position);
                            }
                            _ => {
                                self.update_action(Action::Choose);
                                return;
                            }
                        }
                    }
                    _ => self.update_action(Action::Move(piece_pos))
                }
            },
            Action::Move(from) => {
                match game_phase {
                    GamePhase::Moving | GamePhase::Flying => {
                        if self.position.is_mill_closed(Some(from), piece_pos, current_position.active_player()) {
                            self.update_action(Action::Kill(from, Some(piece_pos)));
                            return;
                        }

                        let move_action = Move::Move {from_position: from, to_position: piece_pos, kill: None};
                        let result = Self::execute_action(current_position, move_action);
                        match result {
                            Some(position) => {
                                self.update_position(position);
                            }
                            _ => {
                                self.update_action(Action::Choose);
                                return;
                            }
                        }
                    }
                    _ => self.update_action(Action::Kill(from, None))
                }
            },
            Action::Kill(from, to, ) => {
                match game_phase {
                    GamePhase::Placing => {
                        let move_action = Move::Place {position: from, kill: Some(piece_pos)};
                        let result = Self::execute_action(current_position, move_action);
                        match result {
                            Some(position) => {
                                self.update_position(position);
                            }
                            _ => {
                                self.update_action(Action::Choose);
                                return;
                            }
                        }
                    }
                    GamePhase::Moving | GamePhase::Flying => {
                        if to.is_none() {
                            self.update_action(Action::Choose);
                            return;
                        }

                        let move_action = Move::Move { from_position: from, to_position: to.unwrap(), kill: Some(piece_pos) };
                        let result = Self::execute_action(current_position, move_action);
                        match result {
                            Some(position) => {
                                self.update_position(position);
                            }
                            _ => {
                                self.update_action(Action::Choose);
                                return;
                            }
                        }
                    }
                }
            },
        }

    }

    fn handle_agent_move(&mut self) {
        let active_player = self.position.active_player();
        let agent_service = match active_player {
            Player::White => &mut self.agent_white,
            Player::Black => &mut self.agent_black
        };

        if !agent_service.agent_active {
            return;
        }

        if let Some(agent) = agent_service.agent_mut() {
            agent.update_log();

            let game_phase = self.position.get_game_phase_for_current_player();

            match agent.query(&self.position, game_phase) {
                Ok(Some(move_action)) => {
                    match calculate_next_position(&self.position, move_action) {
                        Some(new_pos) => {
                            self.update_position(new_pos)
                        }
                        None => {
                            self.agent_error = Some(format!("{} agent suggested invalid move {}", active_player, move_action));
                            self.stop_game(None);
                        }
                    }
                }
                Ok(None) => (),
                Err(err) => {
                    self.agent_error = Some(format!("Error querying {} agent: {}", active_player, err));
                    self.stop_game(None)
                }
            }


        }
    }

    fn execute_action(current_position: &Position, move_action: Move) -> Option<Position> {
        calculate_next_position(current_position, move_action)
    }

    fn update_action(&mut self, action: Action) {
        self.current_action = action;
    }

    fn update_position(&mut self, new_position: Position) {
        self.position = new_position;
        self.reset_action();

        if let Some(player) = self.position.is_game_finished() {
            self.stop_game(Some(player));
        }
    }

    fn reset_action(&mut self) {
        self.current_action = Action::Choose;
    }
}



impl event::EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        while _ctx.time.check_update_time(FPS) {
            if self.gui_phase == GuiPhase::Active {
                self.handle_agent_move();

            }
        }

        display_agent_setup_window(self, _ctx);

        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = Canvas::from_frame(_ctx, Color::WHITE);

        draw_background(_ctx, &mut canvas);
        draw_board(self, _ctx, &mut canvas);

        draw_information(self, _ctx, &mut canvas);
        draw_position(self, &mut canvas);

        draw_current_action(self, _ctx, &mut canvas);
        draw_timer(self, _ctx, &mut canvas);

        canvas.draw(&self.gui, DrawParam::default().dest(Vec2::ZERO));

        canvas.finish(_ctx)?;
        Ok(())
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, _button: MouseButton, _x: f32, _y: f32) -> Result<(), GameError> {
        if self.gui_phase != GuiPhase::Active || self.is_active_player_agent() {
            return Ok(())
        }

        match _button {
            MouseButton::Left => {
                let option = self.piece_at(_x, _y);
                match option {
                    Some(i) => {
                        self.handle_piece_action(_ctx, i);
                    }
                    None => ()
                }


                Ok(())
            }
            _ => Ok(()),
        }
    }

    fn key_down_event(&mut self, _ctx: &mut Context, input: KeyInput, _repeated: bool) -> Result<(), GameError> {
        match input.keycode {
            Some(KeyCode::X) => {
                if self.gui_phase == GuiPhase::Active {
                    self.stop_game(None);
                }
            }
            _ => ()
        }
        Ok(())
    }
}

fn display_agent_setup_window(state: &mut State, ctx: &mut Context) {
    let gui_ctx = state.gui.ctx();

    let is_editable = state.gui_phase != GuiPhase::Active;
    Window::new("Agent Setup").show(&gui_ctx, |ui| {
        ui.set_enabled(is_editable);

        ui.columns(2, |column| {
            column[0].label("White Player (starts):");
            setup_individual_agent(&mut state.agent_white, &mut column[0]);

            column[1].label("Black Player:");
            setup_individual_agent(&mut state.agent_black, &mut column[1]);
        });

        ui.separator();

        if let Some(err) = &state.agent_error {
            ui.colored_label(Color32::RED, err);
        }

        if is_editable {
            if ui.button("Start Game").clicked() {
                state.start_game();
            }
        } else {
            if ui.button("Stop Game (X) ").clicked() {
                state.stop_game(None);
            }
        }

        // Logs

    });
    state.gui.update(ctx);
}

fn setup_individual_agent(agent_service: &mut AgentService, ui: &mut Ui) {
    ui.radio_value(&mut agent_service.agent_active, false, "Human");
    ui.radio_value(&mut agent_service.agent_active, true, "Computer");

    if agent_service.agent_active {
        ui.horizontal(|ui| {
            ui.label("Exe:");
            ui.text_edit_singleline(&mut agent_service.exe);
        });
        if ui.button("Open file...").clicked() {
            if let Some(path) = FileDialog::new().pick_file() {
                agent_service.exe = path.display().to_string();
            }
        }
    }
}

fn draw_timer(state: &State, ctx: &mut Context, canvas: &mut Canvas) {
    let duration = state.elapsed_time_in_sec().unwrap_or(0);
    let mut timer_text;
    let minutes = duration / 60;
    let seconds = duration % 60;
    let mut param = DrawParam::default();
    match state.gui_phase {
        GuiPhase::Initial => {
            timer_text = Text::new("Timer stopped");
            param = param.color(Color::RED);
        }
        GuiPhase::Active => {
            timer_text = Text::new(format!("Time elapsed: {:02}:{:02}", minutes, seconds));
            param = param.color(Color::WHITE);
        }
        GuiPhase::Finished(optional_player) => {
            let winning_text: String = match optional_player {
                Some(player) => format!("{} won!", player),
                None => String::from("Tie!")
            };
            timer_text = Text::new(format!("{} Time spent: {:02}:{:02}", winning_text, minutes, seconds));
            param = param.color(Color::GREEN);
        }
    }
    timer_text.set_scale(PxScale::from(32.0));
    let width = timer_text.dimensions(&ctx.gfx).unwrap().w;
    let dest = Point2 { x: CENTER.0 - width / 2f32, y: SCREEN_SIZE.1 * 0.05 };
    canvas.draw(&timer_text, param.dest(dest));
}

fn draw_position(state: &State, canvas: &mut Canvas) {
    let position = &state.position;

    for (pos, &piece) in position.pieces().iter().enumerate() {
        match piece {
            Point::Occupied(player) => {
                draw_piece_on_board_for_player(state, canvas, player, pos as u8);
            }
            _ => continue
        }
    }
}

fn draw_current_action(state: &State, ctx: &mut Context, canvas: &mut Canvas) {
    let action = state.current_action;

    fn draw_visual(ctx: &mut Context, canvas: &mut Canvas, center: Point2<f32>, color: Color) {
        draw_dashed_circle(
            ctx,
            canvas,
            center,
            50.0,
            5.0,
            10.0,
            10.0,3,
            color
        );
    }

    match action {
        Action::Move(direction) => {
            let (x, y) = state.board[direction as usize];
            draw_visual(ctx, canvas, Point2 { x, y }, Color::WHITE);
        }
        Action::Kill(from, to) => {
            let (x_from, y_from) = state.board[from as usize];
            draw_visual(ctx, canvas, Point2 { x: x_from, y: y_from }, Color::WHITE);

            if let Some(to_pos) = to {
                let (x_to, y_to) = state.board[to_pos as usize];
                draw_visual(ctx, canvas, Point2 { x: x_to, y: y_to }, Color::GREEN);
            }
        }
        _ => ()
    };
}

// Function generated with AI
fn draw_dashed_circle(
    ctx: &mut Context,
    canvas: &mut Canvas,
    center: Point2<f32>,
    radius: f32,
    stroke_width: f32,
    dash_length: f32, // arc length in pixels for dash
    gap_length: f32,  // arc length in pixels for gap
    samples_per_dash: usize,
    color: Color,
) {
    // convert arc-length to angle (radians): angle = length / radius
    let dash_angle = dash_length / radius;
    let gap_angle = gap_length / radius;

    let mut angle = 0.0_f32;
    while angle < 2.0 * PI - 1e-6 {
        let start = angle;
        let end = (angle + dash_angle).min(2.0 * PI);

        // sample points along this arc segment
        let mut pts: Vec<Point2<f32>> = Vec::with_capacity(samples_per_dash + 1);
        for i in 0..=samples_per_dash {
            let t = i as f32 / samples_per_dash as f32;
            let theta = start + (end - start) * t;
            let x = center.x + radius * theta.cos();
            let y = center.y + radius * theta.sin();
            pts.push(Point2 { x, y });
        }

        // draw the dash as a line mesh (if at least 2 points)
        if pts.len() >= 2 {
            let mesh = Mesh::new_line(ctx, &pts, stroke_width, color).unwrap();
            canvas.draw(&mesh, DrawParam::default());
        }

        angle += dash_angle + gap_angle;
    }
}

fn draw_information(state: &State, ctx: &mut Context, canvas: &mut Canvas) {
    draw_player_information(state, ctx, canvas, Player::White);
    draw_player_information(state, ctx, canvas, Player::Black);
}

fn draw_player_information(state: &State, ctx: &mut Context, canvas: &mut Canvas, player: Player) {
    let position = &state.position;
    let active_player = position.active_player();

    draw_player_name(canvas, ctx, player, active_player == player);
    draw_remaining_pieces(canvas, ctx, state, player);
    draw_game_phase(canvas, position, player);
}

fn draw_player_name(canvas: &mut Canvas, ctx: &Context, player: Player, is_active_player: bool) {
    let mut player_name = Text::new(player.to_string().to_uppercase());
    player_name.set_scale(PxScale::from(40.0));

    let text_height = player_name.dimensions(&ctx.gfx).unwrap().h;
    let param = DrawParam::default()
        .dest(Point2 {x: calculate_x_offset_for_player(player), y: 150.0 - text_height})
        .color(if is_active_player {Color::GREEN} else {Color::WHITE});
    canvas.draw(&player_name, param);
}

fn draw_remaining_pieces(canvas: &mut Canvas, ctx: &mut Context, state: &State, player: Player) {
    let position = &state.position;
    let piece_image = if player == Player::White {
        &state.white_stone_image
    } else {
        &state.black_stone_image
    };
    let pieces_to_place = position.pieces_to_place_for_player(player);

    let mesh = generate_board_rectangle(ctx, 0.0,0.0, 150.0, 600.0);
    let draw_param = DrawParam::default()
        .dest(Point2 { x: calculate_x_offset_for_player(player), y: 150.0 });
    canvas.draw(&mesh, draw_param);

    for i in 0..pieces_to_place {
        let param = stone_param(
            calculate_x_offset_for_player(player) + 75.0, 200.0 + (i as f32 * 60.0),
            piece_image
        );
        canvas.draw(piece_image, param);
    }
}

fn draw_game_phase(canvas: &mut Canvas, position: &Position, player: Player) {
    let player_phase = position.get_game_phase_for_player(player);
    let mut player_phase_text = Text::new(player_phase.to_string());
    player_phase_text.set_scale(PxScale::from(40.0));
    let text_position = Point2 {x: calculate_x_offset_for_player(player), y: 750.0};
    canvas.draw(&player_phase_text, DrawParam::default().dest(text_position));
}

fn calculate_x_offset_for_player(player: Player) -> f32 {
    if player == Player::White { 100.0} else { SCREEN_SIZE.0 - 250.0 }
}


fn draw_piece_on_board_for_player(state: &State, canvas: &mut Canvas, player: Player, position: u8) {
    let i = (position % 24) as usize;
    let (x, y) = state.board[i];

    match player {
        Player::White => {
            canvas.draw(&state.white_stone_image, stone_param(x, y, &state.white_stone_image));
        }
        Player::Black => {
            canvas.draw(&state.black_stone_image, stone_param(x, y, &state.black_stone_image));
        }
    }
}

fn stone_param(x: f32, y: f32, image: &Image) -> DrawParam {
    let width = image.width() as f32 * 0.15;
    let height = image.height() as f32 * 0.15;
    DrawParam::new()
        .dest(Point2 {x: x - width / 2f32, y: y - height / 2f32})
        .scale(Vec2 {x: 0.15 , y: 0.15})
}

fn draw_background(ctx: &mut Context, canvas: &mut Canvas) {
    let top_color = Color::from_rgb(
        BACKGROUND_COLOR_TOP_RGB.0,
        BACKGROUND_COLOR_TOP_RGB.1,
        BACKGROUND_COLOR_TOP_RGB.2,
    );

    let bottom_color = Color::from_rgb(
        BACKGROUND_COLOR_BOTTOM_RGB.0,
        BACKGROUND_COLOR_BOTTOM_RGB.1,
        BACKGROUND_COLOR_BOTTOM_RGB.2,
    );

    let mesh = Mesh::new_rectangle(
        ctx,
        DrawMode::fill(),
        Rect::new(0.0, 0.0, SCREEN_SIZE.0, SCREEN_SIZE.1),
        top_color
    ).unwrap();
    canvas.draw(&mesh, DrawParam::default());

    for i in 0..50 {
        let t = i as f32 / 50f32;
        let r = top_color.r + t * (bottom_color.r - top_color.r);
        let g = top_color.g + t * (bottom_color.g - top_color.g);
        let b = top_color.b + t * (bottom_color.b - top_color.b);

        let color = Color::new(r, g, b, 1.0);
        let rect = Mesh::new_rectangle(
            ctx,
            DrawMode::fill(),
            Rect::new(0.0, SCREEN_SIZE.1 * t, SCREEN_SIZE.0, SCREEN_SIZE.1 / 50f32),
            color
        ).unwrap();
        canvas.draw(&rect, DrawParam::default());
    }
}

fn draw_board(state: &mut State, ctx: &mut Context, canvas: &mut Canvas) {
    for (x, y) in state.board {
        let circle = generate_point(ctx, x, y, POINT_SIZE);
        canvas.draw(&circle, DrawParam::default());
    }

    for circle in 0..3 {
        let (x, y) = state.board[circle * 8 + 7];
        let (w, h) = state.board[circle * 8 + 3];
        let mesh = generate_board_rectangle(ctx, x, y, w - x, h - y);
        canvas.draw(&mesh, DrawParam::default());
    }

    for i in 0..4 {
        let point_pos = 2 * i;
        let (x0, y0) = state.board[point_pos];
        let (x1, y1) = state.board[point_pos + (2 * 8)];
        let points = [
            Vec2::new(x0, y0),
            Vec2::new(x1, y1),
        ];
        let line = generate_connecting_line(ctx, &points);
        canvas.draw(&line, DrawParam::default());
    }

}

fn generate_connecting_line(ctx: &mut Context, points: &[Vec2; 2]) -> Mesh {
    Mesh::new_line(
        ctx,
        points,
        5.0,
        Color::WHITE
    ).unwrap()
}

fn generate_board_rectangle(ctx: &mut Context, x: f32, y: f32, w: f32, h: f32) -> Mesh {
    Mesh::new_rectangle(
        ctx,
        DrawMode::stroke(5.0),
        Rect::new(x, y, w, h),
        Color::WHITE
    ).unwrap()
}

fn generate_board() -> [(f32, f32); 24] {
    let mut arr = [(0.0, 0.0); 24];
    // generate all 8 points for each square
    fn square_helper(radius: f32) -> [(f32, f32); 8] {
        [
            (0.0, -radius), // Mitte oben
            (radius, -radius), // Rechts oben
            (radius, 0.0), // Mitte rechts ...
            (radius, radius),
            (0.0, radius),
            (-radius, radius),
            (-radius, 0.0),
            (-radius, -radius)
        ]
    }

    let outside = square_helper(BOARD_CIRCLES.0);
    let middle = square_helper(BOARD_CIRCLES.1);
    let inside = square_helper(BOARD_CIRCLES.2);

    for i in 0..8 {
        arr[i] = (CENTER.0 + outside[i].0, CENTER.1 + outside[i].1);
    }
    for i in 0..8 {
        arr[i + 8] = (CENTER.0 + middle[i].0, CENTER.1 + middle[i].1);
    }
    for i in 0..8 {
        arr[i + 16] = (CENTER.0 + inside[i].0, CENTER.1 + inside[i].1);
    }

    arr
}

fn generate_point(ctx: &mut Context, x: f32, y: f32, radius: f32) -> Mesh {
    Mesh::new_circle(
        ctx,
        DrawMode::fill(),
        Point2 {x, y},
        radius,
        0.1,
        Color::WHITE
    ).unwrap()
}