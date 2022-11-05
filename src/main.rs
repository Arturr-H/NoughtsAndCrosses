/*- Global allowings -*/
#![allow(
    dead_code,
    unused_variables
)]

/*- Imports -*/
use crossterm::{
    terminal::{
        disable_raw_mode,
        enable_raw_mode,
        Clear,
        ClearType
    },
    event::{ read, Event, KeyCode },
    ExecutableCommand,
    cursor
};
use std::{
    io::stdout,
    cmp::min
};

/*- Constants -*/
const GRID_SIZE:(usize, usize) = (5, 5);
const DEFAULT_XOE:&'static [&'static str] = &["❌" , "⭕️", "⬜️"];
const HOVERED_XOE:&'static [&'static str] = &["✖️ ", "🔘", "🔲"];
const DIRECTIONS:&'static [(i8, i8)] = &[(0, -1), (-1, -1), (-1, 0), (-1, 1)];
const WIN_LENGTH:u8 = 4;

/*- Structs, enums & unions -*/
pub struct Board {
    cells: Vec<Vec<Cell>>,
    title: Option<String>,
    player: Cell,
    cursor: (usize, usize)
}
#[derive(Clone, Debug)]
enum Cell { X, O, Empty }
enum Direction { L, R, U, D }

/*- Initialize -*/
fn main() -> () {
    let mut board:Board = Board::new(GRID_SIZE);
    let mut stdout = stdout();

    /*- Display -*/
    board.display();

    /*- For the crossterm lib, rawmode enables keypress recognization without enter key -*/
    enable_raw_mode().unwrap();

    'main: loop {
        match read().unwrap() {
            Event::Key(key) => {
                match key.code {

                    /*- Quit program -*/
                      KeyCode::Char('q')
                    | KeyCode::Esc => {
                        println!("Quitting...");
                        
                        /*- Clear terminal -*/
                        stdout
                            .execute(cursor::MoveToNextLine(1)).unwrap()
                            .execute(Clear(ClearType::CurrentLine)).unwrap();

                        break 'main;
                    },

                    /*- Move cursor -*/
                      KeyCode::Char('w')
                    | KeyCode::Char('a')
                    | KeyCode::Char('s')
                    | KeyCode::Char('d') => {
                        board.move_cursor(key.code);

                        /*- Refresh and continue -*/
                        board.display();
                    },

                    /*- Place tile -*/
                    KeyCode::Char(' ') => {
                        /*- Coordinates of cursor -*/
                        let (x, y) = (board.cursor.0, board.cursor.1);

                        /*- Try set tile -*/
                        match board.set(x, y, board.player.clone()) {
                            Ok(_) => (),
                            Err(tile) => {
                                board.title = Some(format!("Occupied by {}", tile.display(DEFAULT_XOE)));

                                /*- Display & continue -*/
                                board.display();
                                board.title = None;
                                continue;
                            }
                        };

                        /*- Check if there are any winners -*/
                        if board.check_win(board.cursor) == true {
                            println!("{} IS THE WINNER", board.player.display(DEFAULT_XOE));
                            return;
                        };

                        /*- Switch player -*/
                        board.toggle_player();

                        /*- Display -*/
                        board.display();
                        board.title = None;
                    }
                    _ => ()
                }
            },
            _ => ()
        };
    };

    /*- Disable raw mode after program finish -*/
    disable_raw_mode().unwrap();
}

/*- Method implementations -*/
impl Board {
    /*- Constructor -*/
    fn new(size:(usize, usize)) -> Self {
        Board {
            cells: vec![
                vec![
                    Cell::Empty; size.0
                ]; size.1
            ],
            player: Cell::X,
            cursor: (GRID_SIZE.0/2, GRID_SIZE.1/2),
            title: None,
        }
    }

    /*- Get cells -*/
    fn get(&self, x:usize, y:usize) -> Option<&Cell> {
        match self.cells.get(y) {
            Some(e) => e.get(x),
            None => None
        }
    }
    fn get_mut(&mut self, x:usize, y:usize) -> Option<&mut Cell> {
        match self.cells.get_mut(y) {
            Some(e) => e.get_mut(x),
            None => None
        }
    }

    /*- Set cell, returns which cell is blocking, or
        if any cell isn't blocking, return empty cell -*/
    fn set(&mut self, x:usize, y:usize, cell:Cell) -> Result<(), Cell> {
        match self.get(x, y) {
            Some(Cell::O) => Err(Cell::O),
            Some(Cell::X) => Err(Cell::X),
            Some(Cell::Empty) => {
                match self.get_mut(x, y) {
                    Some(e) => {
                        *e = cell;
                        Ok(())
                    },
                    None => Err(Cell::Empty)
                }
            },
            None => Err(Cell::Empty)
        }
    }

    /*- Parse user input -*/
    fn parse_input(&mut self, input:&String) -> Option<(usize, usize)> {
        let mut items = input.trim().split_ascii_whitespace();
        Some((
            items.next()?.parse::<usize>().ok()?,
            items.next()?.parse::<usize>().ok()?,
        ))
    }

