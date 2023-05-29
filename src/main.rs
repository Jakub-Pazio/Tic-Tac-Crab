use core::{fmt, panic};
use rand::Rng;
use std::cmp::Ordering;
use std::io;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Player {
    X,
    O,
}

impl PartialOrd for Player {
    fn partial_cmp(&self, other: &Player) -> Option<Ordering> {
        match (self, other) {
            (Player::X, Player::X) => Some(Ordering::Equal),
            (Player::O, Player::O) => Some(Ordering::Equal),
            (Player::X, Player::O) => Some(Ordering::Greater),
            (Player::O, Player::X) => Some(Ordering::Less),
        }
    }
}

impl Ord for Player {
    fn cmp(&self, other: &Player) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

impl Player {
    fn oponent(&self) -> Player {
        return match self {
            Player::X => Player::O,
            Player::O => Player::X,
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GameResult {
    Player(Player),
    Draw,
    InProgress,
}

impl PartialOrd for GameResult {
    fn partial_cmp(&self, other: &GameResult) -> Option<Ordering> {
        match (self, other) {
            (GameResult::Player(Player::X), GameResult::Draw)
            | (GameResult::Player(Player::X), GameResult::InProgress)
            | (GameResult::Player(Player::X), GameResult::Player(Player::O))
            | (GameResult::Draw, GameResult::Player(Player::O))
            | (GameResult::InProgress, GameResult::Player(Player::O)) => Some(Ordering::Greater),

            (GameResult::Player(Player::O), GameResult::Draw)
            | (GameResult::Player(Player::O), GameResult::InProgress)
            | (GameResult::Player(Player::O), GameResult::Player(Player::X))
            | (GameResult::Draw, GameResult::Player(Player::X))
            | (GameResult::InProgress, GameResult::Player(Player::X)) => Some(Ordering::Less),

            (GameResult::InProgress, GameResult::Draw)
            | (GameResult::Draw, GameResult::InProgress) => Some(Ordering::Equal),
            _ => Some(Ordering::Equal),
        }
    }
}

impl Ord for GameResult {
    fn cmp(&self, other: &GameResult) -> Ordering {
        self.partial_cmp(other).unwrap()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Field {
    Player(Player),
    Free,
}

impl fmt::Debug for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            &Field::Player(p) => match p {
                Player::X => write!(f, "X"),
                Player::O => write!(f, "O"),
            },
            &Field::Free => write!(f, " "),
        }
    }
}

#[derive(Clone)]
struct Board {
    fields: Vec<Field>,
    player_turn: Player,
    moves: Vec<u8>,
}

impl Board {
    fn create_board() -> Self {
        Self {
            fields: vec![Field::Free; 9],
            player_turn: Player::X,
            moves: vec![],
        }
    }

    fn get_result(&self) -> GameResult {
        let winner_combinations = vec![
            vec![0, 1, 2],
            vec![3, 4, 5],
            vec![6, 7, 8],
            vec![0, 3, 6],
            vec![1, 4, 7],
            vec![2, 5, 8],
            vec![0, 4, 8],
            vec![2, 4, 6],
        ];
        for combination in winner_combinations {
            let mut player_x = 0;
            let mut player_o = 0;
            for index in combination {
                match self.fields[index] {
                    Field::Player(Player::X) => player_x += 1,
                    Field::Player(Player::O) => player_o += 1,
                    _ => continue,
                }
            }
            if player_x == 3 {
                return GameResult::Player(Player::X);
            }
            if player_o == 3 {
                return GameResult::Player(Player::O);
            }
        }
        let is_play = self.fields.iter().any(|&x| x == Field::Free);
        if is_play {
            return GameResult::InProgress;
        }
        return GameResult::Draw;
    }

    fn make_move(&mut self, index: u8) -> Result<(), &'static str> {
        match index {
            0..=8 => match self.fields[index as usize] {
                Field::Free => {
                    self.fields[index as usize] = Field::Player(self.player_turn);
                    self.moves.push(index);
                    self.player_turn = self.player_turn.oponent();
                }
                Field::Player(_) => return Err("Field is already taken!"),
            },
            _ => return Err("Wrong square number"),
        }
        Ok(())
    }

    fn undo_last_move(&mut self) -> Result<(), &'static str> {
        if self.moves.is_empty() {
            return Err("No moves has been played");
        }
        if let Some(last_move) = self.moves.pop() {
            if self.fields[last_move as usize] == Field::Free {
                panic!("Moves and board were not in sync!");
            }
            self.fields[last_move as usize] = Field::Free;
            self.player_turn = self.player_turn.oponent();
            return Ok(());
        }
        Err("Something went Wrong")
    }

