#![allow(dead_code)]
use macroquad::prelude::*;
use std::collections::VecDeque;

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

//Monsters
struct Monster {
    x: usize,
    y: usize,
    hp: i32,
    cd: f32,
}

//struct for Floating text
struct DmgText {
    x: f32,
    y: f32,
    dmg: i32, // negative dmg will be treated as gold collected
    life: f32,
}

//Math Helper
fn to_screen(x: usize, y: usize, cam: (f32, f32)) -> (f32, f32) {
    (
        (x as f32 - y as f32) * T_SIZE.0 + cam.0,
        (x as f32 + y as f32) * T_SIZE.1 + cam.1,
    )
}

fn to_tile(sx: f32, sy: f32, cam: (f32, f32)) -> (usize, usize) {
    let (ax, ay) = (sx - cam.0, sy - cam.1);
    (
        ((ax / T_SIZE.0 + ay / T_SIZE.1) / 2.) as usize,
        ((ay / T_SIZE.1 - ax / T_SIZE.0) / 2.) as usize,
    )
}

//calculate distance Manhattan distance
fn dist(p1: (usize, usize), p2: (usize, usize)) -> i32 {
    (p1.0 as i32 - p2.0 as i32).abs() + (p1.1 as i32 - p2.1 as i32).abs()
}

//Pathfinding algo
fn bfs(
    map: &[[Tile; MAP]; MAP],
    start: (usize, usize),
    goal: (usize, usize),
) -> Vec<(usize, usize)> {
    let mut q = VecDeque::from([start]);
    let mut visited = [[false; MAP]; MAP];
    visited[start.1][start.0] = true;

    let mut parent = [[None; MAP]; MAP];

    while let Some(curr) = q.pop_front() {
        if curr == goal {
            let mut path = vec![];
            let mut c = goal;
            while c != start {
                path.push(c);
                c = parent[c.1][c.0].unwrap();
            }
            path.reverse();
            return path;
        }

        //check the close
        for (dx, dy) in [(0, -1), (0, 1), (-1, 0), (1, 0)] {
            let (nx, ny) = ((curr.0 as i32 + dx) as usize, (curr.1 as i32 + dy) as usize);

            if nx < MAP && ny < MAP && !visited[ny][nx] && map[ny][nx] == Tile::Floor {
                visited[ny][nx] = true;
                parent[ny][nx] = Some(curr);
                q.push_back((nx, ny));
            }
        }
    }

    vec![]
}

//draw hero and monsters
fn draw_stickman(x: usize, y: usize, cam: (f32, f32), enemy: bool) {
    let (sx, mut sy) = to_screen(x, y, cam);
    sy += 16.;

    //shadow
    draw_ellipse(sx, sy + 3., 10., 5., 0., Color::new(0., 0., 0., 0.2));

    //head
    if enemy {
        draw_line(sx - 5., sy - 32., sx, sy - 30., 2., BLACK);
        draw_line(sx + 5., sy - 32., sx, sy - 30., 2., BLACK);
    } else {
        //player
        draw_circle_lines(sx, sy - 32., 7., 2., BLACK);
    }

    //body and limbs
    for l in [
        [0., -25., 0., -8.],
        [0., -20., -8., -15.],
        [0., -20., 8., -15.],
        [0., -8., -6., 0.],
        [0., -8., 6., 0.],
    ] {
        draw_line(sx + l[0], sy + l[1], sx + l[2], sy + l[3], 2., BLACK);
    }
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
    map: [[Tile; MAP]; MAP],
    cam: (f32, f32),
    px: usize,
    py: usize,
    path: Vec<(usize, usize)>,
    player_cd: f32,
    monsters: Vec<Monster>,
    texts: Vec<DmgText>,
    hp: i32,
    gold: Vec<(usize, usize)>,
    score: i32,
}

impl Game {
    fn new() -> Self {
        let mut map = [[Tile::Floor; MAP]; MAP];

        for i in 0..MAP {
            map[0][i] = Tile::Wall;
            map[MAP - 1][i] = Tile::Wall;
            map[i][0] = Tile::Wall;
            map[i][MAP - 1] = Tile::Wall;
        }

        //add obstacles
        for (x, y) in [(5, 5), (6, 5), (12, 10)] {
            map[y][x] = Tile::Wall;
        }

        Game {
            map,
            cam: (screen_width() / 2., 50.),
            px: 2,
            py: 2,
            path: vec![],
            player_cd: 0.,
            monsters: vec![
                Monster {
                    x: 8,
                    y: 8,
                    hp: 30,
                    cd: 0.,
                },
                Monster {
                    x: 12,
                    y: 4,
                    hp: 30,
                    cd: 0.,
                },
                Monster {
                    x: 15,
                    y: 12,
                    hp: 30,
                    cd: 0.,
                },
            ],
            texts: vec![],
            hp: 100,
            score: 0,
            gold: vec![(3, 3), (10, 2), (16, 5), (6, 14), (17, 17)],
        }
    }

    fn update(&mut self, dt: f32) -> bool {
        //victory/loss condition
        if self.hp <= 0 || self.monsters.is_empty() {
            return true;
        }

        //update text animations
        self.texts.retain_mut(|t| {
            t.life -= dt;
            t.y -= 20. * dt;
            t.life > 0.
        });

        //mouse input logic
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            let (tx, ty) = to_tile(mx, my, self.cam);

            //check if the click is inside the map bounds
            if tx < MAP && ty < MAP && self.map[ty][tx] == Tile::Floor {
                self.path = bfs(&self.map, (self.px, self.py), (tx, ty));
            }
        }