    /*- Game functionality -*/
    fn toggle_player(&mut self) -> () {
        self.player = match self.player {
            Cell::X => Cell::O,
            Cell::O => Cell::X,
            Cell::Empty => unimplemented!()
        };
    }
    fn move_cursor(&mut self, direction:KeyCode) -> () {
        match direction {
            /*- Left -*/
            KeyCode::Char('a') => {
                if self.cursor.0 != 0 {
                    self.cursor.0 -= 1;
                }
            },

            /*- Right -*/
            KeyCode::Char('d') => self.cursor.0 = min(self.cursor.0 + 1, GRID_SIZE.0-1),

            /*- Up -*/
            KeyCode::Char('w') => {
                if self.cursor.1 != 0 {
                    self.cursor.1 -= 1;
                }
            },

            /*- Down -*/
            KeyCode::Char('s') => self.cursor.1 = min(self.cursor.1 + 1, GRID_SIZE.1-1),
            _ => ()
        };
    }

    /*- Did we win or not?! -*/
    fn check_win(&mut self, cursor_pos:(usize, usize)) -> bool {
        for increment in DIRECTIONS {
            let mut cursor_position:(usize, usize) = cursor_pos.clone();

            /*- We're 100% this will always be Some(_) -*/
            let current_tile:&Cell = self.get(cursor_position.0, cursor_position.1).unwrap();

            /*- Iterate backwards in incrementation -*/
            loop {
                if let Some(new_pos) = sum_coordinates(cursor_position, *increment) {
                    if let Some(tile) = self.get(new_pos.0, new_pos.1) {

                        /*- If we're on the same type of tile -*/
                        if tile == current_tile {
                            cursor_position = new_pos;
                        }else {
                            break;
                        }
                    }
                }else {
                    break;
                };
            };

            /*- Number of tiles which are in row -*/
            let mut count:u8 = 0;
            let new_direction = flip_direction(*increment);

            /*- Iterate forwards & keep track of number of same tiles -*/
            'forwards: loop {
                /*- Get tile -*/
                if let Some(tile) = self.get(cursor_position.0, cursor_position.1) {
                    if tile == current_tile {

                        /*- Increment position & count -*/
                        count += 1;
                        if let Some(new_pos) = sum_coordinates(
                            cursor_position,
                            new_direction
                        ) {
                            cursor_position = new_pos;
                        }else {
                            break 'forwards;
                        }
                    }else {
                        break 'forwards;
                    }
                }else {
                    break 'forwards;
                }
            };

            /*- Check if win or not -*/
            if count >= WIN_LENGTH { return true; };
        };

        false
    }

    /*- Show board -*/
    fn display(&self) -> () {
        println!("{:?}", self);
    }
    fn clear_title(&mut self) -> () {
        self.title = None;
    }
}

/*- Sum coordinates (used in win check) -*/
fn sum_coordinates(coord1:(usize, usize), direction:(i8, i8)) -> Option<(usize, usize)> {
    let mut final_:(usize, usize) = coord1;

    /*- If x is negative -*/
    if direction.0.is_negative() {

        /*- Check if x can go lower -*/
        match final_.0.checked_sub(direction.0.abs() as usize) {
            Some(_) => final_.0 -= 1,
            None => return None
        };
    }

    /*- If x will go out of bounds -*/
    else if coord1.0 + direction.0 as usize >= GRID_SIZE.0 {
        return None
    }

    /*- Else add to final -*/
    else {
        final_.0 += direction.0 as usize;
    };


    /*- If y is negative -*/
    if direction.1.is_negative() {

        /*- Check if y can go lower -*/
        match final_.1.checked_sub(direction.1.abs() as usize) {
            Some(_) => final_.1 -= 1,
            None => return None
        };
    }

    /*- If y will go out of bounds -*/
    else if coord1.1 + direction.1 as usize >= GRID_SIZE.1 {
        return None
    }

    /*- Else add to final -*/
    else {
        final_.1 += direction.1 as usize;
    };

    Some(final_)
}
fn flip_direction(direction:(i8, i8)) -> (i8, i8) {
    (direction.0 * -1, direction.1 * -1)
}

/*- Compare for Cell -*/
impl PartialEq for Cell {
    fn eq(&self, other:&Self) -> bool {
        match (self, other) {
            (Cell::X, Cell::X) => true,
            (Cell::O, Cell::O) => true,
            (Cell::Empty, Cell::Empty) => true,
            _ => false
        }
    }
}

/*- Better looking board in terminal -*/
impl std::fmt::Debug for Board {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s:String = String::new();
        stdout()
            .execute(cursor::MoveToNextLine(1)).unwrap()
            .execute(Clear(ClearType::All)).unwrap();
        for y in 0..self.cells.len() {
            for x in 0..self.cells[y].len() {
                if self.cursor == (x, y) {
                    s.push_str(Cell::get_display_string(self.get(x, y), HOVERED_XOE));
                }else {
                    s.push_str(Cell::get_display_string(self.get(x, y), DEFAULT_XOE));
                }
                s.push_str("");
            }
            s.push_str("\r\n");
        }

        /*- Get the board title which lies above the board (displays current status) -*/
        let title:String = {
            if let Some(title) = &self.title {
                title.to_string()
            }else {
                format!("{}'s turn!", self.player.display(DEFAULT_XOE))
            }
        };

        /*- Write to stdout -*/
        write!(f, "{}\r\n{}", title, s)
    }
}
impl Cell {
    fn display<'lf>(&'lf self, xoe:&'lf [&str]) -> &'lf str {
        match self {
            Cell::X => xoe[0],
            Cell::O => xoe[1],
            Cell::Empty => xoe[2],
        }
    }
    fn get_display_string<'la>(self_:Option<&'la Self>, xoe:&'la [&str]) -> &'la str {
        match self_ {
            Some(e) => e.display(xoe),
            None => " "
        }
    }
}
