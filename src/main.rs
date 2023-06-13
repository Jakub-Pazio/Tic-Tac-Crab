use core::{fmt, panic};
use rand::Rng;
use std::cmp::Ordering;
use std::collections::HashMap;
use std::hash::{Hash, Hasher};
use std::io;
use std::time::{Duration, Instant};


#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
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
    fn opponent(&self) -> Player {
        return match self {
            Player::X => Player::O,
            Player::O => Player::X,
        };
    }
}

struct SearchStats {
    result: GameResult,
    visited: usize,
    time: Duration,
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

#[derive(Clone, Copy, PartialEq, Hash)]
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

#[derive(Clone,)]
struct Board {
    fields: Vec<Field>,
    player_turn: Player,
    moves: Vec<u32>,
    size: u32,
}

impl PartialEq for Board {
    fn eq(&self, other: &Self) -> bool {
        self.fields == other.fields
    }
}

impl Eq for Board {

}

impl Hash for Board {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.fields.hash(state);
    }
}

impl Board {
    fn create_board(n: u32) -> Self {
        Self {
            fields: vec![Field::Free; (n*n) as usize],
            player_turn: Player::X,
            moves: vec![],
            size: n,
        }
    }

    fn get_result(&self) -> GameResult {
        // TODO:Create this map once when creating game, and pass immutable reference to this fn
        let mut winner_combinations = HashMap::new();
        winner_combinations.insert(2,  vec![
            // Horizontal lines
            vec![0, 1],
            vec![2, 3],
            // Vertical lines
            vec![0, 2],
            vec![1, 3],
            // Diagonal lines
            vec![0, 3],
            vec![1, 2],
        ]);
        winner_combinations.insert(3,  vec![
            // Horizontal lines
            vec![0, 1, 2],
            vec![3, 4, 5],
            vec![6, 7, 8],
            // Vertical lines
            vec![0, 3, 6],
            vec![1, 4, 7],
            vec![2, 5, 8],
            // Diagonal lines
            vec![0, 4, 8],
            vec![2, 4, 6],
        ]);
        winner_combinations.insert(4,  vec![
            // Horizontal lines
            vec![0, 1, 2, 3],
            vec![4, 5, 6, 7],
            vec![8, 9, 10, 11],
            vec![12, 13, 14, 15],
            // Vertical lines
            vec![0, 4, 8, 12],
            vec![1, 5, 9, 13],
            vec![2, 6, 10, 14],
            vec![3, 7, 11, 15],
            // Diagonal lines
            vec![0, 5, 10, 15],
            vec![3, 6, 9, 12],
        ]);
        winner_combinations.insert(5,  vec![
            // Horizontal lines
            vec![0, 1, 2, 3, 4],
            vec![5, 6, 7, 8, 9],
            vec![10, 11, 12, 13, 14],
            vec![15, 16, 17, 18, 19],
            vec![20, 21, 22, 23, 24],
            // Vertical lines
            vec![0, 5, 10, 15, 20],
            vec![1, 6, 11, 16, 21],
            vec![2, 7, 12, 17, 22],
            vec![3, 8, 13, 18, 23],
            vec![4, 9, 14, 19, 24],
            // Diagonal lines
            vec![0, 6, 12, 18, 24],
            vec![4, 8, 12, 16, 20],
        ]);


        for combination in winner_combinations.get(&self.size).unwrap() {
            let mut player_x = 0;
            let mut player_o = 0;
            for &index in combination {
                match self.fields[index] {
                    Field::Player(Player::X) => player_x += 1,
                    Field::Player(Player::O) => player_o += 1,
                    _ => continue,
                }
            }
            if player_x == self.size {
                return GameResult::Player(Player::X);
            }
            if player_o == self.size {
                return GameResult::Player(Player::O);
            }
        }
        let is_play = self.fields.iter().any(|&x| x == Field::Free);
        if is_play {
            return GameResult::InProgress;
        }
        return GameResult::Draw;
    }