    fn generate_moves(&self) -> Vec<Self> {
        let mut result = vec![];
        for (index, &field) in self.fields.iter().enumerate() {
            match field {
                Field::Player(_) => {}
                Field::Free => {
                    let mut temp = self.clone();
                    temp.make_move(index as u8).unwrap();
                    result.push(temp);
                }
            }
        }
        return result;
    }

    fn find_best_move(&self) -> u8 {
        match self.player_turn {
            Player::X => {
                let mut best_move = self.generate_moves()[0].clone(); // I'll get int at the end!
                let mut best_move_score = min_max(&mut best_move, 10, Player::O);
                for legal_board in self.generate_moves() {
                    if min_max(&mut legal_board.clone(), 10, Player::O) >= best_move_score {
                        best_move_score = min_max(&mut legal_board.clone(), 10, Player::O);
                        best_move = legal_board;
                    }
                }
                return *best_move.moves.last().unwrap();
            }
            Player::O => {
                // Do the same but just flip grater sign i guess ?
                let mut best_move = self.generate_moves()[0].clone();
                let mut best_move_score = min_max(&mut best_move, 10, Player::X);
                for legal_board in self.generate_moves() {
                    if min_max(&mut legal_board.clone(), 10, Player::X) <= best_move_score {
                        best_move_score = min_max(&mut legal_board.clone(), 10, Player::X);
                        best_move = legal_board;
                    }
                }
                return *best_move.moves.last().unwrap();
            }
        }
    }
    fn find_best_move_alfa_beta(&self) -> u8 {
        let mut alfa = GameResult::Player(Player::O);
        let mut beta = GameResult::Player(Player::X);
        match self.player_turn {
            Player::X => {
                let mut best_move = self.generate_moves()[0].clone(); // I'll get int at the end!
                let mut best_move_score = alpha_beta(&mut best_move, 10, &mut alfa, &mut beta, Player::O);
                for legal_board in self.generate_moves() {
                    if alpha_beta(&mut legal_board.clone(), 10, &mut alfa, &mut beta, Player::O) >= best_move_score {
                        best_move_score = alpha_beta(&mut legal_board.clone(), 10, &mut alfa, &mut beta, Player::O);
                        best_move = legal_board;
                    }
                }
                return *best_move.moves.last().unwrap();
            }
            Player::O => {
                // Do the same but just flip grater sign i guess ?
                let mut best_move = self.generate_moves()[0].clone();
                let mut best_move_score = alpha_beta(&mut best_move, 10, &mut alfa, &mut beta, Player::X);
                for legal_board in self.generate_moves() {
                    if alpha_beta(&mut legal_board.clone(), 10, &mut alfa, &mut beta, Player::X) <= best_move_score {
                        best_move_score = alpha_beta(&mut legal_board.clone(), 10, &mut alfa, &mut beta, Player::X);
                        best_move = legal_board;
                    }
                }
                return *best_move.moves.last().unwrap();
            }
        }
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (index, field) in self.fields.iter().enumerate() {
            if index % 3 == 0 {
                write!(f, "\n------------\n")?;
            }
            write!(f, " {:?} ", field)?;
            if index % 3 != 2 {
                write!(f, " |").expect("bad");
            }
        }
        write!(f, "\n------------\n")?;
        Ok(())
    }
}

fn min_max(board: &mut Board, _depth: u8, max_player: Player) -> GameResult {
    unsafe {
      COUNTER += 1;
    }
    let result = board.get_result();
    match result {
        GameResult::Draw => GameResult::Draw,
        GameResult::Player(x) => GameResult::Player(x),
        GameResult::InProgress => {
            match max_player {
                Player::X => {
                    // We want to maximize the score
                    let mut best_score = GameResult::Player(Player::O);
                    let possible_moves: Vec<usize> = board
                        .fields
                        .iter()
                        .enumerate()
                        .filter(|(_, value)| **value == Field::Free)
                        .map(|(index, _)| index)
                        .collect();
                    //println!("{:?}", possible_moves.len());
                    for legal_move in possible_moves {
                        board.make_move(legal_move as u8).unwrap();
                        let local_result = min_max(board, _depth - 1, board.player_turn);
                        if local_result > best_score {
                            best_score = local_result;
                        }
                        board.undo_last_move().unwrap();
                    }
                    return best_score;
                }
                Player::O => {
                    let mut best_score = GameResult::Player(Player::X);
                    let possible_moves: Vec<usize> = board
                        .fields
                        .iter()
                        .enumerate()
                        .filter(|(_, value)| **value == Field::Free)
                        .map(|(index, _)| index)
                        .collect();
                    for legal_move in possible_moves {
                        board.make_move(legal_move as u8).unwrap();
                        let local_result = min_max(board, _depth - 1, board.player_turn);
                        if local_result < best_score {
                            best_score = local_result;
                        }
                        board.undo_last_move().unwrap();
                    }
                    return best_score;
                }
            }
        }
    }
}
// TODO bad implementation btw
fn alpha_beta(board: &mut Board, _depth: u8, alfa: &mut GameResult, beta: &mut GameResult, max_player: Player) -> GameResult {
    unsafe {
        A_B_COUNTER += 1;
    }
    let result = board.get_result();
    match result {
        GameResult::Draw => GameResult::Draw,
        GameResult::Player(x) => GameResult::Player(x),
        GameResult::InProgress => {
            match max_player {
                Player::X => {
                    // We want to maximize the score
                    let mut best_score = GameResult::Player(Player::O);
                    let possible_moves: Vec<usize> = board
                        .fields
                        .iter()
                        .enumerate()
                        .filter(|(_, value)| **value == Field::Free)
                        .map(|(index, _)| index)
                        .collect();
                    //println!("{:?}", possible_moves.len());
                    for legal_move in possible_moves {
                        board.make_move(legal_move as u8).unwrap();
                        let local_result = alpha_beta(board, _depth - 1, alfa, beta, board.player_turn);
                        if local_result > best_score {
                            best_score = local_result;
                        }
                        if *alfa > *beta {
                            board.undo_last_move().unwrap();
                            break;
                        }
                        *alfa = local_result;
                        board.undo_last_move().unwrap();
                    }
                    return best_score;
                }
                Player::O => {
                    let mut best_score = GameResult::Player(Player::X);
                    let possible_moves: Vec<usize> = board
                        .fields
                        .iter()
                        .enumerate()
                        .filter(|(_, value)| **value == Field::Free)
                        .map(|(index, _)| index)
                        .collect();
                    for legal_move in possible_moves {
                        board.make_move(legal_move as u8).unwrap();
                        let local_result = alpha_beta(board, _depth - 1, alfa, beta, board.player_turn);
                        if local_result < best_score {
                            best_score = local_result;
                        }
                        if *beta < *alfa {
                            board.undo_last_move().unwrap();
                            break;
                        }
                        *beta = local_result;
                        board.undo_last_move().unwrap();
                    }
                    return best_score;
                }
            }
        }
    }
}

#[derive(Clone)]
struct Game {
    board: Board,
    winner: GameResult,
}

impl Game {
    fn new() -> Self {
        Self {
            board: Board::create_board(),
            winner: GameResult::InProgress,
        }
    }

