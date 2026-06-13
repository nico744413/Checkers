// for managing the app
pub struct App {
    pub mode: Mode,
    pub player: Player,
    pub depth: u32,
    pub legal_moves: Vec<Piece>,
    pub selected_piece: ShowPiece, // for selecting a piece to move
    pub last_piece: ShowPiece,
    pub dots: Vec<ShowPiece>, // for selecting moves
    pub won: bool,
}

impl App {
    pub fn new() -> Self {
        Self {
            mode: Mode::Ai,
            player: Player::None,
            depth: 0,
            legal_moves: Vec::new(),
            selected_piece: ShowPiece::None,
            last_piece: ShowPiece::None,
            dots: vec![],
            won: false,
        }
    }

    pub fn reset(&mut self) {
        self.player = Player::None;
        self.legal_moves = Vec::new();
        self.selected_piece = ShowPiece::None;
        self.last_piece = ShowPiece::None;
        self.dots = vec![];
        self.won = false;
    }
}

#[derive(PartialEq)]
pub enum Mode {
    Ai,
    Human
}

// for passing messages between ui and logic
pub enum Message {
    AiMode,
    HumanMode,
    StartGame(Player),
    EndGame,
    Clicked(usize),
    SetDifficulty(u32),
    None,
}

#[derive(Clone, Copy)]
pub enum ShowPiece {
    Piece(usize),
    None,
}

#[derive(PartialEq, Clone, Copy)]
pub enum Player {
    Black,
    White,
    Ai,
    Human,
    None,
}

// ------------ the checkers engine, technically still a component -----------

#[derive(Clone, Copy)]
pub struct Piece {
    pub from: u8,
    pub to: u8,
    capture: Option<u8>,
}

#[derive(Clone)]
pub struct Board {
    pub white_pieces: u32,
    pub black_pieces: u32,
    pub kings: u32,
    pub mid_jump_idx: Option<u8>,
}

const NOT_LEFT_EDGE: u32 = 0xFBFBFBFB;
const NOT_RIGHT_EDGE: u32 = 0xFDFDFDFD;

const WHITE_PROMOTION_ROW: u32 = 0x82000820; // Row 0
const BLACK_PROMOTION_ROW: u32 = 0x00041041; // Row 7

impl Board {
    pub fn new() -> Self {
        Self {
            white_pieces: 0x041C71C3,
            black_pieces: 0xE3820C38,
            kings: 0,
            mid_jump_idx: None,
        }
    }

    pub fn reset(&mut self) {
        self.white_pieces = 0x041C71C3;
        self.black_pieces = 0xE3820C38;
        self.kings = 0;
        self.mid_jump_idx = None;
    }

    pub fn evaluate(&self, player: Player) -> (i32, bool) {
        let white_count = self.white_pieces.count_ones() as i32;
        let black_count = self.black_pieces.count_ones() as i32;
        match player {
            Player::Ai => {
                let white_kings = (self.white_pieces & self.kings).count_ones() as i32;
                let black_kings = (self.black_pieces & self.kings).count_ones() as i32;
                let score = (black_count - white_count) * 100 + (black_kings - white_kings) * 75;
                return (score, white_count == 0)
            },
            Player::Human | Player::White => return (0, black_count == 0),
            Player::Black => return (0, white_count == 0),
            Player::None => panic!(),
        }
    }

    pub fn step(&mut self, piece: Piece, player: Player) -> (Vec<Piece>, Player, i32, bool) {
        let crowned = self.apply_move(&piece, &player);
        let (score, won) = self.evaluate(player);

        if piece.capture.is_some() && !crowned {
            self.mid_jump_idx = Some(piece.to);
            let extra_jumps = self.get_legal_moves(&player);
            if extra_jumps.is_empty() {
                self.mid_jump_idx = None;
                let nxt_player = match player {
                    Player::White => Player::Black,
                    Player::Black => Player::White,
                    Player::Ai => Player::Human,
                    Player::Human => Player::Ai,
                    Player::None => panic!("PlayerNone shouldnt appear here 0")
                };
                let nxt_legal_moves = self.get_legal_moves(&nxt_player);
                return (nxt_legal_moves, nxt_player, score, won)

            } else {return (extra_jumps, player, score, won)}

        } else {
            self.mid_jump_idx = None;
            let nxt_player = match player {
                Player::White => Player::Black,
                Player::Black => Player::White,
                Player::Ai => Player::Human,
                Player::Human => Player::Ai,
                Player::None => panic!("PlayerNone shouldnt appear here 0")
            };
            let nxt_legal_moves = self.get_legal_moves(&nxt_player);
            return (nxt_legal_moves, nxt_player, score, won)
        };
    }

    fn apply_move(&mut self, piece: &Piece, player: &Player) -> bool {
        let old_piece = 1 << piece.from;
        let new_piece = 1 << piece.to;
        let move_mask = old_piece | new_piece;

        let mut crowned = false;

        match player {
            Player::Ai | Player::Black => self.black_pieces ^= move_mask,
            Player::Human | Player::White => self.white_pieces ^= move_mask,
            Player::None => panic!("PlayerNone shouldnt appear here 1"),
        }

        if (self.kings & old_piece) != 0 {
            self.kings ^= move_mask;
        } else {
            match player {
                Player::Human | Player::White => {
                    if (new_piece & WHITE_PROMOTION_ROW) != 0 {
                        self.kings |= new_piece;
                        crowned = true;
                    }
                },
                Player::Ai | Player::Black => {
                    if (new_piece & BLACK_PROMOTION_ROW) != 0 {
                        self.kings |= new_piece;
                        crowned = true;
                    }
                },
                Player::None => {}
            }
        }

        match piece.capture {
            Some(capture) => {
                let capture_mask = !(1 << capture);
                self.white_pieces &= capture_mask;
                self.black_pieces &= capture_mask;
                self.kings &= capture_mask; // If a king is captured, remove it
            },
            None => {}
        }
        crowned
    }