    fn lines_heuristic(&self, player: Player) -> i32 {
        let mut winner_combinations = HashMap::new();
        winner_combinations.insert(2,  vec![
            // Horizontal lines
            vec![0, 1],
            vec![2, 3],
            // Vertical lines
            vec![0, 2],
            vec![1, 3],
            // Diagonal lines
            vec![0, 3],
            vec![1, 2],
        ]);
        winner_combinations.insert(3,  vec![
            // Horizontal lines
            vec![0, 1, 2],
            vec![3, 4, 5],
            vec![6, 7, 8],
            // Vertical lines
            vec![0, 3, 6],
            vec![1, 4, 7],
            vec![2, 5, 8],
            // Diagonal lines
            vec![0, 4, 8],
            vec![2, 4, 6],
        ]);
        winner_combinations.insert(4,  vec![
            // Horizontal lines
            vec![0, 1, 2, 3],
            vec![4, 5, 6, 7],
            vec![8, 9, 10, 11],
            vec![12, 13, 14, 15],
            // Vertical lines
            vec![0, 4, 8, 12],
            vec![1, 5, 9, 13],
            vec![2, 6, 10, 14],
            vec![3, 7, 11, 15],
            // Diagonal lines
            vec![0, 5, 10, 15],
            vec![3, 6, 9, 12],
        ]);
        let mut p_possible_wins: i32 = self.size as i32 * 2 + 2;
        let mut o_possible_wins: i32 = self.size as i32 * 2 + 2;

        for combination in winner_combinations.get(&self.size).unwrap() {
            if (combination.iter().any(|&x| self.fields[x] == Field::Player(player))) {
                o_possible_wins -= 1;
            }
            if (combination.iter().any(|&x| self.fields[x] == Field::Player(player.opponent()))) {
                p_possible_wins -= 1;
            }
        }

        p_possible_wins - o_possible_wins
    }
    // TODO: There is bug to fix
    fn better_heuristic(&self, player: Player) -> i32 {
        let mut result = 0;

        let mut winner_combinations = HashMap::new();
        winner_combinations.insert(2,  vec![
            // Horizontal lines
            vec![0, 1],
            vec![2, 3],
            // Vertical lines
            vec![0, 2],
            vec![1, 3],
            // Diagonal lines
            vec![0, 3],
            vec![1, 2],
        ]);
        winner_combinations.insert(3,  vec![
            // Horizontal lines
            vec![0, 1, 2],
            vec![3, 4, 5],
            vec![6, 7, 8],
            // Vertical lines
            vec![0, 3, 6],
            vec![1, 4, 7],
            vec![2, 5, 8],
            // Diagonal lines
            vec![0, 4, 8],
            vec![2, 4, 6],
        ]);
        winner_combinations.insert(4,  vec![
            // Horizontal lines
            vec![0, 1, 2, 3],
            vec![4, 5, 6, 7],
            vec![8, 9, 10, 11],
            vec![12, 13, 14, 15],
            // Vertical lines
            vec![0, 4, 8, 12],
            vec![1, 5, 9, 13],
            vec![2, 6, 10, 14],
            vec![3, 7, 11, 15],
            // Diagonal lines
            vec![0, 5, 10, 15],
            vec![3, 6, 9, 12],
        ]);

        for combination in winner_combinations.get(&self.size).unwrap() {
            let mut player_x = 0;
            let mut player_o = 0;
            for &index in combination {
                match self.fields[index] {
                    Field::Player(Player::X) => player_x += 1,
                    Field::Player(Player::O) => player_o += 1,
                    _ => continue,
                }
            }
            if player_x == self.size - 1 {
                match player {
                    Player::X => result += 5,
                    Player::O => result -= 100,
                }
            }
            if player_o == self.size - 1 {
                match player {
                    Player::X => result -= 100,
                    Player::O => result += 5,
                }
            }
        }

        result
    }

