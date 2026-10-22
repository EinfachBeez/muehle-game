use ggez::{conf, event, ContextBuilder, GameResult};
use ggez::conf::{WindowMode, WindowSetup};
use crate::ui::ui::State;

mod agent_helper;
mod ui;

#[cfg(test)]
mod test;

const GAME_NAME: &str = "Nine Men's Morris";
const GAME_AUTHOR: &str = "Felix";
const BOARD_APP_IMAGE: &str = "/board_app_image.jpg";
const SCREEN_SIZE: (f32, f32) = (1500.0, 900.0);

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