        //handle movement for the player
        if !self.path.is_empty() {
            self.player_cd -= dt;

            //time to move?
            if self.player_cd <= 0. {
                self.player_cd = 0.15;

                let (nx, ny) = self.path[0];

                //Combat logic for the player
                if let Some(i) = self.monsters.iter().position(|m| m.x == nx && m.y == ny) {
                    //attack
                    self.damage_monster(i, 10);
                    //stop moving
                    self.path.clear();
                } else {
                    //move
                    self.path.remove(0);
                    self.px = nx;
                    self.py = ny;

                    //collect gold logic
                    if let Some(i) = self.gold.iter().position(|&g| g == (self.px, self.py)) {
                        self.gold.remove(i);
                        self.score += 100;
                        //spawn a green text
                        let (sx, sy) = to_screen(self.px, self.py, self.cam);
                        self.texts.push(DmgText {
                            x: sx,
                            y: sy - 40.,
                            dmg: -100,
                            life: 1.,
                        })
                    }
                }
            }
        }

        //Monster logic
        //calculate te occupied spots so enemies don't stack
        let occupied: Vec<_> = self
            .monsters
            .iter()
            .map(|m| (m.x, m.y))
            .chain(std::iter::once((self.px, self.py)))
            .collect();

        for i in 0..self.monsters.len() {
            self.monsters[i].cd -= dt;
            if self.monsters[i].cd <= 0. {
                self.monsters[i].cd = 1.0; //slow

                let (mx, my) = (self.monsters[i].x, self.monsters[i].y);
                let d = dist((mx, my), (self.px, self.py));

                if d == 1 {
                    self.hp -= 5;
                    let (sx, sy) = to_screen(self.px, self.py, self.cam);
                    self.texts.push(DmgText {
                        x: sx,
                        y: sy - 40.,
                        dmg: 5,
                        life: 1.,
                    });
                } else {
                    //chase the player
                    let path = bfs(&self.map, (mx, my), (self.px, self.py));
                    if path.len() > 1 && !occupied.contains(&path[0]) {
                        self.monsters[i].x = path[0].0;
                        self.monsters[i].y = path[0].1;
                    }
                }
            }
        }

        false
    }

    //helper to damage monsters
    fn damage_monster(&mut self, idx: usize, amount: i32) {
        self.monsters[idx].hp -= amount;

        //spawn text
        let (sx, sy) = to_screen(self.monsters[idx].x, self.monsters[idx].y, self.cam);
        self.texts.push(DmgText {
            x: sx,
            y: sy - 40.,
            dmg: amount,
            life: 1.,
        });

        //kill logic
        if self.monsters[idx].hp <= 0 {
            self.monsters.remove(idx);
            //+50 bonus if I kill a monster
            self.score += 50;
        }
    }

    fn draw(&self) {
        for y in 0..MAP {
            for x in 0..MAP {
                if self.map[y][x] == Tile::Wall {
                    draw_wall(x, y, self.cam);
                } else {
                    if self.gold.contains(&(x, y)) {
                        let (sx, sy) = to_screen(x, y, self.cam);
                        draw_circle(sx, sy + 16., 6., GOLD);
                    } else {
                        let (sx, sy) = to_screen(x, y, self.cam);
                        draw_circle(sx, sy + 16., 2., LIGHTGRAY);
                    }
                }
            }
        }

        //draw the path
        for (px, py) in &self.path {
            let (sx, sy) = to_screen(*px, *py, self.cam);
            draw_circle(sx, sy + 16., 4., GOLD);
        }

        //draw the player
        draw_stickman(self.px, self.py, self.cam, false);

        //draw monsters
        for m in &self.monsters {
            draw_stickman(m.x, m.y, self.cam, true);
        }

        //draw floating texts
        for t in &self.texts {
            if t.dmg < 0 {
                draw_text(&format!("+{}", -t.dmg), t.x, t.y, 20., GREEN);
            } else {
                draw_text(&format!("-{}", t.dmg), t.x, t.y, 20., RED);
            }
        }

        //HUD
        draw_text(
            &format!("HP: {}", self.hp),
            20.,
            screen_height() - 40.,
            30.,
            BLACK,
        );
        draw_text(
            &format!("SCORE: {}", self.score),
            20.,
            screen_height() - 70.,
            30.,
            BLACK,
        );
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

                //Victory vs Defeat logic
                let (msg, col) = if game.hp > 0 {
                    ("VICTORY", GOLD)
                } else {
                    ("GAME OVER", RED)
                };

                draw_text(
                    msg,
                    screen_width() / 2. - 100.,
                    screen_height() / 2.,
                    60.,
                    col,
                );
                draw_text(
                    &format!("Final Score: {}", game.score),
                    screen_width() / 2. - 80.,
                    screen_height() / 2. + 50.,
                    30.,
                    BLACK,
                );

                draw_text(
                    "Enter to reset",
                    screen_width() / 2. - 80.,
                    screen_height() / 2. + 90.,
                    20.,
                    GRAY,
                );
                if is_key_pressed(KeyCode::Enter) {
                    state = AppState::Menu;
                }
            }
        }
        next_frame().await;
    }
}