    fn make_move(&mut self, index: u32, len: u32) -> Result<(), &'static str> {
        if index <= len {
            match self.fields[index as usize] {
                Field::Free => {
                    self.fields[index as usize] = Field::Player(self.player_turn);
                    self.moves.push(index);
                    self.player_turn = self.player_turn.opponent();
                    Ok(())
                }
                Field::Player(_) => Err("Field is already taken!"),
            }
        } else {
            Err("Wrong square number")
        }
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
            self.player_turn = self.player_turn.opponent();
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
                    temp.make_move(index as u32, self.size * self.size).unwrap();
                    result.push(temp);
                }
            }
        }
        return result;
    }
    fn generate_sorted_lines_heuristic(&self) -> Vec<Self> {
        let mut res = self.generate_moves();
        res.sort_by_key(|b| std::cmp::Reverse(b.lines_heuristic(self.player_turn)));
        res
    }

    fn generate_own_heuristic(&self) -> Vec<Self> {
        let mut res = self.generate_moves();
        res.sort_by_key(|b| std::cmp::Reverse(b.better_heuristic(self.player_turn)));
        res
    }

    fn find_best_move(&self) -> u32 {
        match self.player_turn {
            Player::X => {
                let mut best_move = self.generate_moves()[0].clone(); // I'll get int at the end!
                let mut best_move_score = min_max(&mut best_move, 10, Player::O);
                for legal_board in self.generate_moves() {
                    if min_max(&mut legal_board.clone(), 10, Player::O).result > best_move_score.result {
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
                    if min_max(&mut legal_board.clone(), 10, Player::X).result < best_move_score.result {
                        best_move_score = min_max(&mut legal_board.clone(), 10, Player::X);
                        best_move = legal_board;
                    }
                }
                return *best_move.moves.last().unwrap();
            }
        }
    }
    fn find_best_move_alfa_beta(&self) -> u32 {
        let mut alfa = GameResult::Player(Player::O);
        let mut beta = GameResult::Player(Player::X);
        match self.player_turn {
            Player::X => {
                let mut best_move = self.generate_moves()[0].clone(); // I'll get int at the end!
                let mut best_move_score = alpha_beta(&mut best_move, 10, alfa, beta, Player::O);
                for legal_board in self.generate_moves() {
                    if alpha_beta(&mut legal_board.clone(), 10, alfa, beta, Player::O).result > best_move_score.result {
                        best_move_score = alpha_beta(&mut legal_board.clone(), 10,  alfa, beta, Player::O);
                        best_move = legal_board;
                    }
                }
                return *best_move.moves.last().unwrap();
            }
            Player::O => {
                // Do the same but just flip grater sign i guess ?
                let mut best_move = self.generate_moves()[0].clone();
                let mut best_move_score = alpha_beta(&mut best_move, 10, alfa, beta, Player::X);
                for legal_board in self.generate_moves() {
                    if alpha_beta(&mut legal_board.clone(), 10, alfa, beta, Player::X).result < best_move_score.result {
                        best_move_score = alpha_beta(&mut legal_board.clone(), 10, alfa, beta, Player::X);
                        best_move = legal_board;
                    }
                }
                return *best_move.moves.last().unwrap();
            }
        }
    }
    fn rot90board(&self) -> Self {
        let mut result_board = vec![Field::Free; (self.size * self.size) as usize];

        for i in 0..self.size {
            for j in 0..self.size {
                result_board[(j * self.size + (self.size - 1 - i)) as usize] = self.fields[(i * self.size + j) as usize];
            }
        }

        Board{
            fields: result_board,
            moves: self.moves.clone(),
            player_turn: self.player_turn,
            size: self.size
        }
    }
}

impl fmt::Debug for Board {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for (index, field) in self.fields.iter().enumerate() {
            if index % self.size as usize == 0 {
                write!(f, "\n")?;
                for _ in 0..self.size {
                    write!(f, "-----")?;
                }
                write!(f, "\n")?;
            }
            write!(f, " {:?} ", field)?;
            if index % self.size as usize != (self.size as usize) - 1 {
                write!(f, " |").expect("bad");
            }
        }
        write!(f, "\n")?;
        for _ in 0..self.size {
            write!(f, "-----")?;
        }
        write!(f, "\n")?;
        Ok(())
    }
}

// TODO bad implementation btw
fn alpha_beta(board: &mut Board, _depth: i8, mut alfa: GameResult, mut beta: GameResult, max_player: Player) -> SearchStats {
    let start_time = Instant::now();
    unsafe {
        A_B_COUNTER += 1;
    }
    let result = board.get_result();
    match result {
        GameResult::Draw => SearchStats {
            result: result,
            visited: 10,
            time: start_time.elapsed(),
        },
        GameResult::Player(_) => SearchStats {
            result: result,
            visited: 10,
            time: start_time.elapsed(),
        },
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
                        board.make_move(legal_move as u32, board.size * board.size).unwrap();
                        let local_result = alpha_beta(board, _depth - 1, alfa, beta, board.player_turn);
                        if local_result.result > best_score {
                            best_score = local_result.result;
                        }
                        if alfa < best_score {
                            alfa = best_score;
                        }
                        if best_score >= beta {
                            board.undo_last_move().unwrap();
                            break;
                        }
                        board.undo_last_move().unwrap();
                    }
                    return SearchStats {
                        result: best_score,
                        visited: 10,
                        time: start_time.elapsed(),
                    };
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
                        board.make_move(legal_move as u32, board.size * board.size).unwrap();
                        let local_result = alpha_beta(board, _depth - 1, alfa, beta, board.player_turn);
                        if local_result.result < best_score {
                            best_score = local_result.result;
                        }
                        if beta > best_score {
                            beta = best_score;
                        }
                        if best_score <= alfa {
                            board.undo_last_move().unwrap();
                            break;
                        }
                        board.undo_last_move().unwrap();
                    }
                    return SearchStats {
                        result: best_score,
                        visited: 10,
                        time: start_time.elapsed(),
                    };
                }
            }
        }
    }
}

