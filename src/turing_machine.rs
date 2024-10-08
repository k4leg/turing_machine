// SPDX-FileCopyrightText: 2024 k4leg <pOgtq@yandex.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use anyhow::{anyhow, Result};

use self::core::{Command, Direction, MultiCommand, Program};
use self::tape::Tape;

pub mod cell;
pub mod core;
pub mod tape;

pub struct TuringMachine {
    tapes: Vec<Tape>,
    current_state: String,
    program: Program,
}

impl TuringMachine {
    pub fn new(start_tapes: &[&str], start_state: String, program: Program) -> Result<Self> {
        let length = match program.values().nth(0) {
            Some(instructions) => match instructions.keys().nth(0) {
                Some(cells) => cells.len(),
                None => return Err(anyhow!("there's no instructions")),
            },
            None => return Err(anyhow!("there's no instructions")),
        };
        if length != start_tapes.len() {
            return Err(anyhow!("length of instructions and tapes does not equal"));
        }
        for instructions in program.values() {
            for (icells, (_, ocells, directions)) in instructions {
                if length != icells.len() || length != ocells.len() || length != directions.len() {
                    return Err(anyhow!("invalid instructions length"));
                }
            }
        }
        let mut tapes = Vec::new();
        for &i in start_tapes {
            tapes.push(Tape::from(i));
        }
        Ok(Self {
            tapes,
            current_state: start_state,
            program,
        })
    }

    pub fn from(start_tape: &str, commands: Vec<Command>) -> Result<Self> {
        let start_state = match commands.first() {
            Some(cmd) => cmd.istate.to_owned(),
            None => return Err(anyhow!("no commands")),
        };
        let mut program = Program::new();
        for cmd in commands {
            program.entry(cmd.istate).or_default().insert(
                vec![cmd.icell],
                (cmd.ostate, vec![cmd.ocell], vec![cmd.direction]),
            );
        }
        Ok(Self {
            tapes: vec![Tape::from(start_tape)],
            current_state: start_state,
            program,
        })
    }

    pub fn from_multi(start_tapes: &[&str], commands: Vec<MultiCommand>) -> Result<Self> {
        let length = match commands.first() {
            Some(c) => c.len(),
            None => return Err(anyhow!("no commands")),
        };
        let start_state = commands[0].istate.to_owned();
        if length != start_tapes.len() {
            return Err(anyhow!("invalid tapes length"));
        }
        let mut program = Program::new();
        for cmd in commands {
            if length != cmd.len() {
                return Err(anyhow!("invalid tapes length"));
            }
            let (istate, icells, ostate, ocells, directions) = cmd.unpack();
            program
                .entry(istate)
                .or_default()
                .insert(icells, (ostate, ocells, directions));
        }
        let mut tapes = Vec::new();
        for &i in start_tapes {
            tapes.push(Tape::from(i));
        }
        Ok(Self {
            tapes,
            current_state: start_state,
            program,
        })
    }

    pub fn restart(&mut self, start_tapes: &[&str], start_state: String) -> Result<()> {
        if start_tapes.len() != self.tapes.len() {
            return Err(anyhow!("invalid start tapes"));
        }
        for (n, &i) in start_tapes.iter().enumerate() {
            self.tapes[n] = Tape::from(i);
        }
        self.current_state = start_state;
        Ok(())
    }

    pub fn to_strings(&self) -> Vec<String> {
        let mut strings = Vec::new();
        for tape in &self.tapes {
            strings.push(tape.to_string_with_state(&self.current_state));
        }
        strings
    }
}

impl Iterator for TuringMachine {
    type Item = Vec<String>;

    fn next(&mut self) -> Option<Self::Item> {
        let instructions = match self.program.get(&self.current_state) {
            Some(i) => i,
            None => return None,
        };
        let icells: Vec<_> = self
            .tapes
            .iter()
            .map(|tape| tape.get().to_owned())
            .collect();
        let (state, ocells, directions) = match instructions.get(&icells) {
            Some(v) => v,
            None => return None,
        };
        for (tape, (&cell, direction)) in self.tapes.iter_mut().zip(ocells.iter().zip(directions)) {
            tape.write(cell);
            match direction {
                Direction::Left => tape.left(),
                Direction::None => {}
                Direction::Right => tape.right(),
            }
        }
        self.current_state = state.to_owned();
        let strings = self.to_strings();
        Some(strings)
    }
}

#[cfg(test)]
mod tests {
    use crate::turing_machine::cell::{Cell, BLANK_CHAR};
    use crate::turing_machine::core::Instructions;

    use super::*;

    #[test]
    fn test_1() {
        let program = Program::from([(
            "q0".into(),
            Instructions::from([(
                vec![Cell::Symbol('a')],
                ("q0".into(), vec![Cell::Symbol('b')], vec![Direction::Right]),
            )]),
        )]);
        let mut tm = TuringMachine::new(&["aaa"], "q0".into(), program).unwrap();
        assert_eq!(tm.to_strings(), vec!["q0aaa"]);
        assert_eq!(tm.next(), Some(vec!["bq0aa".into()]));
        assert_eq!(tm.next(), Some(vec!["bbq0a".into()]));
        assert_eq!(tm.next(), Some(vec![format!("bbbq0{BLANK_CHAR}")]));
        assert_eq!(tm.next(), None);
    }