    fn next_player(&mut self) {
        match self.board.player_turn {
            Player::X => self.board.player_turn = Player::O,
            Player::O => self.board.player_turn = Player::X,
        }
    }

    fn human_move(&mut self) {
        let mut user_move = String::new();
        io::stdin()
            .read_line(&mut user_move)
            .expect("Failed to read input");
        let index = user_move.trim_end().parse::<usize>().unwrap();
        self.board.make_move(index as u8).unwrap();
    }
    fn make_rand_move(&mut self) {
        let possible_moves: Vec<usize> = self
            .board
            .fields
            .iter()
            .enumerate()
            .filter(|(_, value)| **value == Field::Free)
            .map(|(index, _)| index)
            .collect();
        let rng = rand::thread_rng().gen_range(0..possible_moves.len());
        let rng_move = possible_moves[rng];
        print!("{:?}", possible_moves.len());
        self.board.make_move(rng_move as u8).unwrap();
    }

    fn make_best_move(&mut self) {
        self.board.make_move(self.board.find_best_move()).unwrap();
    }
    fn make_best_move_a_b(&mut self) {
        self.board.make_move(self.board.find_best_move_alfa_beta()).unwrap();
    }

    // fn make_best_move(&mut self) {
    //     !todo!();
    // }
    // fn make_best_move(&mut self) {
    //     let max_player = self.board.player_turn;
    //     let mut best_result = GameResult::InProgress;
    //     let mut best_move = None;

