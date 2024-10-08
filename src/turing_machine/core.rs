// SPDX-FileCopyrightText: 2024 k4leg <pOgtq@yandex.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fmt;

use anyhow::{anyhow, Result};

use super::cell::Cell;

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum Direction {
    Left,
    None,
    Right,
}

impl fmt::Display for Direction {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Left => "L".fmt(f),
            Self::None => "N".fmt(f),
            Self::Right => "R".fmt(f),
        }
    }
}

impl Direction {
    pub fn from_char(ch: char) -> Result<Self> {
        match ch {
            'L' => Ok(Self::Left),
            'N' => Ok(Self::None),
            'R' => Ok(Self::Right),
            _ => Err(anyhow!("invalid char")),
        }
    }
}

pub type Instructions = HashMap<Vec<Cell>, (String, Vec<Cell>, Vec<Direction>)>;
pub type Program = HashMap<String, Instructions>;

#[derive(Clone, Debug, PartialEq)]
pub struct Command {
    pub istate: String,
    pub icell: Cell,
    pub ostate: String,
    pub ocell: Cell,
    pub direction: Direction,
}

impl Command {
    pub fn new(
        istate: String,
        icell: Cell,
        ostate: String,
        ocell: Cell,
        direction: Direction,
    ) -> Self {
        Self {
            istate,
            icell,
            ostate,
            ocell,
            direction,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct MultiCommand {
    pub istate: String,
    icells: Vec<Cell>,
    pub ostate: String,
    ocells: Vec<Cell>,
    directions: Vec<Direction>,
    length: usize,
}

impl MultiCommand {
    pub fn new(
        istate: String,
        icells: Vec<Cell>,
        ostate: String,
        ocells: Vec<Cell>,
        directions: Vec<Direction>,
    ) -> Result<Self> {
        let length = icells.len();
        if length == 0 || length != ocells.len() || length != directions.len() {
            Err(anyhow!("invalid length"))
        } else {
            Ok(Self {
                istate,
                icells,
                ostate,
                ocells,
                directions,
                length,
            })
        }
    }

    pub fn add_tape(&mut self) {
        self.icells.push(Cell::Blank);
        self.ocells.push(Cell::Blank);
        self.directions.push(Direction::None);
        self.length += 1;
    }

    pub fn remove_tape(&mut self) {
        if self.length == 1 {
            return;
        }
        self.icells.pop();
        self.ocells.pop();
        self.directions.pop();
        self.length -= 1;
    }

    pub fn get_mut_icell(&mut self, n: usize) -> Option<&mut Cell> {
        self.icells.get_mut(n)
    }

    pub fn get_mut_ocell(&mut self, n: usize) -> Option<&mut Cell> {
        self.ocells.get_mut(n)
    }

    pub fn get_mut_direction(&mut self, n: usize) -> Option<&mut Direction> {
        self.directions.get_mut(n)
    }

    pub fn unpack(self) -> (String, Vec<Cell>, String, Vec<Cell>, Vec<Direction>) {
        (
            self.istate,
            self.icells,
            self.ostate,
            self.ocells,
            self.directions,
        )
    }

    pub fn len(&self) -> usize {
        self.length
    }
}

impl From<Command> for MultiCommand {
    fn from(value: Command) -> Self {
        Self {
            istate: value.istate,
            icells: vec![value.icell],
            ostate: value.ostate,
            ocells: vec![value.ocell],
            directions: vec![value.direction],
            length: 1,
        }
    }
}

#[macro_export]
macro_rules! tm_cmd {
    ($istate:literal, $icell:expr, $ostate:literal, $ocell:expr, $dir:literal $(,)?) => {
        $crate::turing_machine::core::Command::new(
            $istate.into(),
            $icell.into(),
            $ostate.into(),
            $ocell.into(),
            $crate::turing_machine::core::Direction::from_char($dir).unwrap(),
        )
    };
}

#[macro_export]
macro_rules! tm_cmds {
    ($([$istate:literal, $icell:expr, $ostate:literal, $ocell:expr, $dir:literal $(,)?]),* $(,)?) => {
        vec![$(tm_cmd!($istate, $icell, $ostate, $ocell, $dir)),*]
    };
}

#[macro_export]
macro_rules! tm_mcmd {
    ($istate:literal, [$($icell:expr),+ $(,)?], $ostate:literal, [$($ocell:expr),+ $(,)?], [$($dir:literal),+ $(,)?] $(,)?) => {
        $crate::turing_machine::core::MultiCommand::new(
            $istate.into(),
            vec![$($icell.into()),+],
            $ostate.into(),
            vec![$($ocell.into()),+],
            vec![$($crate::turing_machine::core::Direction::from_char($dir).unwrap()),+],
        )
        .unwrap()
    };
}

#[macro_export]
macro_rules! tm_mcmds {
    ($([$istate:literal, [$($icell:expr),+ $(,)?], $ostate:literal, [$($ocell:expr),+ $(,)?], [$($dir:literal),+ $(,)?] $(,)?]),+ $(,)?) => {
        {
            let res = vec![$(tm_mcmd!($istate, [$($icell),+], $ostate, [$($ocell),+], [$($dir),+])),+];
            let t = res[0].len();
            debug_assert!(res.iter().all(|cmd| cmd.len() == t));
            res
        }
    };
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let cmd = MultiCommand::new("q0".into(), vec![], "q0".into(), vec![], vec![]);
        assert!(matches!(cmd, Err(_)));
    }

    #[test]
    fn test_2() {
        let cmd = MultiCommand::new(
            "q0".into(),
            vec![Cell::Blank],
            "q0".into(),
            vec![Cell::Blank, Cell::Blank],
            vec![Direction::None],
        );
        assert!(matches!(cmd, Err(_)));
    }

    #[test]
    fn test_3() {
        let cmd = MultiCommand::new(
            "q".into(),
            vec![Cell::Blank, Cell::Blank],
            "q".into(),
            vec![Cell::Blank, Cell::Blank],
            vec![Direction::None, Direction::None],
        );
        assert!(matches!(cmd, Ok(_)));
        assert_eq!(cmd.unwrap().len(), 2);
    }

    #[test]
    #[should_panic]
    fn test_4() {
        MultiCommand::new(
            "q".into(),
            vec![Cell::Blank, Cell::Blank],
            "q".into(),
            vec![Cell::Blank, Cell::Blank],
            vec![Direction::None],
        )
        .unwrap();
    }

    #[test]
    fn test_tm_cmd() {
        let cmd1 = tm_cmd!("q0", '0', "q0", '1', 'R');
        let cmd2 = Command::new(
            "q0".into(),
            '0'.into(),
            "q0".into(),
            '1'.into(),
            Direction::Right,
        );
        assert_eq!(cmd1, cmd2);
    }

    #[test]
    fn test_tm_cmds() {
        let cmd1 = tm_cmds![["q0", '0', "q0", '1', 'R'], ["q0", '1', "q0", '0', 'R'],];
        let cmd2 = vec![
            Command::new(
                "q0".into(),
                '0'.into(),
                "q0".into(),
                '1'.into(),
                Direction::Right,
            ),
            Command::new(
                "q0".into(),
                '1'.into(),
                "q0".into(),
                '0'.into(),
                Direction::Right,
            ),
        ];
        assert_eq!(cmd1, cmd2);
    }

    #[test]
    fn test_tm_mcmd_1() {
        let mcmd1 = tm_mcmd!("q0", ['0', Cell::Blank], "q0", ['1', '0'], ['R', 'N']);
        let mcmd2 = MultiCommand::new(
            "q0".into(),
            vec!['0'.into(), Cell::Blank],
            "q0".into(),
            vec!['1'.into(), '0'.into()],
            vec![Direction::Right, Direction::None],
        )
        .unwrap();
        assert_eq!(mcmd1, mcmd2);
    }

    #[test]
    #[should_panic]
    fn test_tm_mcmd_2() {
        tm_mcmd!("q0", ['0'], "q0", ['1', '0'], ['R', 'N']);
    }

    #[test]
    fn test_tm_mcmds() {
        let mcmds1 = tm_mcmds![
            ["q0", ['0', Cell::Blank], "q0", ['1', '2'], ['R', 'L']],
            ["q1", ['2', '3'], "q1", [Cell::Blank, '2'], ['N', 'R']],
        ];
        let mcmds2 = vec![
            MultiCommand::new(
                "q0".into(),
                vec!['0'.into(), Cell::Blank],
                "q0".into(),
                vec!['1'.into(), '2'.into()],
                vec![Direction::Right, Direction::Left],
            )
            .unwrap(),
            MultiCommand::new(
                "q1".into(),
                vec!['2'.into(), '3'.into()],
                "q1".into(),
                vec![Cell::Blank, '2'.into()],
                vec![Direction::None, Direction::Right],
            )
            .unwrap(),
        ];
        assert_eq!(mcmds1, mcmds2);
    }
}
