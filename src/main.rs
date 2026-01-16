#![allow(dead_code)]
use macroquad::prelude::*;

const MAP: usize = 20;
const T_SIZE: (f32, f32) = (32., 16.);

enum AppState {
    Menu,
    Playing,
    GameOver,
}

#[derive(Copy, Clone, PartialEq)]
enum Tile {
    Wall,
    Floor,
}

//Math Helper
fn to_screen(x: usize, y: usize, cam: (f32, f32)) -> (f32, f32) {
    (
        (x as f32 - y as f32) * T_SIZE.0 + cam.0,
        (x as f32 + y as f32) * T_SIZE.1 + cam.1,
    )
}

fn draw_wall(x: usize, y: usize, cam: (f32, f32)) {
    let (sx, sy) = to_screen(x, y, cam);

    let v = [
        vec2(sx, sy - 40.),
        vec2(sx + 32., sy - 24.),
        vec2(sx, sy - 8.),
        vec2(sx - 32., sy - 24.),
        vec2(sx + 32., sy),
        vec2(sx, sy + 16.),
        vec2(sx - 32., sy),
    ];

    let colors = [
        Color::new(0.8, 0.8, 0.8, 1.),
        Color::new(0.5, 0.5, 0.5, 1.),
        Color::new(0.6, 0.6, 0.6, 1.),
    ];

    //draw faces
    draw_triangle(v[0], v[1], v[2], colors[0]);
    draw_triangle(v[0], v[2], v[3], colors[0]);
    draw_triangle(v[1], v[4], v[5], colors[1]);
    draw_triangle(v[1], v[5], v[2], colors[1]);
    draw_triangle(v[3], v[2], v[5], colors[2]);
    draw_triangle(v[3], v[5], v[6], colors[2]);

    //draw outline
    for (a, b) in [(0, 1), (1, 2), (2, 3), (3, 0), (1, 4), (2, 5), (3, 6)] {
        draw_line(v[a].x, v[a].y, v[b].x, v[b].y, 1., BLACK);
    }
}

struct Game {
    map: [[Tile;MAP];MAP],
    cam: (f32, f32)
}

impl Game {
    fn new() -> Self {
        let mut map = [[Tile::Floor;MAP];MAP];

        for i in 0..MAP {
            map[0][i] = Tile::Wall;
            map[MAP -1][i] = Tile::Wall;
            map[i][0] = Tile::Wall;
            map[i][MAP -1] = Tile::Wall;
        }

        //add obstacles
        for (x, y) in [(5,5), (6,5), (12,10)] {
            map[y][x] = Tile::Wall;
        }

        Game {
            map,
            cam: (screen_width() /2., 50.)
        }
    }

    fn update(&mut self, _dt: f32) -> bool {
        //fake gaming logic
        if is_key_pressed(KeyCode::Space) {
            return true;
        }
        false
    }

    fn draw(&self) {
        for y in 0..MAP {
            for x in 0..MAP {
                if self.map[y][x] == Tile::Wall{
                    draw_wall(x, y, self.cam);
                } else {
                    let (sx, sy) = to_screen(x, y, self.cam);
                    draw_circle(sx, sy + 16., 2., LIGHTGRAY);
                }
            }
        }
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
