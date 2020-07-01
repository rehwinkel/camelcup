use std::collections::HashMap;

type Player = u8;

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
#[repr(u8)]
enum Color {
    Green = 1,
    Blue = 2,
    Yellow = 4,
    White = 8,
    Orange = 16,
}

/*
#[derive(Clone, Debug)]
struct Stack {
    data: [Color; 5],
    size: u8,
}

impl Stack {
    fn new() -> Self {
        Stack {
            data: [Color::None; 5],
            size: 5,
        }
    }

    fn is_empty(&self) -> bool {
        self.data[0] as u8
            & self.data[1] as u8
            & self.data[2] as u8
            & self.data[3] as u8
            & self.data[4] as u8
            > 0
    }

    fn index(&self, color: Color) -> usize {
        for i in 0..self.size as usize {
            if self.data[i] as u8 & color as u8 > 0 {
                return i;
            }
        }
        return 5;
    }

    fn push(&mut self, color: Color) {
        self.data[self.size as usize] = color;
        self.size += 1;
    }

    fn extend(&mut self, colors: &[Color]) {
        std::ptr::copy_nonoverlapping(
            colors.as_ptr(),
            self.data.as_mut_ptr().add(self.size as usize),
            colors.len(),
        );
        self.size += colors.len() as u8;
    }
}
*/

impl Color {
    fn to_index(&self) -> usize {
        [0, 0, 1, 0, 2, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, 4][*self as u8 as usize]
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Hash, Eq)]
enum Move {
    None,
    Roll,
    Place(u8, bool),
    LapBet(Color),
    BestBet(Color),
    WorstBet(Color),
}
const PLAYERS: usize = 2;

#[derive(Clone, Debug)]
struct Game {
    camels: [Vec<Color>; 17],
    desert_tiles: [Option<(Player, bool)>; 15],
    coins: [i16; PLAYERS],
    bet_cards: [u8; PLAYERS],
    lap_bets: [Vec<(Color, i32)>; PLAYERS],
    game_over: bool,
    avail_lap_bets: [usize; 5],
    dice: u8,
    best_bets: Vec<(Player, Color)>,
    worst_bets: Vec<(Player, Color)>,
}

fn make_vec_array<T>() -> [Vec<T>; PLAYERS] {
    let mut data = [(0_usize, 0_usize, 0_usize); PLAYERS];
    for i in 0..PLAYERS {
        unsafe {
            *data.as_mut_ptr().add(i) = std::mem::transmute(Vec::<T>::new());
        }
    }
    unsafe { std::mem::transmute(data) }
}

impl Game {
    fn start() -> Self {
        let mut camels = [
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
            Vec::with_capacity(6),
        ];
        camels[0] = vec![
            Color::Green,
            Color::Blue,
            Color::Yellow,
            Color::White,
            Color::Orange,
        ];
        camels[0].is_empty();
        Game {
            desert_tiles: [None; 15],
            avail_lap_bets: [5; 5],
            game_over: false,
            camels,
            lap_bets: make_vec_array(),
            coins: [3; PLAYERS],
            bet_cards: [Color::Orange as u8
                | Color::Yellow as u8
                | Color::Blue as u8
                | Color::White as u8
                | Color::Green as u8; PLAYERS],
            dice: Color::Orange as u8
                | Color::Yellow as u8
                | Color::Blue as u8
                | Color::White as u8
                | Color::Green as u8,
            worst_bets: Vec::with_capacity(10),
            best_bets: Vec::with_capacity(10),
        }
    }

    fn move_camel(&mut self, player: Player, camel: Color, steps: usize) {
        for (field, immut_stack) in self.camels.iter().enumerate() {
            let stack: &mut Vec<Color> = unsafe {
                (immut_stack as *const Vec<Color> as *mut Vec<Color>)
                    .as_mut()
                    .unwrap()
            };
            let index = stack.iter().position(|c| *c == camel);
            if let Some(index) = index {
                let mut dest = field + steps;
                if let Some((pl, score)) = self.desert_tiles[dest - 1] {
                    if score {
                        dest += 1;
                    } else {
                        dest -= 1;
                    }
                    self.coins[pl as usize] += 1;
                }
                self.coins[player as usize] += 1;

                if field == 0 {
                    stack.remove(index);
                    self.camels[dest].push(camel);
                } else {
                    let mvstack = &stack[index..];
                    self.camels[dest].extend(mvstack);
                    for _ in 0..mvstack.len() {
                        stack.remove(index);
                    }
                }
                break;
            }
        }
    }