    pub fn get_legal_moves(&self, player: &Player) -> Vec<Piece> {
        let (my_pieces, opp_pieces, is_white) = match player {
            Player::Human | Player::White => (self.white_pieces, self.black_pieces, true),
            Player::Ai | Player::Black => (self.black_pieces, self.white_pieces, false),
            Player::None => panic!("PlayerNone shouldnt appear here 2"),
        };
        let empty = !(self.white_pieces | self.black_pieces);
        let active_pieces = match self.mid_jump_idx {
            None => my_pieces,
            Some(idx) => my_pieces & (1 << idx),
        };

        let my_kings = active_pieces & self.kings;
        let up_movers = if is_white { active_pieces } else { my_kings };
        let down_movers = if is_white { my_kings } else { active_pieces };

        let step_ul = |b: u32| ((b & NOT_LEFT_EDGE) & !WHITE_PROMOTION_ROW).rotate_left(7);  // Up-Left (+7)
        let step_ur = |b: u32| ((b & NOT_RIGHT_EDGE) & !WHITE_PROMOTION_ROW).rotate_left(1); // Up-Right (+1)
        let step_dl = |b: u32| ((b & NOT_LEFT_EDGE) & !BLACK_PROMOTION_ROW).rotate_right(1);  // Down-Left (-1)
        let step_dr = |b: u32| ((b & NOT_RIGHT_EDGE) & !BLACK_PROMOTION_ROW).rotate_right(7); // Down-Right (-7)

        let mut jumps = Vec::with_capacity(8);

        // Up-Left Jump (+14) -> from = to - 14. Wrapped: (to + 18) & 31
        let mut land = step_ul(step_ul(up_movers) & opp_pieces) & empty;
        while land != 0 {
            let to = land.trailing_zeros() as u8;
            land &= land - 1; // Clear lowest set bit
            jumps.push(Piece { from: (to + 18) & 31, to, capture: Some((to + 25) & 31) });
        }

        // Up-Right Jump (+2) -> from = to - 2. Wrapped: (to + 30) & 31
        let mut land = step_ur(step_ur(up_movers) & opp_pieces) & empty;
        while land != 0 {
            let to = land.trailing_zeros() as u8;
            land &= land - 1;
            jumps.push(Piece { from: (to + 30) & 31, to, capture: Some((to + 31) & 31) });
        }

        // Down-Left Jump (-2) -> from = to + 2. Wrapped: (to + 2) & 31
        let mut land = step_dl(step_dl(down_movers) & opp_pieces) & empty;
        while land != 0 {
            let to = land.trailing_zeros() as u8;
            land &= land - 1;
            jumps.push(Piece { from: (to + 2) & 31, to, capture: Some((to + 1) & 31) });
        }

        // Down-Right Jump (-14) -> from = to + 14. Wrapped: (to + 14) & 31
        let mut land = step_dr(step_dr(down_movers) & opp_pieces) & empty;
        while land != 0 {
            let to = land.trailing_zeros() as u8;
            land &= land - 1;
            jumps.push(Piece { from: (to + 14) & 31, to, capture: Some((to + 7) & 31) });
        }

        // If any captures exist, the player MUST take one. Standard rules skip normal moves.
        if !jumps.is_empty() {
            return jumps;
        }
        if self.mid_jump_idx.is_some() {
            return vec![];
        }

        let mut moves = Vec::with_capacity(16);

        // Up-Left Step (+7) -> from = to - 7. Wrapped: (to + 25) & 31
        let mut land = step_ul(up_movers) & empty;
        while land != 0 {
            let to = land.trailing_zeros() as u8;
            land &= land - 1;
            moves.push(Piece { from: (to + 25) & 31, to, capture: None });
        }

        // Up-Right Step (+1) -> from = to - 1. Wrapped: (to + 31) & 31
        let mut land = step_ur(up_movers) & empty;
        while land != 0 {
            let to = land.trailing_zeros() as u8;
            land &= land - 1;
            moves.push(Piece { from: (to + 31) & 31, to, capture: None});
        }

        // Down-Left Step (-1) -> from = to + 1. Wrapped: (to + 1) & 31
        let mut land = step_dl(down_movers) & empty;
        while land != 0 {
            let to = land.trailing_zeros() as u8;
            land &= land - 1;
            moves.push(Piece { from: (to + 1) & 31, to, capture: None });
        }

        // Down-Right Step (-7) -> from = to + 7. Wrapped: (to + 7) & 31
        let mut land = step_dr(down_movers) & empty;
        while land != 0 {
            let to = land.trailing_zeros() as u8;
            land &= land - 1;
            moves.push(Piece { from: (to + 7) & 31, to, capture: None });
        }
        moves
    }
}
