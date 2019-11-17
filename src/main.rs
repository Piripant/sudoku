use std::collections::{HashMap, HashSet};
use sudoku::Table;

const SCALE: f32 = 50.0;
const OFFSET: f32 = 10.0;

use ggez::event::*;
use ggez::nalgebra as na;
use ggez::*;


// TABLE LOGIC
struct GameState {
    table: Table,
    // tiles which we are sure are correct
    correct: HashSet<usize>,
    // tiles we placed down which we are uncertain are correct
    uncertain: HashMap<usize, u8>,
    // a tile that is blocking the value we are trying to place
    blocking: Option<(usize, usize)>,
    // the selected value to place
    selected: u8,
}

impl GameState {
    fn new() -> GameResult<GameState> {
        let mut game = GameState {
            table: Table::new(3),
            correct: HashSet::new(),
            uncertain: HashMap::new(),
            blocking: None,
            selected: 0,
        };
        game.reset();

        Ok(game)
    }

    fn reset(&mut self) {
        self.table.clear();
        self.table.fill(0);
        self.table.unsolve();

        // Set the correct tiles to the ones the algorithm has left in
        self.correct.clear();
        for i in 0..self.table.side * self.table.side {
            if self.table.grid[i as usize] != 0 {
                self.correct.insert(i);
            }
        }
    }

    fn nowhere_else(&self, index: usize, target: u8, indexes: impl Iterator<Item = usize>) -> bool {
        // No other tile in `indexes` can assume this value
        // Because it is already present in their neighborhood (column + row + quadrant)
        // We filter out the correct tiles because we already know they can only assume
        // a certain value, so they will never be able to be == target
        indexes
            .filter(|i| !(self.correct.contains(i) || *i == index))
            .map(|i| {
                let (x, y) = self.table.position(i);
                self.table
                    .neighborhood(x, y)
                    .any(|i| self.table.grid[i] == target)
            })
            .all(|found| found)
    }

    fn place_tile(&mut self, index: usize, value: u8) {
        // We aren't overriding a correct tile
        assert!(self.correct.contains(index));

        self.table.grid[index] = value;
        if value == 0 {
            self.uncertain.remove(&index);
        } else {
            self.uncertain.insert(&index);
        }

        self.update_correct();
    }

    fn update_correct(&mut self) {
        // Remove all the uncertain values
        for index in self.uncertain.keys() {
            self.table.grid[*index] = 0;
        }

        // Do this each time a new certain value is added
        let mut changed = true;
        while changed {
            let mut changed_index = 0;
            changed = false;

            // Now consider if each value is correct based only
            // on the values we are sure are correct (so no guessed the user made mess us up)
            for (index, value) in &self.uncertain {
                let (x, y) = self.table.position(*index);
                changed_index = *index;

                let valid = self.table.valid(*index);
                if valid.len() == 1 && valid.contains(value) {
                    changed = true;
                    break;
                }

                // The value which none of the others in the quadrant can assume
                if self.nowhere_else(*index, *value, self.table.quadrant(x, y)) {
                    changed = true;
                    break;
                }

                // The value which none of the others in the row can assume
                if self.nowhere_else(*index, *value, self.table.row(y)) {
                    changed = true;
                    break;
                }

                // The value which none of the others in the column can assume
                if self.nowhere_else(*index, *value, self.table.column(x)) {
                    changed = true;
                    break;
                }
            }

            if changed {
                self.correct.insert(changed_index);
                let value = self.uncertain.remove(&changed_index).unwrap();
                self.table.grid[changed_index] = value;
            }
        }

        // Reset the grid back to the original
        for (index, value) in &self.uncertain {
            self.table.grid[*index] = *value;
        }

        // All cells are correct
        if self.correct.len() == self.table.side * self.table.side {
            self.reset();
        }
    }

    fn step(&mut self) {
        // Remove all the not correct tiles
        // So we can calculate the next step basing our assumption
        // on only correct cells
        for index in self.uncertain.keys() {
            self.table.grid[*index] = 0;
        }

        let mut holes = HashSet::new();
        for i in 0..self.table.side * self.table.side {
            if self.table.grid[i] == 0 {
                holes.insert(i);
            }
        }

        if let Some((index, value)) = self.table.obvious_move(&holes) {
            self.table.grid[index] = value;
            self.correct.insert(index);
            self.uncertain.remove(&index);
        }

        // Check if our uncertain values are now certain or invalid
        let mut to_remove = Vec::new();
        for (index, value) in &self.uncertain {
            if self.table.valid(*index).contains(&value) {
                self.table.grid[*index] = *value;
            } else {
                to_remove.push(*index);
            }
        }
        for index in to_remove {
            self.uncertain.remove(&index);
        }

        self.update_correct();
    }
}

// RENDERING
fn to_screen(x: usize, y: usize) -> na::Point2<f32> {
    na::Point2::new(x as f32 * SCALE + OFFSET, y as f32 * SCALE + OFFSET)
}

fn to_world(x: f32, y: f32) -> (usize, usize) {
    (
        ((x - OFFSET) / SCALE) as usize,
        ((y - OFFSET) / SCALE) as usize,
    )
}

