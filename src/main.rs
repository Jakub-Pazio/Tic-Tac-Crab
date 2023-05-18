use core::{panic, fmt};
use std::io;
use rand::Rng;

#[derive(Debug,Clone,Copy)]
enum Player {
    X,
    O,
}

#[derive(Clone,Copy)]
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
            }
            &Field::Free => write!(f, " "),
        }
    }
}

#[derive(Clone)]
struct Board {
    fields: Vec<Field>,
    player_turn: Player,
}

impl Board {
    fn create_board() -> Self {
        Self {
            fields: vec![Field::Free;9],
            player_turn: Player::O,
        }
    }

    fn generate_moves(&self) -> Vec<Self> {
        let mut result = vec![];
        for (index, &filed) in self.fields.iter().enumerate() {
            match filed {
                Field::Player(_) => {},
                Field::Free => {
                    let mut temp = self.clone();
                    temp.fields[index] = Field::Player(self.player_turn);
                    result.push(temp);
                }
            }
        }

        return result;
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

struct Game {
    board: Board,
    winner: Option<Player>,
}

impl Game {
    fn new() -> Self {
        Self {
            board: Board::create_board(),
            winner: None,
        }
    }

    fn make_move(&mut self) {
        let mut user_move = String::new();
        io::stdin()
            .read_line(&mut user_move)
            .expect("Failed to read input");
        let index = match user_move.trim_end().parse::<usize>() {
            Ok(i) => i,
            _ => panic!("can't read!"),
        };
        match self.board.fields[index] {
            Field::Player(_) => {},
            Field::Free => {
                self.board.fields[index] = Field::Player(self.board.player_turn);
                match self.board.player_turn {
                    Player::X => self.board.player_turn = Player::O,
                    Player::O => self.board.player_turn = Player::X,
                }
            }
        }
    }
    fn make_rand_move(&mut self) {
        let possible_moves = self.board.generate_moves();
        let rng = rand::thread_rng().gen_range(0..possible_moves.len());
        self.board = possible_moves[rng].clone();
        match self.board.player_turn {
            Player::X => self.board.player_turn = Player::O,
            Player::O => self.board.player_turn = Player::X,
        }
    }

    fn check_winner(&mut self) {
        let winner_combinations = vec![
            vec![0,1,2],
            vec![3,4,5],
            vec![6,7,8],
            vec![0,3,6],
            vec![1,4,7],
            vec![2,5,8],
            vec![0,4,8],
            vec![2,4,6],
        ];
        for combination in winner_combinations {
            let mut player_x = 0;
            let mut player_y = 0;
            for index in combination {
                match self.board.fields[index] {
                    Field::Player(Player::X) => player_x += 1,
                    Field::Player(Player::O) => player_y += 1,
                    _ => continue,
                }
            }
            if player_x == 3 {
                self.winner = Some(Player::X);
                return;
            }
            if player_y == 3 {
                self.winner = Some(Player::O);
                return;
            }
        }
    }

    fn play(&mut self) {
        loop {
            self.make_move();
            println!("{:?}", self.board);
            self.check_winner();
            match self.winner {
                Some(_) => {
                    let winner = self.winner.expect("No winner");
                    println!("Winner is {:?}", winner);
                    break
                },
                None => {}
            }
            self.make_rand_move();
            println!("{:?}", self.board);
            self.check_winner();
            match self.winner {
                Some(_) => {
                    let winner = self.winner.expect("No winner");
                    println!("Winner is {:?}", winner);
                    break
                },
                None => {}
            }
        }
    }
}
    

fn main() {
    let mut game = Game::new();
    game.play(); 
}