    //     for possible_move in self.board.generate_moves() {
    //         let new_game = Game {
    //             board: possible_move.clone(),
    //             winner: GameResult::InProgress,
    //         };
    //let result = min_max(&new_game.board, 8, max_player);

    // if best_result <= result {
    //     best_result = result;
    //     best_move = Some(possible_move);
    // }
    // }

    // if let Some(mv) = best_move {
    //     self.board = mv;
    //     self.next_player();
    // } else {
    //     panic!("No valid move found!");
    // }
    // }

    fn play(&mut self) {
        loop {
            self.make_best_move_a_b();
            println!("{:?}", self.board);
            self.winner = self.board.get_result();
            match self.winner {
                GameResult::Player(_) => {
                    let winner = self.winner;
                    println!("Winner is {:?}", winner);
                    break;
                }
                GameResult::Draw => {
                    println!("Good game, Draw!");
                    return;
                }
                _ => {}
            }
            self.human_move();
            println!("{:?}", self.board);
            self.winner = self.board.get_result();
            match self.winner {
                GameResult::Player(_) => {
                    let winner = self.winner;
                    println!("Winner is {:?}", winner);
                    break;
                }
                GameResult::Draw => {
                    println!("Good game, Draw!");
                    return;
                }
                _ => {}
            }
            println!("{:?}", self.board.moves);
        }
    }
}
// This is very bad, but i don't know how to do it idiomatically
// When i will have time i need to TODO: Refactor that
// I'm sorry :(

static mut COUNTER: i32 = 0;
static mut A_B_COUNTER: i32 = 0;


fn main() {
    assert!(Player::X > Player::O);
    assert!(GameResult::Player(Player::X) > GameResult::Player(Player::O));
    assert!(GameResult::Player(Player::O) < GameResult::Draw);
    assert!(GameResult::Player(Player::X) > GameResult::Draw);
    assert!(GameResult::Player(Player::X) > GameResult::InProgress);
    assert!(GameResult::Player(Player::O) < GameResult::InProgress);
    assert_ne!(GameResult::InProgress > GameResult::Draw, true);
    assert_ne!(GameResult::InProgress < GameResult::Draw, true);
    let mut game = Game::new();
    let mut game2 = Game::new();

    let first_min_max = min_max(&mut game.board.clone(), 10, game.board.player_turn);
    unsafe {
        println!("{:?}", COUNTER);
    }
    println!("{:?}", first_min_max);

    let mut alfa = GameResult::Player(Player::O);
    let mut beta = GameResult::Player(Player::X);
    let first_alfa_beta = alpha_beta(&mut game2.board.clone(), 10, &mut alfa, &mut beta, game.board.player_turn);
    unsafe {
        println!("{:?}", A_B_COUNTER);
    }
    println!("{:?}", first_min_max);
    println!("{:?}", first_alfa_beta);

    game.play();
}
