use ggez::conf::{WindowMode, WindowSetup};
use ggez::glam::Vec2;
use ggez::graphics::{Canvas, Color, DrawMode, DrawParam, Drawable, Image, Mesh, PxScale, Rect, Text};
use ggez::mint::Point2;
use ggez::*;
use ggez::input::keyboard::{KeyCode, KeyInput};
use muehle_game::{Player, Position};

// pixel size of a point
const POINT_SIZE: f32 = 10.0;
const SCREEN_SIZE: (f32, f32) = (1500.0, 900.0);
const CENTER: (f32, f32) = (SCREEN_SIZE.0 / 2.0, SCREEN_SIZE.1 / 2.0);
const BOARD_CIRCLES: (f32, f32, f32) = (300.0, 200.0, 100.0);

const BACKGROUND_COLOR_TOP_RGB: (u8, u8, u8) = (243, 186, 126);
const BACKGROUND_COLOR_BOTTOM_RGB: (u8, u8, u8) = (209, 129, 61);

const GAME_NAME: &str = "Nine Men's Morris";
const GAME_AUTHOR: &str = "Felix";
const WHITE_STONES_PATH: &str = "/white_piece.png";
const BLACK_PIECE_PATH: &str = "/black_piece.png";
const BOARD_APP_IMAGE: &str = "/board_app_image.jpg";

#[derive(Copy, Clone, Eq, PartialEq)]
enum GuiPhase {
    INITIAL,
    ACTIVE,
    FINISHED
}

struct State {
    position: Position,
    board: [(f32, f32); 24],
    gui_phase: GuiPhase,
    game_timer: u32,

    white_stone_image: Image,
    black_stone_image: Image
}

impl State {
    fn new(ctx: &mut Context) -> State {
        let default_position = Position::new(Player::White);
        State {
            position: default_position,
            board: generate_board(),
            gui_phase: GuiPhase::INITIAL,
            game_timer: 0,
            white_stone_image: Image::from_path(ctx, WHITE_STONES_PATH).unwrap(),
            black_stone_image: Image::from_path(ctx, BLACK_PIECE_PATH).unwrap()
        }
    }

    fn start_game(&mut self) {
        self.gui_phase = GuiPhase::ACTIVE;
    }

    fn stop_game(&mut self) {
        self.gui_phase = GuiPhase::FINISHED;
    }
}

impl event::EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        if self.gui_phase == GuiPhase::ACTIVE {
            let elapsed = _ctx.time.time_since_start().as_secs_f32();
            self.game_timer = elapsed.floor() as u32;
        }

        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = Canvas::from_frame(_ctx, Color::WHITE);

        draw_background(_ctx, &mut canvas);
        draw_board(self, _ctx, &mut canvas);

        draw_player_information(self, _ctx, &mut canvas);
        draw_player_information(self, _ctx, &mut canvas);

        draw_stone_on_board_for_player(self, &mut canvas, Player::White, self.game_timer);
        draw_stone_on_board_for_player(self, &mut canvas, Player::Black, self.game_timer + 1);

        draw_timer(self, _ctx, &mut canvas);
        canvas.finish(_ctx)?;
        Ok(())
    }

    fn key_down_event(&mut self, ctx: &mut Context, input: KeyInput, _repeated: bool) -> Result<(), GameError> {
        match input.keycode {
            Some(KeyCode::Space) => {
                self.start_game();
            }
            Some(KeyCode::X) => {
                self.stop_game();
            }
            _ => ()
        }
        Ok(())
    }
}

fn draw_timer(state: &mut State, ctx: &mut Context, canvas: &mut Canvas) {
    let mut timer_text;
    let minutes = state.game_timer / 60;
    let seconds = state.game_timer % 60;
    let mut param = DrawParam::default();
    match state.gui_phase {
        GuiPhase::INITIAL => {
            timer_text = Text::new("Timer stopped");
            param = param.color(Color::RED);
        }
        GuiPhase::ACTIVE => {
            timer_text = Text::new(format!("Time elapsed: {:02}:{:02}", minutes, seconds));
            param = param.color(Color::WHITE);
        }
        GuiPhase::FINISHED => {
            timer_text = Text::new(format!("Time spent: {:02}:{:02}", minutes, seconds));
            param = param.color(Color::GREEN);
        }
    }
    timer_text.set_scale(PxScale::from(32.0));
    let width = timer_text.dimensions(&ctx.gfx).unwrap().w;
    let dest = Point2 { x: CENTER.0 - width / 2f32, y: SCREEN_SIZE.1 * 0.05 };
    canvas.draw(&timer_text, param.dest(dest));
}

fn draw_player_information(state: &mut State, ctx: &mut Context, canvas: &mut Canvas) {
    let (white_pieces_to_place, black_pieces_to_place) = state.position.pieces_to_place();
    let active_player= state.position.active_player();

    fn generate_information(state: &mut State, ctx: &mut Context, canvas: &mut Canvas, player: Player, x_offset: f32, pieces_to_place: u8, active_player: Player) {
        let player_name;
        let color;
        let piece_image: &Image;
        match player {
            Player::White => {
                player_name = "WHITE";
                color = Color::WHITE;
                piece_image = &state.white_stone_image;
            }
            Player::Black => {
                player_name = "BLACK";
                color = Color::WHITE;
                piece_image = &state.black_stone_image;
            }
        }

        let mut player_name = Text::new(player_name);
        player_name.set_scale(PxScale::from(40.0));
        let text_height = player_name.dimensions(&ctx.gfx).unwrap().h;
        let text_position = Point2 {x: x_offset, y: 150.0 - text_height};
        let mut param = DrawParam::default().color(color).dest(text_position).color(Color::WHITE);
        if active_player == player {
            param = param.color(Color::GREEN);
        }
        canvas.draw(&player_name, param);

        let mesh = generate_board_rectangle(ctx, 0.0,0.0, 150.0, 600.0);
        canvas.draw(&mesh, DrawParam::default().dest(Point2 {x: x_offset, y: 150.0}));

        for i in 0..pieces_to_place {
            let param = stone_param(x_offset + 75.0, 200.0 + (i as f32 * 60.0), piece_image);
            canvas.draw(piece_image, param);
        }



    }

    generate_information(state, ctx, canvas, Player::White, 100.0, white_pieces_to_place, active_player);
    generate_information(state, ctx, canvas, Player::Black, SCREEN_SIZE.0 - 250.0, black_pieces_to_place, active_player);
}

fn draw_stone_on_board_for_player(state: &mut State, canvas: &mut Canvas, player: Player, position: u32) {
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

fn main() -> GameResult<()> {

    let config = conf::Conf::new();
    let (mut ctx, event_loop) = ContextBuilder::new(GAME_NAME, GAME_AUTHOR)
        .default_conf(config)
        .add_resource_path("./resources")
        .window_setup(WindowSetup::default().title("Nine Men's Morris").icon(BOARD_APP_IMAGE))
        .window_mode(WindowMode::default()
            .resizable(false)
            .maximized(false)
            .dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1)
        )
        .build()?;

    let state = State::new(&mut ctx);

    event::run(ctx, event_loop, state);
}