impl event::EventHandler for GameState {
    fn update(&mut self, _ctx: &mut ggez::Context) -> ggez::GameResult {
        Ok(())
    }

    fn draw(&mut self, ctx: &mut ggez::Context) -> ggez::GameResult {
        graphics::clear(ctx, [1.0, 1.0, 1.0, 1.0].into());

        // Vertical lines
        for x in 0..=self.table.side {
            let thick = if x % self.table.quadrant_side == 0 {
                3.0
            } else {
                2.0
            };

            let start = to_screen(x, 0);
            let end = to_screen(x, self.table.side);
            let circle = graphics::Mesh::new_line(ctx, &[start, end], thick, graphics::BLACK)?;
            graphics::draw(ctx, &circle, (na::Point2::new(0.0, 0.0),))?;
        }

        // Horizontal lines
        for y in 0..=self.table.side {
            let thick = if y % self.table.quadrant_side == 0 {
                3.0
            } else {
                2.0
            };

            let start = to_screen(0, y);
            let end = to_screen(self.table.side, y);
            let circle = graphics::Mesh::new_line(ctx, &[start, end], thick, graphics::BLACK)?;
            graphics::draw(ctx, &circle, (na::Point2::new(0.0, 0.0),))?;
        }

        let font = graphics::Font::default();
        for x in 0..self.table.side {
            for y in 0..self.table.side {
                let index = self.table.index(x, y);
                let value = self.table.grid[index];

                if value == 0 {
                    continue;
                }

                let mut text = graphics::Text::new(value.to_string());
                text.set_font(font, graphics::Scale::uniform(40.0));
                text.set_bounds(na::Point2::new(SCALE, SCALE), graphics::Align::Center);

                let mut position = to_screen(x, y);
                position.y += (SCALE - text.height(ctx) as f32) / 2.0;

                let color = if self.correct.contains(&index) {
                    (0.1, 0.8, 0.1).into()
                } else {
                    graphics::BLACK
                };

                graphics::draw(
                    ctx,
                    &text,
                    graphics::DrawParam::default().dest(position).color(color),
                )?;
            }
        }

        if let Some(blocking) = self.blocking {
            let position = to_screen(blocking.0, blocking.1);
            let rect = graphics::Mesh::new_rectangle(
                ctx,
                graphics::DrawMode::fill(),
                graphics::Rect::new(position.x, position.y, SCALE, SCALE),
                [0.8, 0.2, 0.2, 0.4].into(),
            )
            .unwrap();
            graphics::draw(ctx, &rect, graphics::DrawParam::default())?;
        }

        graphics::present(ctx)?;
        Ok(())
    }

    fn key_down_event(
        &mut self,
        _ctx: &mut Context,
        keycode: KeyCode,
        _keymod: KeyMods,
        _repeat: bool,
    ) {
        match keycode {
            KeyCode::Key1 => self.selected = 1,
            KeyCode::Key2 => self.selected = 2,
            KeyCode::Key3 => self.selected = 3,
            KeyCode::Key4 => self.selected = 4,
            KeyCode::Key5 => self.selected = 5,
            KeyCode::Key6 => self.selected = 6,
            KeyCode::Key7 => self.selected = 7,
            KeyCode::Key8 => self.selected = 8,
            KeyCode::Key9 => self.selected = 9,
            KeyCode::Tab => self.step(),
            _ => {}
        }

        self.selected = u8::min(self.selected, self.table.side as u8);
    }

    fn mouse_button_down_event(&mut self, _ctx: &mut Context, button: MouseButton, x: f32, y: f32) {
        let (grid_x, grid_y) = to_world(x, y);
        if grid_x >= self.table.side || grid_y >= self.table.side {
            return;
        }

        let index = self.table.index(grid_x, grid_y);
        match button {
            MouseButton::Left => {
                let valid = self.table.valid(index);
                if valid.contains(&self.selected) {
                    // Dont override correct tiles
                    if !self.correct.contains(&index) {
                        self.place_tile(index, self.selected);
                    }
                } else {
                    // Let's find what tile makes it invalid
                    let blocking_index = self
                        .table
                        .neighborhood(grid_x, grid_y)
                        .find(|index| self.table.grid[*index] == self.selected)
                        .unwrap();

                    self.blocking = Some(self.table.position(blocking_index));
                }
            }
            MouseButton::Right => {
                // Only remove uncertain tiles
                if self.uncertain.contains_key(&index) {
                    self.place_tile(index, 0);
                }
            }
            _ => {}
        }
    }

    fn resize_event(&mut self, ctx: &mut Context, width: f32, height: f32) {
        graphics::set_screen_coordinates(ctx, [0.0, 0.0, width, height].into()).unwrap();
    }
}

pub fn main() -> ggez::GameResult {
    let cb = ggez::ContextBuilder::new("sudoku", "piripant")
        .window_setup(ggez::conf::WindowSetup::default().title("sudoku"))
        .window_mode(ggez::conf::WindowMode::default().resizable(true));
    let (ctx, event_loop) = &mut cb.build()?;
    let state = &mut GameState::new()?;
    event::run(ctx, event_loop, state)
}