fn alpha_beta_h1(board: &mut Board, _depth: i8, mut alfa: GameResult, mut beta: GameResult, max_player: Player) -> SearchStats {
    let start_time = Instant::now();
    unsafe {
        A_B_COUNTER_H1 += 1;
    }
    let result = board.get_result();
    match result {
        GameResult::Draw => SearchStats {
            result: result,
            visited: 10,
            time: start_time.elapsed(),
        },
        GameResult::Player(_) => SearchStats {
            result: result,
            visited: 10,
            time: start_time.elapsed(),
        },
        GameResult::InProgress => {
            match max_player {
                Player::X => {
                    // We want to maximize the score
                    let mut best_score = GameResult::Player(Player::O);
                    let possible_moves = board.generate_sorted_lines_heuristic();
                    for legal_move in possible_moves {
                        board.make_move(*legal_move.moves.last().unwrap() as u32, board.size * board.size).unwrap();
                        let local_result = alpha_beta_h1(board, _depth - 1, alfa, beta, board.player_turn);
                        if local_result.result > best_score {
                            best_score = local_result.result;
                        }
                        if alfa < best_score {
                            alfa = best_score;
                        }
                        if best_score >= beta {
                            board.undo_last_move().unwrap();
                            break;
                        }
                        board.undo_last_move().unwrap();
                    }
                    return SearchStats {
                        result: best_score,
                        visited: 10,
                        time: start_time.elapsed(),
                    };
                }
                Player::O => {
                    let mut best_score = GameResult::Player(Player::X);
                    let possible_moves = board.generate_sorted_lines_heuristic();
                    for legal_move in possible_moves {
                        board.make_move(*legal_move.moves.last().unwrap() as u32, board.size * board.size).unwrap();
                        let local_result = alpha_beta_h1(board, _depth - 1, alfa, beta, board.player_turn);
                        if local_result.result < best_score {
                            best_score = local_result.result;
                        }
                        if beta > best_score {
                            beta = best_score;
                        }
                        if best_score <= alfa {
                            board.undo_last_move().unwrap();
                            break;
                        }
                        board.undo_last_move().unwrap();
                    }
                    return SearchStats {
                        result: best_score,
                        visited: 10,
                        time: start_time.elapsed(),
                    };
                }
            }
        }
    }
}

fn alpha_beta_h2(board: &mut Board, _depth: i8, mut alfa: GameResult, mut beta: GameResult, max_player: Player) -> SearchStats {
    let start_time = Instant::now();
    unsafe {
        A_B_H2 += 1;
    }
    let result = board.get_result();
    match result {
        GameResult::Draw => SearchStats {
            result: result,
            visited: 10,
            time: start_time.elapsed(),
        },
        GameResult::Player(_) => SearchStats {
            result: result,
            visited: 10,
            time: start_time.elapsed(),
        },
        GameResult::InProgress => {
            match max_player {
                Player::X => {
                    // We want to maximize the score
                    let mut best_score = GameResult::Player(Player::O);
                    let possible_moves = board.generate_own_heuristic();
                    for legal_move in possible_moves {
                        board.make_move(*legal_move.moves.last().unwrap() as u32, board.size * board.size).unwrap();
                        let local_result = alpha_beta_h2(board, _depth - 1, alfa, beta, board.player_turn);
                        if local_result.result > best_score {
                            best_score = local_result.result;
                        }
                        if alfa < best_score {
                            alfa = best_score;
                        }
                        if best_score >= beta {
                            board.undo_last_move().unwrap();
                            break;
                        }
                        board.undo_last_move().unwrap();
                    }
                    return SearchStats {
                        result: best_score,
                        visited: 10,
                        time: start_time.elapsed(),
                    };
                }
                Player::O => {
                    let mut best_score = GameResult::Player(Player::X);
                    let possible_moves = board.generate_own_heuristic();
                    for legal_move in possible_moves {
                        board.make_move(*legal_move.moves.last().unwrap() as u32, board.size * board.size).unwrap();
                        let local_result = alpha_beta_h2(board, _depth - 1, alfa, beta, board.player_turn);
                        if local_result.result < best_score {
                            best_score = local_result.result;
                        }
                        if beta > best_score {
                            beta = best_score;
                        }
                        if best_score <= alfa {
                            board.undo_last_move().unwrap();
                            break;
                        }
                        board.undo_last_move().unwrap();
                    }
                    return SearchStats {
                        result: best_score,
                        visited: 10,
                        time: start_time.elapsed(),
                    };
                }
            }
        }
    }
}

