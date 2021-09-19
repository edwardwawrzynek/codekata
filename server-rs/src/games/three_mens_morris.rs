use crate::games::{Fmt, GameInstance, GameScore, GameState, GameTurn, GameType};
use crate::models::UserId;
use std::fmt;

#[derive(Debug)]
pub struct ThreeMensMorrisGame();

#[derive(Eq, PartialEq, Copy, Clone, Debug)]
enum Cell {
    Piece(i8),
    Empty,
}

pub struct ThreeMensMorrisGameInstance {
    players: [UserId; 2],
    board: [[Cell; 3]; 3],
    turn: i8,
}

impl GameType for ThreeMensMorrisGame {
    fn deserialize(&self, data: &str, players: &[UserId]) -> Option<Box<dyn GameInstance>> {
        let mut components = data.split(',');
        let state = components.next()?;
        let turn = parse_num(components.next()?).map_or(None, |n| Some(n))? as i8;

        let mut board = [[Cell::Empty; 3]; 3];

        let mut y = 0;
        let mut x = 0;

        for c in state.chars() {
            match c {
                '0' => {
                    board[y][x] = Cell::Piece(0);
                }
                '1' => {
                    board[y][x] = Cell::Piece(1);
                }
                _ => {}
            }

            x += 1;
            if x >= 3 {
                x = 0;
                y += 1;
            }

            if y >= 3 {
                break;
            }
        }

        Some(Box::new(ThreeMensMorrisGameInstance {
            turn,
            players: [players[0], players[1]],
            board,
        }))
    }

    fn new(&self, players: &[UserId]) -> Option<Box<dyn GameInstance>> {
        if players.len() != 2 {
            None
        } else {
            Some(Box::new(ThreeMensMorrisGameInstance {
                board: [[Cell::Empty; 3]; 3],
                turn: 0,
                players: [players[0], players[1]],
            }))
        }
    }
}

impl ThreeMensMorrisGameInstance {
    fn check_win(&self, p: i8) -> bool {
        // vertical wins
        for x in 0..3 {
            let mut not_win = false;
            for y in 0..3 {
                match self.board[y][x] {
                    Cell::Piece(c) if c != p => {
                        not_win = true;
                    }
                    Cell::Empty => {
                        not_win = true;
                    }
                    _ => {}
                }
            }

            if !not_win {
                return true;
            }
        }
        // horizontal wins
        for y in 0..3 {
            let mut not_win = false;
            for x in 0..3 {
                match self.board[y][x] {
                    Cell::Piece(c) if c != p => {
                        not_win = true;
                    }
                    Cell::Empty => {
                        not_win = true;
                    }
                    _ => {}
                }
            }

            if !not_win {
                return true;
            }
        }
        // diagonal wins
        let mut not_win = false;
        for i in 0..3 {
            match self.board[i][i] {
                Cell::Piece(c) if c != p => {
                    not_win = true;
                }
                Cell::Empty => {
                    not_win = true;
                }
                _ => {}
            }
        }
        if !not_win {
            return true;
        }

        not_win = false;
        for i in 0..3 {
            match self.board[i][2 - i] {
                Cell::Piece(c) if c != p => {
                    not_win = true;
                }
                Cell::Empty => {
                    not_win = true;
                }
                _ => {}
            }
        }
        if !not_win {
            return true;
        }

        false
    }

    fn win(&self) -> Option<i8> {
        if self.check_win(0) {
            Some(0)
        } else if self.check_win(1) {
            Some(1)
        } else {
            None
        }
    }

    fn count(&self, p: i8) -> i32 {
        let mut count = 0;

        for row in &self.board {
            for cell in row {
                match *cell {
                    Cell::Piece(c) if c == p => {
                        count += 1;
                    }
                    _ => {}
                }
            }
        }

        count
    }
}

fn parse_num(str: &str) -> Result<usize, String> {
    match str.parse::<usize>() {
        Ok(i) => Ok(i),
        Err(_) => Err(format!("invalid number: {}", str)),
    }
}

fn in_bounds(x0: usize, y0: usize) -> Result<(), String> {
    if x0 >= 3 || y0 >= 3 {
        return Err(format!("cell {} {} is outside the board", x0, y0));
    }
    Ok(())
}

fn expect(str: Option<&str>) -> Result<&str, String> {
    match str {
        Some(s) => Ok(s),
        None => Err("expected another argument".to_string()),
    }
}

impl GameInstance for ThreeMensMorrisGameInstance {
    fn serialize(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        for row in &self.board {
            for cell in row {
                match *cell {
                    Cell::Empty => write!(f, ".")?,
                    Cell::Piece(p) => write!(f, "{}", p)?,
                }
            }
        }
        write!(f, ",{}", self.turn)?;
        Ok(())
    }

