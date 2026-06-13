use macroquad::prelude::*;

mod ui;
mod components;
mod logic;
use ui::*;
use components::*;
use logic::*;

#[macroquad::main("Checkers")]
async fn main() {
    macroquad::rand::srand(miniquad::date::now() as u64);
    let mut app = App::new();
    let mut board = Board::new();

    let mut ai_thinking = false;
    let mut ai_ready_to_compute = false; // New state latch

    loop {
        let mut messages = Vec::new();
        let (height, width) = (screen_height(), screen_width());

        messages.push(render_gui(&app, height, width));
        messages.push(render_board(&app, &board, height, width));

        update(&mut app, &mut board, messages);

        if app.mode == Mode::Human || app.player == Player::None {
            ai_thinking = false;
            ai_ready_to_compute = false;
        }
        if app.player == Player::Ai && !app.won {
            if !ai_thinking {
                    ai_thinking = true;
            } else if !ai_ready_to_compute {
                ai_ready_to_compute = true;
            } else {
                let (next_moves, next_player, _, won, chosen_move) = minimax(&mut board, &app);
                app.legal_moves = next_moves;
                app.player = next_player;
                app.won = won;
                // Update the app visual state so the UI highlights where the AI moved from and to
                app.selected_piece = ShowPiece::Piece(chosen_move.from as usize);
                app.last_piece = ShowPiece::Piece(chosen_move.to as usize);
                ai_thinking = false;
                ai_ready_to_compute = false;
            }
        }
        egui_macroquad::draw();
        next_frame().await;
    }
}