fn alpha_beta_lookup(board: &mut Board, _depth: i8, mut alfa: GameResult, mut beta: GameResult, max_player: Player, look_up: &mut HashMap<Board, GameResult>) -> SearchStats {
    let start_time = Instant::now();
    match look_up.get(board) {
        Some(&res) => SearchStats {
            result: res,
            visited: 10,
            time: start_time.elapsed()
        },
        _ => {
            unsafe {
                A_B_LOOK_UP += 1;
            }
            let result = board.get_result();
            match result {
                GameResult::Draw => {
                    look_up.insert(board.clone(), result);
                    SearchStats {
                        result: result,
                        visited: 10,
                        time: start_time.elapsed(),
                    }
                },
                GameResult::Player(_) => {
                    look_up.insert(board.clone(), result);
                    SearchStats {
                        result: result,
                        visited: 10,
                        time: start_time.elapsed(),
                    }
                },
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
                                board.make_move(legal_move as u32, board.size * board.size).unwrap();
                                let local_result = alpha_beta_lookup(board, _depth - 1, alfa, beta, board.player_turn, look_up);
                                if local_result.result > best_score {
                                    best_score = local_result.result;
                                }
                                if alfa < best_score {
                                    alfa = best_score;
                                }
                                if best_score >= beta {
                                    board.undo_last_move().unwrap();
                                    break;
                                }
                                board.undo_last_move().unwrap();
                            }
                            return SearchStats {
                                result: best_score,
                                visited: 10,
                                time: start_time.elapsed(),
                            };
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
                                board.make_move(legal_move as u32, board.size * board.size).unwrap();
                                let local_result = alpha_beta_lookup(board, _depth - 1, alfa, beta, board.player_turn, look_up);
                                if local_result.result < best_score {
                                    best_score = local_result.result;
                                }
                                if beta > best_score {
                                    beta = best_score;
                                }
                                if best_score <= alfa {
                                    board.undo_last_move().unwrap();
                                    break;
                                }
                                board.undo_last_move().unwrap();
                            }
                            look_up.insert(board.clone(), best_score);
                            return SearchStats {
                                result: best_score,
                                visited: 10,
                                time: start_time.elapsed(),
                            };
                        }
                    }
                }
            }
        }
    }
}

fn check_for_rotation(board: &mut Board,lookup: &HashMap<Board, GameResult>) -> Option<GameResult> {
    match lookup.get(board) {
        Some(res) => {
            return Some(*res);
        },
        _ => {}
    }
    match lookup.get(&board.rot90board()) {
        Some(res) => {
            return Some(*res);
        },
        _ => {}
    }
    match lookup.get(&board.rot90board().rot90board()) {
        Some(res) => {
            return Some(*res);
        },
        _ => {}
    }
    match lookup.get(&board.rot90board().rot90board().rot90board()) {
        Some(res) => {
            return Some(*res);
        },
        _ => {}
    }
    None
}


fn alpha_beta_lookup_sym(board: &mut Board, _depth: i8, mut alfa: GameResult, mut beta: GameResult, max_player: Player, look_up: &mut HashMap<Board, GameResult>) -> SearchStats {
    let start_time = Instant::now();
    let lookup_result = check_for_rotation(&mut board.clone(), look_up);
    match lookup_result {
        Some(res) => SearchStats {
            result: res,
            visited: 10,
            time: start_time.elapsed()
        },
        _ => {
            unsafe {
                A_B_LOOK_UP_SYM += 1;
            }
            let result = board.get_result();
            match result {
                GameResult::Draw => {
                    look_up.insert(board.clone(), result);
                    SearchStats {
                        result: result,
                        visited: 10,
                        time: start_time.elapsed(),
                    }
                },
                GameResult::Player(_) => {
                    look_up.insert(board.clone(), result);
                    SearchStats {
                        result: result,
                        visited: 10,
                        time: start_time.elapsed(),
                    }
                },
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
                                board.make_move(legal_move as u32, board.size * board.size).unwrap();
                                let local_result = alpha_beta_lookup_sym(board, _depth - 1, alfa, beta, board.player_turn, look_up);
                                if local_result.result > best_score {
                                    best_score = local_result.result;
                                }
                                if alfa < best_score {
                                    alfa = best_score;
                                }
                                if best_score >= beta {
                                    board.undo_last_move().unwrap();
                                    break;
                                }
                                board.undo_last_move().unwrap();
                            }
                            return SearchStats {
                                result: best_score,
                                visited: 10,
                                time: start_time.elapsed(),
                            };
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
                                board.make_move(legal_move as u32, board.size * board.size).unwrap();
                                let local_result = alpha_beta_lookup_sym(board, _depth - 1, alfa, beta, board.player_turn, look_up);
                                if local_result.result < best_score {
                                    best_score = local_result.result;
                                }
                                if beta > best_score {
                                    beta = best_score;
                                }
                                if best_score <= alfa {
                                    board.undo_last_move().unwrap();
                                    break;
                                }
                                board.undo_last_move().unwrap();
                            }
                            look_up.insert(board.clone(), best_score);
                            return SearchStats {
                                result: best_score,
                                visited: 10,
                                time: start_time.elapsed(),
                            };
                        }
                    }
                }
            }
        }
    }
}