    fn turn(&self) -> GameTurn {
        if let Some(_) = self.win() {
            GameTurn::Finished
        } else {
            GameTurn::Turn(self.players[self.turn as usize])
        }
    }

    fn make_move(&mut self, player: UserId, move_to_make: &str) -> Result<(), String> {
        let p = if player == self.players[0] { 0 } else { 1 };

        let mut components = move_to_make.trim().split(' ');
        let x0 = parse_num(expect(components.next())?)?;
        let y0 = parse_num(expect(components.next())?)?;

        let pieces_left = self.count(p) != 3;
        if pieces_left {
            in_bounds(x0, y0)?;
            // make sure target is empty
            match self.board[y0][x0] {
                Cell::Empty => self.board[y0][x0] = Cell::Piece(p),
                _ => {
                    return Err(format!("target cell {} {} is not empty", x0, y0));
                }
            }
        } else {
            let x1 = parse_num(expect(components.next())?)?;
            let y1 = parse_num(expect(components.next())?)?;

            in_bounds(x0, y0)?;
            in_bounds(x1, y1)?;
            // make sure source is ours
            match self.board[y0][x0] {
                Cell::Piece(owner) if owner == p => {}
                _ => {
                    return Err(format!(
                        "source cell {} {} does not contain one of your pieces",
                        x0, y0
                    ))
                }
            }
            // make sure target is empty
            match self.board[y1][x1] {
                Cell::Empty => {}
                _ => {
                    return Err(format!("target cell {} {} is not empty", x1, y1));
                }
            }
            // move
            self.board[y0][x0] = Cell::Empty;
            self.board[y1][x1] = Cell::Piece(p);
        }

        if self.turn == 0 {
            self.turn = 1;
        } else {
            self.turn = 0;
        }

        Ok(())
    }

    fn end_state(&self) -> Option<GameState> {
        if let Some(p) = self.win() {
            Some(GameState::Win(self.players[p as usize]))
        } else {
            Some(GameState::InProgress)
        }
    }

    fn scores(&self) -> Option<GameScore> {
        None
    }
}

#[test]
fn three_mens_morris_test() {
    let game = ThreeMensMorrisGame();
    let instance = game.new(&vec![1, 2]);
    if let Some(mut inst) = instance {
        assert_eq!(inst.end_state(), Some(GameState::InProgress));
        assert_eq!(inst.turn(), GameTurn::Turn(1));

        assert_eq!(format!("{}", Fmt(|f| inst.serialize(f))), ".........,0");
        assert_eq!(
            inst.make_move(1, "0"),
            Err("expected another argument".to_string())
        );
        assert_eq!(inst.make_move(1, "0 0"), Ok(()));
        assert_eq!(format!("{}", Fmt(|f| inst.serialize(f))), "0........,1");

        assert_eq!(
            inst.make_move(2, "0 0"),
            Err("target cell 0 0 is not empty".to_string())
        );
        assert_eq!(inst.make_move(2, "0 1"), Ok(()));
        assert_eq!(format!("{}", Fmt(|f| inst.serialize(f))), "0..1.....,0");

        assert_eq!(inst.make_move(1, "1 0"), Ok(()));
        assert_eq!(format!("{}", Fmt(|f| inst.serialize(f))), "00.1.....,1");

        assert_eq!(inst.make_move(2, "1 1"), Ok(()));
        assert_eq!(format!("{}", Fmt(|f| inst.serialize(f))), "00.11....,0");

        assert_eq!(inst.make_move(1, "2 2"), Ok(()));
        assert_eq!(format!("{}", Fmt(|f| inst.serialize(f))), "00.11...0,1");

        assert_eq!(inst.make_move(2, "0 2"), Ok(()));
        assert_eq!(format!("{}", Fmt(|f| inst.serialize(f))), "00.11.1.0,0");

        assert_eq!(
            inst.make_move(1, "0 2"),
            Err("expected another argument".to_string())
        );
        assert_eq!(
            inst.make_move(1, "0 1 2 2"),
            Err("source cell 0 1 does not contain one of your pieces".to_string())
        );
        assert_eq!(
            inst.make_move(1, "2 2 0 0"),
            Err("target cell 0 0 is not empty".to_string())
        );

        assert_eq!(inst.make_move(1, "2 2 2 0"), Ok(()));
        assert_eq!(format!("{}", Fmt(|f| inst.serialize(f))), "00011.1..,1");

        assert_eq!(inst.turn(), GameTurn::Finished);
        assert_eq!(inst.end_state(), Some(GameState::Win(1)));
    } else {
        panic!("game should have been created")
    }
}