    fn roll(&self, player: Player) -> Vec<Game> {
        let mut games = Vec::with_capacity(16);
        for number in 1..=3 {
            for die_id in 0..4 {
                if (1u8 << die_id) & self.dice > 0 {
                    let mut game = self.clone();
                    game.move_camel(
                        player,
                        unsafe { std::mem::transmute(1u8 << die_id) },
                        number,
                    );
                    games.push(game);
                }
            }
        }
        games
    }

    fn place(&self, player: Player, position: usize, score: bool) -> Option<Game> {
        if let None = self.desert_tiles[position - 2] {
            let mut game = self.clone();
            game.desert_tiles[position - 2] = Some((player, score));
            Some(game)
        } else {
            None
        }
    }

    fn lap_bet(&self, player: Player, color: Color) -> Option<Game> {
        if self.avail_lap_bets[color.to_index()] != 0 {
            let mut game = self.clone();
            let score = game.avail_lap_bets[color.to_index()];
            game.avail_lap_bets[color.to_index()] = match score {
                5 => 3,
                3 => 2,
                2 => 0,
                _ => panic!(),
            };
            game.lap_bets[player as usize].push((color, score as i32));
            Some(game)
        } else {
            None
        }
    }

    fn best_bet(&self, player: Player, color: Color) -> Option<Game> {
        if self.bet_cards[player as usize] & color as u8 > 0 {
            let mut game = self.clone();
            game.best_bets.push((player, color));
            Some(game)
        } else {
            None
        }
    }

    fn worst_bet(&self, player: Player, color: Color) -> Option<Game> {
        if self.bet_cards[player as usize] & color as u8 > 0 {
            let mut game = self.clone();
            game.worst_bets.push((player, color));
            Some(game)
        } else {
            None
        }
    }

    fn evaluate(&self, player: Player) -> f64 {
        self.coins[player as usize] as f64 * 10.0
    }
}

fn get_possible_games(start: &Game, player: Player) -> Vec<(Game, Move)> {
    let mut games: Vec<(Game, Move)> = start
        .roll(player)
        .into_iter()
        .map(|g| (g, Move::Roll))
        .collect();

    let mut maxfield: usize = 0;
    for i in 0..17 {
        if !start.camels[16 - i].is_empty() {
            maxfield = 16 - i;
            break;
        }
    }
    let maxfield = usize::max(maxfield, 2);
    for field in maxfield..17 {
        if let Some(g) = start.place(player, field, true) {
            games.push((g, Move::Place(field as u8, true)));
        }
        if let Some(g) = start.place(player, field, false) {
            games.push((g, Move::Place(field as u8, false)));
        }
    }
    let colors = [
        Color::Orange,
        Color::White,
        Color::Yellow,
        Color::Green,
        Color::Blue,
    ];
    for color in &colors {
        if let Some(g) = start.lap_bet(player, *color) {
            games.push((g, Move::LapBet(*color)));
        }
    }
    for color in &colors {
        if let Some(g) = start.worst_bet(player, *color) {
            games.push((g, Move::WorstBet(*color)));
        }
    }
    for color in &colors {
        if let Some(g) = start.best_bet(player, *color) {
            games.push((g, Move::BestBet(*color)));
        }
    }
    games
}

fn minimax(start: &Game, opt_player: Player, depth: usize, current_player: Player) -> (f64, Move) {
    if start.game_over || depth == 0 {
        (start.evaluate(opt_player), Move::None)
    } else {
        let mut scores = HashMap::new();
        let games = get_possible_games(start, current_player);
        for (game, mv) in games {
            let score = minimax(
                &game,
                opt_player,
                depth - 1,
                (current_player + 1) % PLAYERS as u8,
            )
            .0;
            if !scores.contains_key(&mv) {
                scores.insert(mv, (score, 1f64));
            } else {
                let r = scores.get_mut(&mv).unwrap();
                *r = (r.0 + score, r.1 + 1.0)
            }
        }
        scores
            .into_iter()
            .fold((-f64::INFINITY, Move::None), |acc, next| {
                let r = (next.1).0 / (next.1).1;
                if acc.0 < r {
                    (r, next.0)
                } else {
                    acc
                }
            })
    }
}

fn main() {
    let game = Game::start();
    let score = minimax(&game, 0, 4, 0);
    println!("{:?}", score);
}