fn alpha_beta_lookup_sym_h1(board: &mut Board, _depth: i8, mut alfa: GameResult, mut beta: GameResult, max_player: Player, look_up: &mut HashMap<Board, GameResult>) -> SearchStats {
    let start_time = Instant::now();
    let lookup_result = check_for_rotation(&mut board.clone(), look_up);
    match lookup_result {
        Some(res) => SearchStats {
            result: res,
            visited: 10,
            time: start_time.elapsed()
        },
        _ => {
            unsafe {
                A_B_LOOK_UP_SYM_H1 += 1;
            }
            let result = board.get_result();
            match result {
                GameResult::Draw => {
                    look_up.insert(board.clone(), result);
                    SearchStats {
                        result: result,
                        visited: 10,
                        time: start_time.elapsed(),
                    }
                },
                GameResult::Player(_) => {
                    look_up.insert(board.clone(), result);
                    SearchStats {
                        result: result,
                        visited: 10,
                        time: start_time.elapsed(),
                    }
                },
                GameResult::InProgress => {
                    match max_player {
                        Player::X => {
                            // We want to maximize the score
                            let mut best_score = GameResult::Player(Player::O);
                            let possible_moves = board.generate_sorted_lines_heuristic();
                            //println!("{:?}", possible_moves.len());
                            for legal_move in possible_moves {
                                board.make_move(*legal_move.moves.last().unwrap(), board.size * board.size).unwrap();
                                let local_result = alpha_beta_lookup_sym_h1(board, _depth - 1, alfa, beta, board.player_turn, look_up);
                                if local_result.result > best_score {
                                    best_score = local_result.result;
                                }
                                if alfa < best_score {
                                    alfa = best_score;
                                }
                                if best_score >= beta {
                                    board.undo_last_move().unwrap();
                                    break;
                                }
                                board.undo_last_move().unwrap();
                            }
                            return SearchStats {
                                result: best_score,
                                visited: 10,
                                time: start_time.elapsed(),
                            };
                        }
                        Player::O => {
                            let mut best_score = GameResult::Player(Player::X);
                            let possible_moves = board.generate_sorted_lines_heuristic();
                            for legal_move in possible_moves {
                                board.make_move(*legal_move.moves.last().unwrap() as u32, board.size * board.size).unwrap();
                                let local_result = alpha_beta_lookup_sym_h1(board, _depth - 1, alfa, beta, board.player_turn, look_up);
                                if local_result.result < best_score {
                                    best_score = local_result.result;
                                }
                                if beta > best_score {
                                    beta = best_score;
                                }
                                if best_score <= alfa {
                                    board.undo_last_move().unwrap();
                                    break;
                                }
                                board.undo_last_move().unwrap();
                            }
                            look_up.insert(board.clone(), best_score);
                            return SearchStats {
                                result: best_score,
                                visited: 10,
                                time: start_time.elapsed(),
                            };
                        }
                    }
                }
            }
        }
    }
}

fn alpha_beta_lookup_sym_h2(board: &mut Board, _depth: i8, mut alfa: GameResult, mut beta: GameResult, max_player: Player, look_up: &mut HashMap<Board, GameResult>) -> SearchStats {
    let start_time = Instant::now();
    let lookup_result = check_for_rotation(&mut board.clone(), look_up);
    match lookup_result {
        Some(res) => SearchStats {
            result: res,
            visited: 10,
            time: start_time.elapsed()
        },
        _ => {
            unsafe {
                A_B_LOOK_UP_SYM_H2 += 1;
            }
            let result = board.get_result();
            match result {
                GameResult::Draw => {
                    look_up.insert(board.clone(), result);
                    SearchStats {
                        result: result,
                        visited: 10,
                        time: start_time.elapsed(),
                    }
                },
                GameResult::Player(_) => {
                    look_up.insert(board.clone(), result);
                    SearchStats {
                        result: result,
                        visited: 10,
                        time: start_time.elapsed(),
                    }
                },
                GameResult::InProgress => {
                    match max_player {
                        Player::X => {
                            // We want to maximize the score
                            let mut best_score = GameResult::Player(Player::O);
                            let possible_moves = board.generate_own_heuristic();
                            //println!("{:?}", possible_moves.len());
                            for legal_move in possible_moves {
                                board.make_move(*legal_move.moves.last().unwrap(), board.size * board.size).unwrap();
                                let local_result = alpha_beta_lookup_sym_h2(board, _depth - 1, alfa, beta, board.player_turn, look_up);
                                if local_result.result > best_score {
                                    best_score = local_result.result;
                                }
                                if alfa < best_score {
                                    alfa = best_score;
                                }
                                if best_score >= beta {
                                    board.undo_last_move().unwrap();
                                    break;
                                }
                                board.undo_last_move().unwrap();
                            }
                            return SearchStats {
                                result: best_score,
                                visited: 10,
                                time: start_time.elapsed(),
                            };
                        }
                        Player::O => {
                            let mut best_score = GameResult::Player(Player::X);
                            let possible_moves = board.generate_own_heuristic();
                            for legal_move in possible_moves {
                                board.make_move(*legal_move.moves.last().unwrap() as u32, board.size * board.size).unwrap();
                                let local_result = alpha_beta_lookup_sym_h2(board, _depth - 1, alfa, beta, board.player_turn, look_up);
                                if local_result.result < best_score {
                                    best_score = local_result.result;
                                }
                                if beta > best_score {
                                    beta = best_score;
                                }
                                if best_score <= alfa {
                                    board.undo_last_move().unwrap();
                                    break;
                                }
                                board.undo_last_move().unwrap();
                            }
                            look_up.insert(board.clone(), best_score);
                            return SearchStats {
                                result: best_score,
                                visited: 10,
                                time: start_time.elapsed(),
                            };
                        }
                    }
                }
            }
        }
    }
}