    #[test]
    fn test_2() {
        let program = Program::from([
            (
                "q0".into(),
                Instructions::from([
                    (
                        vec![Cell::Symbol('0')],
                        ("q0".into(), vec![Cell::Symbol('1')], vec![Direction::Right]),
                    ),
                    (
                        vec![Cell::Symbol('1')],
                        ("q0".into(), vec![Cell::Symbol('0')], vec![Direction::Right]),
                    ),
                    (
                        vec![Cell::Blank],
                        ("q1".into(), vec![Cell::Blank], vec![Direction::Left]),
                    ),
                ]),
            ),
            (
                "q1".into(),
                Instructions::from([
                    (
                        vec![Cell::Symbol('0')],
                        ("q1".into(), vec![Cell::Symbol('0')], vec![Direction::Left]),
                    ),
                    (
                        vec![Cell::Symbol('1')],
                        ("q1".into(), vec![Cell::Symbol('1')], vec![Direction::Left]),
                    ),
                    (
                        vec![Cell::Blank],
                        ("qz".into(), vec![Cell::Blank], vec![Direction::Right]),
                    ),
                ]),
            ),
        ]);
        let mut tm = TuringMachine::new(&["101101"], "q0".into(), program).unwrap();
        assert_eq!(tm.to_strings(), vec!["q0101101"]);
        assert_eq!(tm.next().unwrap(), vec!["0q001101"]);
        assert_eq!(tm.next().unwrap(), vec!["01q01101"]);
        assert_eq!(tm.next().unwrap(), vec!["010q0101"]);
        assert_eq!(tm.next().unwrap(), vec!["0100q001"]);
        assert_eq!(tm.next().unwrap(), vec!["01001q01"]);
        assert_eq!(tm.next().unwrap(), vec![format!("010010q0{BLANK_CHAR}")]);
        assert_eq!(tm.next().unwrap(), vec!["01001q10"]);
        assert_eq!(tm.next().unwrap(), vec!["0100q110"]);
        assert_eq!(tm.next().unwrap(), vec!["010q1010"]);
        assert_eq!(tm.next().unwrap(), vec!["01q10010"]);
        assert_eq!(tm.next().unwrap(), vec!["0q110010"]);
        assert_eq!(tm.next().unwrap(), vec!["q1010010"]);
        assert_eq!(tm.next().unwrap(), vec![format!("q1{BLANK_CHAR}010010")]);
        assert_eq!(tm.next().unwrap(), vec!["qz010010"]);
        assert_eq!(tm.next(), None);
    }

    #[test]
    fn test_3() {
        let mut tm = TuringMachine::from(
            "aaa",
            vec![Command::new(
                "q0".into(),
                Cell::Symbol('a'),
                "q0".into(),
                Cell::Symbol('b'),
                Direction::Right,
            )],
        )
        .unwrap();
        assert_eq!(tm.to_strings(), vec!["q0aaa"]);
        assert_eq!(tm.next(), Some(vec!["bq0aa".into()]));
        assert_eq!(tm.next(), Some(vec!["bbq0a".into()]));
        assert_eq!(tm.next(), Some(vec![format!("bbbq0{BLANK_CHAR}")]));
        assert_eq!(tm.next(), None);
    }

    #[test]
    fn test_4() {
        let cmds = vec![
            Command::new(
                "q0".into(),
                Cell::Symbol('0'),
                "q0".into(),
                Cell::Symbol('1'),
                Direction::Right,
            ),
            Command::new(
                "q0".into(),
                Cell::Symbol('1'),
                "q0".into(),
                Cell::Symbol('0'),
                Direction::Right,
            ),
            Command::new(
                "q0".into(),
                Cell::Blank,
                "q1".into(),
                Cell::Blank,
                Direction::Left,
            ),
            Command::new(
                "q1".into(),
                Cell::Symbol('0'),
                "q1".into(),
                Cell::Symbol('0'),
                Direction::Left,
            ),
            Command::new(
                "q1".into(),
                Cell::Symbol('1'),
                "q1".into(),
                Cell::Symbol('1'),
                Direction::Left,
            ),
            Command::new(
                "q1".into(),
                Cell::Blank,
                "qz".into(),
                Cell::Blank,
                Direction::Right,
            ),
        ];
        let mut tm = TuringMachine::from("101101", cmds).unwrap();
        assert_eq!(tm.to_strings(), vec!["q0101101"]);
        assert_eq!(tm.next().unwrap(), vec!["0q001101"]);
        assert_eq!(tm.next().unwrap(), vec!["01q01101"]);
        assert_eq!(tm.next().unwrap(), vec!["010q0101"]);
        assert_eq!(tm.next().unwrap(), vec!["0100q001"]);
        assert_eq!(tm.next().unwrap(), vec!["01001q01"]);
        assert_eq!(tm.next().unwrap(), vec![format!("010010q0{BLANK_CHAR}")]);
        assert_eq!(tm.next().unwrap(), vec!["01001q10"]);
        assert_eq!(tm.next().unwrap(), vec!["0100q110"]);
        assert_eq!(tm.next().unwrap(), vec!["010q1010"]);
        assert_eq!(tm.next().unwrap(), vec!["01q10010"]);
        assert_eq!(tm.next().unwrap(), vec!["0q110010"]);
        assert_eq!(tm.next().unwrap(), vec!["q1010010"]);
        assert_eq!(tm.next().unwrap(), vec![format!("q1{BLANK_CHAR}010010")]);
        assert_eq!(tm.next().unwrap(), vec!["qz010010"]);
        assert_eq!(tm.next(), None);
    }
}
