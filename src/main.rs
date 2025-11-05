use ggez::*;
use ggez::conf::{WindowMode, WindowSetup};
use ggez::glam::Vec2;
use ggez::graphics::{Canvas, Color, DrawMode, DrawParam, Image, Mesh, Rect};
use ggez::mint::Point2;
use muehle_game::{Player, Position};

// pixel size of a point
const POINT_SIZE: f32 = 10.0;
const SCREEN_SIZE: (f32, f32) = (1200.0, 1000.0);
const CENTER: (f32, f32) = (SCREEN_SIZE.0 / 2.0, SCREEN_SIZE.1 / 2.0);
const BOARD_CIRCLES: (f32, f32, f32) = (300.0, 200.0, 100.0);

const BACKGROUND_COLOR_TOP_RGB: (u8, u8, u8) = (243, 186, 126);
const BACKGROUND_COLOR_BOTTOM_RGB: (u8, u8, u8) = (209, 129, 61);

const STONES_PATH: &str = "/stones.png";

struct State {
    position: Position,
    board: [(f32, f32); 24],
    game_timer: i64,

    stone_mesh: Image,
    white_stone_atlas: Rect,
    black_stone_atlas: Rect
}

impl State {
    fn new(ctx: &mut Context) -> State {
        let default_position = Position::new(Player::White);
        State {
            position: default_position,
            board: generate_board(),
            game_timer: 0,
            stone_mesh: Image::from_path(ctx, STONES_PATH).unwrap(),
            white_stone_atlas: Rect::new(0.0, 0.0, 0.5, 1.0),
            black_stone_atlas: Rect::new(0.5, 0.0, 0.5, 1.0),
        }
    }
}

impl event::EventHandler for State {
    fn update(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        Ok(())
    }

    fn draw(&mut self, _ctx: &mut Context) -> Result<(), GameError> {
        let mut canvas = Canvas::from_frame(_ctx, Color::WHITE);

        draw_background(_ctx, &mut canvas);
        draw_board(self, _ctx, &mut canvas);

        draw_stone_for_player(&mut canvas, self, Player::White);
        canvas.finish(_ctx)?;
        Ok(())
    }
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

fn draw_stone_for_player(canvas: &mut Canvas, state: &mut State, player: Player) {
    match player {
        Player::White => {
            let white_stone = DrawParam::new()
                .src(state.white_stone_atlas)
                .dest(Point2 { x: SCREEN_SIZE.0 / 2.0, y: SCREEN_SIZE.1 / 2.0 })
                .scale(Vec2::new(40.0, 40.0));
            canvas.draw(&state.stone_mesh, white_stone);
        }
        Player::Black => {
            let black_stone = DrawParam::new()
                .src(state.black_stone_atlas)
                .dest(Point2 { x: SCREEN_SIZE.0 / 2.0, y: SCREEN_SIZE.1 / 2.0 })
                .scale(Vec2::new(40.0, 40.0));
            canvas.draw(&state.stone_mesh, black_stone);
        }
    }
}

fn draw_board(state: &mut State, ctx: &mut Context, canvas: &mut Canvas) {
    for (x, y) in state.board {
        let circle = generate_point(ctx, x, y, POINT_SIZE);
        canvas.draw(&circle, DrawParam::default());
    }

    for circle in 0..3 {
        let (x, y) = state.board[circle * 8];
        let (w, h) = state.board[circle * 8 + 4];
        let mesh = generate_board_rectangle(ctx, x, y, w - x, h - y);
        canvas.draw(&mesh, DrawParam::default());
    }

    for i in 0..4 {
        let point_pos = 2 * i + 1;
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
            (-radius, -radius), // links oben
            (0.0, -radius), // Mitte oben
            (radius, -radius), // Rechts oben
            (radius, 0.0), // Mitte rechts ...
            (radius, radius),
            (0.0, radius),
            (-radius, radius),
            (-radius, 0.0)]
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
    let (mut ctx, event_loop) = ContextBuilder::new("Nine Men's Morris", "Felix")
        .default_conf(config)
        .add_resource_path("./resources")
        .window_setup(WindowSetup::default().title("Nine Men's Morris"))
        .window_mode(WindowMode::default().dimensions(SCREEN_SIZE.0, SCREEN_SIZE.1))
        .build()?;

    let state = State::new(&mut ctx);

    event::run(ctx, event_loop, state);
}