fn min_max(board: &mut Board, _depth: i16, max_player: Player) -> SearchStats {
    let start_time = Instant::now();
    unsafe {
        COUNTER += 1;
    }
    let result = board.get_result();
    match result {
        GameResult::Draw => SearchStats{
            result: result,
            visited: 10,
            time: start_time.elapsed()
        },
        GameResult::Player(_) => SearchStats{
            result: result,
            visited: 10,
            time: start_time.elapsed()
        },
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
                        board.make_move(legal_move as u32, board.size * board.size).unwrap();
                        let local_result = min_max(board, _depth - 1, board.player_turn);
                        if local_result.result > best_score {
                            best_score = local_result.result;
                        }
                        board.undo_last_move().unwrap();
                    }
                    return SearchStats{
                        result: best_score,
                        visited: 10,
                        time: start_time.elapsed()
                    };
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
                        board.make_move(legal_move as u32, board.size * board.size).unwrap();
                        let local_result = min_max(board, _depth - 1, board.player_turn);
                        if local_result.result < best_score {
                            best_score = local_result.result;
                        }
                        board.undo_last_move().unwrap();
                    }
                    return SearchStats{
                        result: best_score,
                        visited: 10,
                        time: start_time.elapsed()
                    };
                }
            }
        }
    }
}



