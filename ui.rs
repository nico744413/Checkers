use macroquad::prelude::*;

use crate::components::*;

// Renders gui according to mode and return button messages
pub fn render_gui(app: &App, height: f32, width: f32) -> Message {
    let mut message = Message::None;

    let panel_width = (width * 0.22).clamp(160.0, 240.0).floor();

    egui_macroquad::ui(|ctx| {
        egui_macroquad::egui::SidePanel::right("controls_panel")
            .exact_width(panel_width)
            .resizable(false)
            .show(ctx, |ui| {
                let labels: &[&str] = match app.mode {
                    Mode::Human => &["Clear Board", "Black First", "White First", "Human vs AI"],
                    Mode::Ai => &["AI First", "Human First", "Again", "Human vs Human"],
                };

                let button_height = (height * 0.055).clamp(34.0, 44.0).floor();
                let spacing = (height * 0.014).clamp(6.0, 12.0).floor();
                ui.spacing_mut().item_spacing = egui_macroquad::egui::vec2(0.0, spacing);

                // Safe top padding offset away from window edges
                ui.add_space(spacing * 1.5);

                // --- AI Settings Segment ---
                if app.mode == Mode::Ai {
                    ui.label(
                        egui_macroquad::egui::RichText::new("Search Depth")
                            .size((button_height * 0.34).clamp(11.0, 15.0))
                            .strong(),
                    );

                    let mut current_depth = app.depth;
                    // egui sliders naturally occupy full available width safely
                    let slider = egui_macroquad::egui::Slider::new(&mut current_depth, 0..=18);

                    if ui.add(slider).changed() {
                        message = Message::SetDifficulty(current_depth);
                    }
                    ui.add_space(spacing);
                }

                // --- Action Button Grid (Respects exact container bounds) ---
                let button_width = (ui.available_width() * 0.92).floor();
                let button_size = egui_macroquad::egui::vec2(button_width, button_height);

                for &text in labels {
                    let label = egui_macroquad::egui::RichText::new(text).size((button_height * 0.36).clamp(11.0, 15.0));

                    if ui.add_sized(button_size, egui_macroquad::egui::Button::new(label)).clicked() {
                        message = match text {
                            "Clear Board" | "Again" => Message::EndGame,
                            "Black First" => Message::StartGame(Player::Black),
                            "White First" => Message::StartGame(Player::White),
                            "AI First" => Message::StartGame(Player::Ai),
                            "Human First" => Message::StartGame(Player::Human),
                            "Human vs AI" => Message::AiMode,
                            "Human vs Human" => Message::HumanMode,
                            _ => Message::None,
                        };
                    }
                }

                // --- Bottom Game Status Monitoring Block ---
                ui.add_space(spacing * 2.0);
                ui.separator();
                ui.add_space(spacing);

                let font_size = (height * 0.020).clamp(13.0, 17.0).floor();

                if app.won {
                    let win_text = match app.player {
                        Player::White | Player::Human => "Black Wins",
                        Player::Black | Player::Ai => "White Wins",
                        _ => "Draw",
                    };

                    ui.label(
                        egui_macroquad::egui::RichText::new(win_text)
                            .color(egui_macroquad::egui::Color32::from_rgb(255, 215, 0))
                            .size(font_size)
                            .strong(),
                    );
                } else {
                    let status_text = match app.player {
                        Player::White | Player::Human => "White Turn".to_string(),
                        Player::Black => "Black Turn".to_string(),
                        Player::Ai => format!("AI Thinking (D{})", app.depth),
                        Player::None => "Ready".to_string(),
                    };

                    ui.label(
                        egui_macroquad::egui::RichText::new(status_text)
                            .color(egui_macroquad::egui::Color32::WHITE)
                            .size(font_size)
                            .strong(),
                    );
                }
            });
    });

    message
}

