#![allow(dead_code)]
use macroquad::prelude::*;

const MAP: usize = 20;
const T_SIZE: (f32, f32) = (32., 16.);

enum AppState {
    Menu,
    Playing,
    GameOver,
}

struct Game {}

impl Game {
    fn new() -> Self {
        Game {}
    }

    fn update(&mut self, _dt: f32) -> bool {
        //fake gaming logic
        if is_key_pressed(KeyCode::Space) {
            return true;
        }
        false
    }

    fn draw(&self) {
        draw_text("Game Running...", 20., 40., 30., BLACK);
        draw_text("Press Space to die...", 20., 80., 20., DARKGRAY);
    }
}

#[macroquad::main("Crablo")]
async fn main() {
    let mut game = Game::new();
    let mut state = AppState::Menu;

    loop {
        clear_background(WHITE);

        match state {
            AppState::Menu => {
                draw_text("Menu - Enter to start", 100., 100., 40., BLACK);
                if is_key_pressed(KeyCode::Enter) {
                    game = Game::new();
                    state = AppState::Playing;
                }
            }
            AppState::Playing => {
                if game.update(get_frame_time()) {
                    state = AppState::GameOver;
                }
                game.draw();
            }
            AppState::GameOver => {
                game.draw();
                draw_rectangle(
                    0.,
                    0.,
                    screen_width(),
                    screen_height(),
                    Color::new(1., 1., 1., 0.7),
                );
                draw_text("GAME OVER", 100., 100., 60., RED);
                draw_text("Enter to reset", 100., 150., 20., GRAY);

                if is_key_pressed(KeyCode::Enter) {
                    state = AppState::Menu;
                }
            }
        }
        next_frame().await;
    }
}
