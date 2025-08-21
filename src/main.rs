#[derive(Debug, serde::Serialize)]
struct GameStats {
    turns: usize,
    hits: usize,
    misses: usize,
    ships_left: usize,
    total_ships: usize,
}

use rand::seq::IndexedRandom;

use eframe::egui;
use rand::seq::SliceRandom;
use rand::Rng;

const SHIPS: [(&str, usize); 3] = [("Destroyer", 2), ("Cruiser", 3), ("Battleship", 4)];

#[derive(Clone, Copy, PartialEq)]
enum Cell {
    Empty,
    Ship(usize), // ship index
}

#[derive(Clone, Copy, PartialEq)]
enum Shot {
    Untargeted,
    Miss,
    Hit,
}

#[derive(Clone)]
struct Ship {
    name: &'static str,
    length: usize,
    positions: Vec<(usize, usize)>,
    sunk: bool,
}

struct BattleshipGame {
    board: Vec<Vec<Cell>>,
    shots: Vec<Vec<Shot>>,
    ships: Vec<Ship>,
    game_over: bool,
    message: String,
    play_again: bool,
    rows: usize,
    cols: usize,
    turns: usize,
}

impl BattleshipGame {
    fn new(rows: usize, cols: usize) -> Self {
        let mut game = BattleshipGame {
            board: vec![vec![Cell::Empty; cols]; rows],
            shots: vec![vec![Shot::Untargeted; cols]; rows],
            ships: vec![],
            game_over: false,
            message: "Welcome to Battleship!".to_owned(),
            play_again: false,
            rows,
            cols,
            turns: 0,
        };
        game.start_game(rows, cols);
        game
    }

    fn start_game(&mut self, rows: usize, cols: usize) {
        self.turns = 0;
        self.rows = rows;
        self.cols = cols;
        self.board = vec![vec![Cell::Empty; cols]; rows];
        self.shots = vec![vec![Shot::Untargeted; cols]; rows];
        self.ships.clear();
        self.game_over = false;
        self.message = "Game started!".to_owned();
        self.play_again = false;
        let mut rng = rand::thread_rng();
        for (ship_idx, (name, length)) in SHIPS.iter().enumerate() {
            'place: loop {
                let dir = *[true, false].choose(&mut rng).unwrap(); // true: horizontal, false: vertical
                let (row, col) = (
                    rng.gen_range(0..rows),
                    rng.gen_range(0..cols),
                );
                let mut positions = vec![];
                for i in 0..*length {
                    let (r, c) = if dir {
                        (row, col + i)
                    } else {
                        (row + i, col)
                    };
                    if r >= rows || c >= cols {
                        continue 'place;
                    }
                    if self.board[r][c] != Cell::Empty {
                        continue 'place;
                    }
                    positions.push((r, c));
                }
                for &(r, c) in &positions {
                    self.board[r][c] = Cell::Ship(ship_idx);
                }
                self.ships.push(Ship {
                    name,
                    length: *length,
                    positions,
                    sunk: false,
                });
                break;
            }
        }
    }

    fn shoot(&mut self, row: usize, col: usize) {
        self.turns += 1;
        if self.game_over || self.shots[row][col] != Shot::Untargeted {
            return;
        }
        match self.board[row][col] {
            Cell::Empty => {
                self.shots[row][col] = Shot::Miss;
                self.message = format!("Miss at ({}, {})!", row + 1, col + 1);
            }
            Cell::Ship(idx) => {
                self.shots[row][col] = Shot::Hit;
                self.message = format!("Hit at ({}, {})!", row + 1, col + 1);
                // Check if ship is sunk
                let ship = &mut self.ships[idx];
                if ship.positions.iter().all(|&(r, c)| self.shots[r][c] == Shot::Hit) {
                    ship.sunk = true;
                    self.message = format!("You sunk the {}!", ship.name);
                }
            }
        }
        let ships_left = self.ships.iter().filter(|s| !s.sunk).count();
        if ships_left == 0 {
            self.game_over = true;
            self.message = "Congratulations! You sunk all the ships!".to_owned();
            self.play_again = true;
        }
    }

    fn game_stats(&self) -> GameStats {
        let mut hits = 0;
        let mut misses = 0;
        for row in &self.shots {
            for &cell in row {
                match cell {
                    Shot::Hit => hits += 1,
                    Shot::Miss => misses += 1,
                    _ => {}
                }
            }
        }
        GameStats {
            turns: self.turns,
            hits,
            misses,
            ships_left: self.ships.iter().filter(|s| !s.sunk).count(),
            total_ships: self.ships.len(),
        }
    }
}

struct BattleshipApp {
    game: BattleshipGame,
    selected_row: usize,
    selected_col: usize,
    input_rows: usize,
    input_cols: usize,
    awaiting_new_game: bool,
}

impl Default for BattleshipApp {
    fn default() -> Self {
        let default_rows = 8;
        let default_cols = 8;
        Self {
            game: BattleshipGame::new(default_rows, default_cols),
            selected_row: 0,
            selected_col: 0,
            input_rows: default_rows,
            input_cols: default_cols,
            awaiting_new_game: false,
        }
    }
}

impl eframe::App for BattleshipApp {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {
            ui.heading("Battleship Game");
            ui.label(&self.game.message);
            ui.separator();

            // Always show board size controls and Start New Game button
            ui.horizontal(|ui| {
                ui.label("Rows:");
                ui.add(egui::DragValue::new(&mut self.input_rows).clamp_range(4..=20));
                ui.label("Columns:");
                ui.add(egui::DragValue::new(&mut self.input_cols).clamp_range(4..=20));
                if ui.button("Start New Game").clicked() {
                    self.game = BattleshipGame::new(self.input_rows, self.input_cols);
                    self.awaiting_new_game = false;
                }
            });
            ui.separator();

            // Board size controls and Start New Game button
            ui.separator();
            // Show game stats
            let stats = self.game.game_stats();
            ui.label(format!("Turns: {} | Hits: {} | Misses: {} | Ships left: {}/{}", stats.turns, stats.hits, stats.misses, stats.ships_left, stats.total_ships));
            ui.separator();
            // Board display
            egui::Grid::new("board_grid").spacing([8.0, 8.0]).show(ui, |ui| {
                ui.label("");
                for col in 0..self.game.cols {
                    ui.label(format!("{}", col + 1));
                }
                ui.end_row();
                for row in 0..self.game.rows {
                    ui.label(format!("{}", row + 1));
                    for col in 0..self.game.cols {
                        let ch = match self.game.shots[row][col] {
                            Shot::Untargeted => " ",
                            Shot::Miss => "O",
                            Shot::Hit => "X",
                        };
                        let color = match self.game.shots[row][col] {
                            Shot::Hit => egui::Color32::RED,
                            Shot::Miss => egui::Color32::LIGHT_BLUE,
                            Shot::Untargeted => egui::Color32::GRAY,
                        };
                        let button = egui::Button::new(ch).fill(color);
                        if ui.add(button).clicked() && !self.game.game_over {
                            self.selected_row = row;
                            self.selected_col = col;
                            self.game.shoot(row, col);
                        }
                    }
                    ui.end_row();
                }
            });

            ui.separator();
            if self.game.game_over {
                if ui.button("Play Again").clicked() {
                    self.awaiting_new_game = true;
                }
            }
        });
    }
}

fn main() {
    let options = eframe::NativeOptions::default();
    eframe::run_native(
        "Battleship Game",
        options,
        Box::new(|_cc| Ok(Box::new(BattleshipApp::default()))),
    ).unwrap();
}