const LAYOUT_MAP: [[Option<usize>; 8]; 8] = [
    [None, Some(11), None, Some(05), None, Some(31), None, Some(25)],
    [Some(10), None, Some(04), None, Some(30), None, Some(24), None],
    [None, Some(03), None, Some(29), None, Some(23), None, Some(17)],
    [Some(02), None, Some(28), None, Some(22), None, Some(16), None],
    [None, Some(27), None, Some(21), None, Some(15), None, Some(09)],
    [Some(26), None, Some(20), None, Some(14), None, Some(08), None],
    [None, Some(19), None, Some(13), None, Some(07), None, Some(01)],
    [Some(18), None, Some(12), None, Some(06), None, Some(00), None],
];

// Takes board and visuals and returns mousebutton state
pub fn render_board(app: &App, board: &Board, height: f32, width: f32) -> Message {
    // Dynamically track the exact side panel size to find remaining viewport space
    let panel_width = (width * 0.22).clamp(160.0, 240.0).floor();
    let playable_width = width - panel_width;

    // Scale and center the board perfectly within the left-hand playable area
    let board_size = (playable_width * 0.90).min(height * 0.90).floor();
    let offset_x = ((playable_width - board_size) / 2.0).floor() + 14.0;
    let offset_y = ((height - board_size) / 2.0).floor();
    let square_size = board_size / 8.0;

    let select_idx = match app.selected_piece { ShowPiece::Piece(i) => Some(i as usize), _ => None };
    let last_idx = match app.last_piece { ShowPiece::Piece(i) => Some(i as usize), _ => None };
    let mut message = Message::None;

    for row in 0..8 {
        for col in 0..8 {
            let x = offset_x + col as f32 * square_size;
            let y = offset_y + row as f32 * square_size;

            let bit_idx = match LAYOUT_MAP[row][col] {
                Some(idx) => idx,
                None => {
                    draw_rectangle(x, y, square_size, square_size, Color::from_rgba(235, 236, 208, 255));
                    continue;
                }
            };

            let mask = 1 << bit_idx;

            let bg_color = if last_idx == Some(bit_idx) {
                draw_rectangle(x, y, square_size, square_size, Color::from_rgba(119, 149, 86, 255));
                Color::from_rgba(247, 247, 105, 130) // Highlight glow
            } else if select_idx == Some(bit_idx) {
                Color::from_rgba(186, 202, 43, 200)  // Selected piece
            } else {
                Color::from_rgba(119, 149, 86, 255)  // Standard green
            };
            draw_rectangle(x, y, square_size, square_size, bg_color);

            let cx = x + square_size / 2.0;
            let cy = y + square_size / 2.0;
            let is_king = (board.kings & mask) != 0;

            if (board.white_pieces & mask) != 0 {
                draw_piece(cx, cy, square_size, Color::from_rgba(245, 245, 245, 255), is_king);
            } else if (board.black_pieces & mask) != 0 {
                draw_piece(cx, cy, square_size, Color::from_rgba(45, 45, 45, 255), is_king);
            }

            let draw_dot = app.dots.iter().any(|dot| match dot {
                ShowPiece::Piece(dot_idx) => *dot_idx == bit_idx,
                ShowPiece::None => false,
            });
            if draw_dot {
                draw_circle(cx, cy, square_size * 0.14, Color::from_rgba(0, 0, 0, 50));
            }

            if is_mouse_button_pressed(MouseButton::Left) {
                let (mx, my) = mouse_position();
                if mx >= x && mx < x + square_size && my >= y && my < y + square_size {
                    message = Message::Clicked(bit_idx);
                }
            }
        }
    }
    message
}

fn draw_piece(cx: f32, cy: f32, square_size: f32, color: Color, is_king: bool) {
    let outer_r = square_size * 0.40;
    let inner_r = square_size * 0.28;
    let shadow_color = Color::from_rgba(20, 20, 20, 40);

    draw_circle(cx, cy, outer_r, shadow_color);
    draw_circle(cx, cy, outer_r * 0.96, color);

    let core_accent = if color.r > 0.5 { Color::from_rgba(220, 220, 220, 255) } else { Color::from_rgba(65, 65, 65, 255) };
    draw_circle(cx, cy, inner_r, core_accent);

    if is_king {
        draw_circle(cx, cy, square_size * 0.12, Color::from_rgba(247, 183, 49, 255));
    }
}