fn min_max_lookup(board: &mut Board, _depth: i16, max_player: Player, look_up: &mut HashMap<Board, GameResult>) -> SearchStats {
    let start_time = Instant::now();
    // Lookup in HashMap, if we visited this board just return its result.
    match look_up.get(board) {
        Some(&res) => SearchStats{
            result: res,
            visited: 10,
            time: start_time.elapsed()
        },
        _ => {
            unsafe {
                COUNTERLOOK += 1;
            }
            let result = board.get_result();
            match result {
                GameResult::Draw => {
                    look_up.insert(board.clone(), result);
                    SearchStats {
                        result: result,
                        visited: 10,
                        time: start_time.elapsed(),
                    }
                },
                GameResult::Player(_) => {
                    look_up.insert(board.clone(), result);
                    SearchStats {
                        result: result,
                        visited: 10,
                        time: start_time.elapsed(),
                    }
                },
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
                                board.make_move(legal_move as u32, board.size * board.size).unwrap();
                                let local_result = min_max_lookup(board, _depth - 1, board.player_turn, look_up);
                                if local_result.result > best_score {
                                    best_score = local_result.result;
                                }
                                board.undo_last_move().unwrap();
                            } // Here add mby?
                            look_up.insert(board.clone(), best_score);
                            return SearchStats{
                                result: best_score,
                                visited: 10,
                                time: start_time.elapsed()
                            };
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
                                board.make_move(legal_move as u32, board.size * board.size).unwrap();
                                let local_result = min_max_lookup(board, _depth - 1, board.player_turn, look_up);
                                if local_result.result < best_score {
                                    best_score = local_result.result;
                                }
                                board.undo_last_move().unwrap();
                            }
                            // Adding here results in adding game-in-progress
                            look_up.insert(board.clone(), best_score);
                            return SearchStats{
                                result: best_score,
                                visited: 10,
                                time: start_time.elapsed()
                            };
                        }
                    }
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
    fn new(n: u32) -> Self {
        Self {
            board: Board::create_board(n),
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
        self.board.make_move(index as u32, self.board.size * self.board.size).unwrap();
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
        self.board.make_move(rng_move as u32, self.board.size * self.board.size).unwrap();
    }

    fn make_best_move(&mut self) {
        self.board.make_move(self.board.find_best_move(), self.board.size * self.board.size).unwrap();
    }
    fn make_best_move_a_b(&mut self) {
        self.board.make_move(self.board.find_best_move_alfa_beta(), self.board.size * self.board.size).unwrap();
    }

    fn make_best_move_lookup(&mut self) {

    }

    fn play(&mut self) {
        loop {
            self.human_move();
            println!("{:?}", self.board);
            println!("{:?}", self.board.lines_heuristic(Player::X));
            println!("{:?}", self.board.better_heuristic(Player::X));
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
            //println!("{:?}", self.board.generate_sorted_lines_heuristic());
            self.human_move();
            println!("{:?}", self.board);
            println!("{:?}", self.board.lines_heuristic(Player::O));
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
static mut A_B_COUNTER_H1: i32 = 0;
static mut COUNTERLOOK: i32 = 0;
static mut A_B_LOOK_UP: i32 = 0;
static mut A_B_H2: i32 = 0;
static mut A_B_LOOK_UP_SYM: i32 = 0;
static mut A_B_LOOK_UP_SYM_H1: i32 = 0;
static mut A_B_LOOK_UP_SYM_H2: i32 = 0;


fn main() {
    assert!(Player::X > Player::O);
    assert!(GameResult::Player(Player::X) > GameResult::Player(Player::O));
    assert!(GameResult::Player(Player::O) < GameResult::Draw);
    assert!(GameResult::Player(Player::X) > GameResult::Draw);
    assert!(GameResult::Player(Player::X) > GameResult::InProgress);
    assert!(GameResult::Player(Player::O) < GameResult::InProgress);
    assert_ne!(GameResult::InProgress > GameResult::Draw, true);
    assert_ne!(GameResult::InProgress < GameResult::Draw, true);
    let mut game = Game::new(3);
    let game1 = Game::new(3);
    let game2 = Game::new(4);
    let game3 = Game::new(3);
    let alfa = GameResult::Player(Player::O);
    let beta = GameResult::Player(Player::X);

    let first_min_max = min_max(&mut game3.board.clone(), 10,game.board.player_turn);

    println!("Min-Max:");

    let mut look_up_change = HashMap::new();
    let first_min_max_lookup = min_max_lookup(&mut game1.board.clone(), 10,game.board.player_turn, &mut look_up_change);
    println!("{:?}", first_min_max.result);
    println!("{:?}", first_min_max.time);

    unsafe {
        println!("{:?}", COUNTER);
    }
    println!("Lookup:");

    println!("{:?}", first_min_max_lookup.result);
    println!("{:?}", first_min_max_lookup.time);

    unsafe {
        println!("{:?}", COUNTERLOOK);
    }

    let first_alfa_beta = alpha_beta(&mut game3.board.clone(), 10, alfa, beta, game.board.player_turn);

    println!("AB:");

    println!("{:?}", first_alfa_beta.result);
    println!("{:?}", first_alfa_beta.time);

    unsafe {
        println!("{:?}", A_B_COUNTER);

    }

    let alfa1 = GameResult::Player(Player::O);
    let beta1 = GameResult::Player(Player::X);
    let first_alfa_beta_h1 = alpha_beta_h1(&mut game3.board.clone(), 10, alfa1, beta1, game.board.player_turn);

    println!("ABH1:");

    println!("{:?}", first_alfa_beta_h1.result);
    println!("{:?}", first_alfa_beta_h1.time);

    unsafe {
        println!("{:?}", A_B_COUNTER_H1);
    }

    println!("AB Lookup:");
    let mut look_up_ab = HashMap::new();

    let first_look_ab = alpha_beta_lookup(&mut game2.board.clone(), 10, alfa, beta, game.board.player_turn, &mut look_up_ab.clone());

    println!("{:?}", first_look_ab.result);
    println!("{:?}", first_look_ab.time);

    unsafe {
        println!("{:?}", A_B_LOOK_UP);
    }

    println!("AB Lookup Sym:");
    let mut look_up_ab_sym = HashMap::new();

    let first_look_ab_sym = alpha_beta_lookup_sym(&mut game2.board.clone(), 10, alfa, beta, game.board.player_turn, &mut look_up_ab_sym.clone());

    println!("{:?}", first_look_ab_sym.result);
    println!("{:?}", first_look_ab_sym.time);

    unsafe {
        println!("{:?}", A_B_LOOK_UP_SYM);
    }

    println!("AB Lookup Sym h1:");
    let first_look_ab_sym_h1 = alpha_beta_lookup_sym_h1(&mut game2.board.clone(), 10, alfa, beta, game.board.player_turn, &mut look_up_ab_sym.clone());

    println!("{:?}", first_look_ab_sym_h1.result);
    println!("{:?}", first_look_ab_sym_h1.time);

    unsafe {
        println!("{:?}", A_B_LOOK_UP_SYM_H1);
    }

    println!("AB Lookup Sym h2:");
    let first_look_ab_sym_h2 = alpha_beta_lookup_sym_h2(&mut game2.board.clone(), 10, alfa, beta, game.board.player_turn, &mut look_up_ab_sym.clone());

    println!("{:?}", first_look_ab_sym_h2.result);
    println!("{:?}", first_look_ab_sym_h2.time);

    unsafe {
        println!("{:?}", A_B_LOOK_UP_SYM_H2);
    }

    game.play();
}
