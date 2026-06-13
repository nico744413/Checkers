use crate::components::*;

pub fn update(app: &mut App, board: &mut Board, messages: Vec<Message>) {
    for message in messages {
        match message {
            Message::AiMode => {if app.player == Player::None {app.reset(); board.reset(); app.mode = Mode::Ai;}},
            Message::HumanMode => {if app.player == Player::None {app.reset(); board.reset(); app.mode = Mode::Human;}},
            Message::StartGame(player) => {
                if app.player == Player::None {
                    app.player = player;
                    app.legal_moves = board.get_legal_moves(&app.player);
                }
            },
            Message::EndGame => {app.reset(); board.reset();},
            Message::Clicked(idx) => {
                let mask = 1 << idx;

                let is_valid_selection = match app.player {
                    Player::White | Player::Human => (board.white_pieces & mask) != 0,
                    Player::Black => (board.black_pieces & mask) != 0,
                    Player::Ai => false,
                    Player::None => false, // no moves allowed if game hasn't started or ended
                };

                if is_valid_selection {
                    app.selected_piece = ShowPiece::Piece(idx);
                    app.dots.clear();
                    for piece in &app.legal_moves {
                        if piece.from as usize == idx {
                            app.dots.push(ShowPiece::Piece(piece.to as usize));
                        }
                    }
                }

                let is_valid_move = app.dots.iter().any(|dot| match dot {
                    ShowPiece::Piece(dot_idx) => *dot_idx == idx,
                    ShowPiece::None => false,
                });

                if is_valid_move {
                    app.last_piece = ShowPiece::Piece(idx);
                    if let ShowPiece::Piece(selected_idx) = app.selected_piece {
                        app.dots.clear();
                        for piece in &app.legal_moves {
                            if piece.to as usize == idx && piece.from as usize == selected_idx {
                                (app.legal_moves, app.player, _, app.won) = board.step(*piece, app.player);
                                break;
                            }
                        }
                    }
                }
            }
            Message::SetDifficulty(depth) => app.depth = depth,
            Message::None => {},
        }
    }
}


#[derive(Clone)]
struct StackFrame {
    board: Board,
    player: Player,
    depth: u32,
    legal_moves: Vec<Piece>,
    move_idx: usize,
    alpha: i32,
    beta: i32,
    best_score: i32,
    best_move: Option<Piece>,
    is_maximizing: bool,
}

pub fn minimax(board: &mut Board, app: &App) -> (Vec<Piece>, Player, i32, bool, Piece) {

    if app.depth == 0 {
        let random_idx = macroquad::rand::gen_range(0, app.legal_moves.len());
        let chosen_move = app.legal_moves[random_idx];
        let (next_moves, next_player, score, won) = board.step(chosen_move, app.player);
        return (next_moves, next_player, score, won, chosen_move); // Return chosen_move
    }

    let mut stack: Vec<StackFrame> = Vec::new();

    // push root node
    stack.push(StackFrame {
        board: board.clone(),
        player: app.player,
        depth: app.depth,
        legal_moves: app.legal_moves.clone(),
        move_idx: 0,
        alpha: i32::MIN,
        beta: i32::MAX,
        best_score: i32::MIN,
        best_move: None,
        is_maximizing: true,
    });

    let mut child_score: Option<i32> = None;
    let mut final_best_move = app.legal_moves[0];

    while let Some(mut frame) = stack.pop() {
        if let Some(score) = child_score {
            if frame.is_maximizing {
                if score > frame.best_score {
                    frame.best_score = score;
                    frame.best_move = Some(frame.legal_moves[frame.move_idx - 1]);
                }
                frame.alpha = frame.alpha.max(frame.best_score);
            } else {
                if score < frame.best_score {
                    frame.best_score = score;
                    frame.best_move = Some(frame.legal_moves[frame.move_idx - 1]);
                }
                frame.beta = frame.beta.min(frame.best_score);
            }

            if frame.beta <= frame.alpha {
                frame.move_idx = frame.legal_moves.len();
            }
            child_score = None;
        }

        if frame.depth == 0 || frame.legal_moves.is_empty() {
            let (eval_score, _) = frame.board.evaluate(Player::Ai);
            child_score = Some(eval_score);
            continue;
        }

        if frame.move_idx < frame.legal_moves.len() {
            let move_to_play = frame.legal_moves[frame.move_idx];
            frame.move_idx += 1;

            stack.push(frame.clone());

            let mut child_board = frame.board.clone();
            let (next_moves, next_player, _, _) = child_board.step(move_to_play, frame.player);

            let is_max = next_player == app.player;

            stack.push(StackFrame {
                board: child_board,
                player: next_player,
                depth: frame.depth - 1,
                legal_moves: next_moves,
                move_idx: 0,
                alpha: frame.alpha,
                beta: frame.beta,
                best_score: if is_max { i32::MIN } else { i32::MAX },
                best_move: None,
                is_maximizing: is_max,
            });
        } else {
            child_score = Some(frame.best_score);

            if stack.is_empty() {
                if let Some(best) = frame.best_move {
                    final_best_move = best;
                }
            }
        }
    }

    // Step the main engine forward and return the post-step state along with the chosen move
    let (next_moves, next_player, score, won) = board.step(final_best_move, app.player);
    (next_moves, next_player, score, won, final_best_move)
}
