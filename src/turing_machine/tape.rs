// SPDX-FileCopyrightText: 2024 k4leg <pOgtq@yandex.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

use std::collections::HashMap;
use std::fmt;
use std::iter::Iterator;

use super::cell::Cell;

pub struct TapeIter<'a> {
    tape: &'a HashMap<isize, Cell>,
    index: isize,
    max: isize,
}

impl<'a> TapeIter<'a> {
    pub fn new(tape: &'a HashMap<isize, Cell>, min: isize, max: isize, head: isize) -> Self {
        Self {
            tape,
            index: if head < min { head } else { min },
            max: if head > max { head } else { max },
        }
    }
}

impl<'a> Iterator for TapeIter<'a> {
    type Item = Cell;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index > self.max {
            return None;
        }
        let cell = self.tape.get(&self.index);
        self.index += 1;
        match cell {
            Some(c) => Some(*c),
            None => Some(Cell::Blank),
        }
    }
}

#[derive(PartialEq)]
pub struct Tape {
    tape: HashMap<isize, Cell>,
    min: isize,
    max: isize,
    head: isize,
}

impl Tape {
    pub fn new() -> Self {
        Self {
            tape: HashMap::new(),
            min: 0,
            max: 0,
            head: 0,
        }
    }

    pub fn left(&mut self) {
        self.head -= 1;
        self.trim();
    }

    pub fn right(&mut self) {
        self.head += 1;
        self.trim();
    }

    pub fn get(&self) -> &Cell {
        match self.tape.get(&self.head) {
            Some(cell) => cell,
            None => &Cell::Blank,
        }
    }

    pub fn write(&mut self, cell: Cell) {
        match cell {
            Cell::Blank => {
                self.tape.remove(&self.head);
            }
            Cell::Symbol(_) => {
                self.tape.insert(self.head, cell);
                if self.head > self.max {
                    self.max = self.head;
                } else if self.head < self.min {
                    self.min = self.head;
                }
            }
        }
        self.trim();
    }

    fn trim(&mut self) {
        if self.head >= self.min {
            if self.head > self.max {
                self.max = self.head;
            }
            while self.min < self.max && !self.tape.contains_key(&self.min) {
                self.min += 1;
            }
            while self.min < self.max && !self.tape.contains_key(&self.max) {
                self.max -= 1;
            }
        } else {
            if self.head < self.min {
                self.min = self.head;
            }
            while self.min < self.max && !self.tape.contains_key(&self.max) {
                self.max -= 1;
            }
            while self.min < self.max && !self.tape.contains_key(&self.min) {
                self.min += 1;
            }
        }
    }

    pub fn iter(&self) -> TapeIter {
        TapeIter::new(&self.tape, self.min, self.max, self.head)
    }

    pub fn len(&self) -> usize {
        (self.max - self.min + 1) as usize
    }

    pub fn to_string_with_state(&self, state: &str) -> String {
        let mut s = self.to_string();
        let idx = if self.head >= self.min {
            (self.head - self.min).unsigned_abs()
        } else {
            0
        };
        s.insert_str(s.char_indices().collect::<Vec<_>>()[idx].0, state);
        s
    }
}

impl fmt::Display for Tape {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        for i in self.iter() {
            s.push(i.into());
        }
        s.fmt(f)
    }
}

impl From<&str> for Tape {
    fn from(value: &str) -> Self {
        let mut tape = HashMap::new();
        for (n, ch) in value.chars().enumerate() {
            tape.insert(n as isize, ch.into());
        }
        Self {
            tape,
            min: 0,
            max: value.len() as isize - 1,
            head: 0,
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::turing_machine::cell::BLANK_CHAR;

    use super::*;

    #[test]
    fn test_to_string_1() {
        let mut t = Tape::new();
        t.write('a'.into());
        t.right();
        t.write('b'.into());
        assert_eq!(t.to_string(), "ab");
    }

    #[test]
    fn test_to_string_2() {
        let t = Tape::new();
        assert_eq!(t.to_string(), format!("{BLANK_CHAR}"));
    }

    #[test]
    fn test_to_string_3() {
        let mut t = Tape::new();
        t.left();
        t.write(Cell::Symbol('a'));
        assert_eq!(t.to_string(), "a");
    }

    #[test]
    fn test_to_string_4() {
        let mut t = Tape::new();
        t.left();
        t.left();
        assert_eq!(t.to_string(), format!("{BLANK_CHAR}"));
    }

    #[test]
    fn test_to_string_5() {
        let mut t = Tape::new();
        t.left();
        t.left();
        t.write(Cell::Symbol('a'));
        assert_eq!(t.to_string(), "a");
    }

    #[test]
    fn test_to_string_6() {
        let mut t = Tape::new();
        t.write(Cell::Symbol('b'));
        t.left();
        t.left();
        t.left();
        t.write(Cell::Symbol('a'));
        assert_eq!(t.to_string(), format!("a{BLANK_CHAR}{BLANK_CHAR}b"));
    }

    #[test]
    fn test_to_string_with_state_1() {
        let mut t = Tape::new();
        t.left();
        assert_eq!(t.to_string_with_state("q0"), format!("q0{BLANK_CHAR}"));
    }

    #[test]
    fn test_to_string_with_state_2() {
        let mut t = Tape::new();
        t.left();
        t.left();
        assert_eq!(t.to_string_with_state("q0"), format!("q0{BLANK_CHAR}"));
    }

    #[test]
    fn test_to_string_with_state_3() {
        let t = Tape::new();
        assert_eq!(t.to_string_with_state("q0"), format!("q0{BLANK_CHAR}"));
    }

    #[test]
    fn test_to_string_with_state_4() {
        let mut t = Tape::new();
        t.write(Cell::Blank);
        assert_eq!(t.to_string_with_state("q0"), format!("q0{BLANK_CHAR}"));
    }

    #[test]
    fn test_to_string_with_state_5() {
        let mut t = Tape::new();
        t.write(Cell::Symbol('a'));
        assert_eq!(t.to_string_with_state("q0"), "q0a");
    }

    #[test]
    fn test_to_string_with_state_6() {
        let mut t = Tape::new();
        t.write(Cell::Symbol('a'));
        t.right();
        t.write(Cell::Symbol('b'));
        assert_eq!(t.to_string_with_state("q0"), "aq0b");
    }

    #[test]
    fn test_to_string_with_state_7() {
        let mut t = Tape::new();
        t.write(Cell::Symbol('a'));
        t.right();
        t.write(Cell::Blank);
        assert_eq!(t.to_string_with_state("q0"), format!("aq0{BLANK_CHAR}"));
    }

    #[test]
    fn test_to_string_with_state_8() {
        let mut t = Tape::new();
        t.write(Cell::Symbol('a'));
        t.right();
        t.right();
        t.right();
        assert_eq!(
            t.to_string_with_state("q0"),
            format!("a{BLANK_CHAR}{BLANK_CHAR}q0{BLANK_CHAR}")
        );
    }

    #[test]
    fn test_to_string_with_state_9() {
        let mut t = Tape::new();
        t.write(Cell::Symbol('a'));
        t.right();
        t.right();
        t.right();
        t.write(Cell::Symbol('b'));
        assert_eq!(
            t.to_string_with_state("q0"),
            format!("a{BLANK_CHAR}{BLANK_CHAR}q0b")
        );
    }

    #[test]
    fn test_get_1() {
        let t = Tape::new();
        assert_eq!(t.get(), &Cell::Blank);
    }

    #[test]
    fn test_get_2() {
        let mut t = Tape::new();
        t.write(Cell::Symbol('a'));
        assert_eq!(t.get(), &Cell::Symbol('a'));
    }

    #[test]
    fn test_get_3() {
        let mut t = Tape::new();
        t.right();
        t.write(Cell::Symbol('a'));
        assert_eq!(t.get(), &Cell::Symbol('a'));
    }

    #[test]
    fn test_get_4() {
        let mut t = Tape::new();
        t.right();
        t.write(Cell::Symbol('a'));
        t.right();
        assert_eq!(t.get(), &Cell::Blank);
    }

    #[test]
    fn test_get_5() {
        let mut t = Tape::new();
        t.right();
        t.write(Cell::Symbol('a'));
        t.right();
        t.left();
        assert_eq!(t.get(), &Cell::Symbol('a'));
    }

    #[test]
    fn test_get_6() {
        let mut t = Tape::new();
        t.right();
        t.write(Cell::Symbol('a'));
        t.right();
        t.left();
        t.left();
        assert_eq!(t.get(), &Cell::Blank);
    }

    #[test]
    fn test_from_1() {
        let t = Tape::from("");
        assert_eq!(t.to_string_with_state("q0"), format!("q0{BLANK_CHAR}"));
    }

    #[test]
    fn test_from_2() {
        let t = Tape::from("aaa");
        assert_eq!(t.to_string_with_state("q0"), "q0aaa");
    }

    #[test]
    fn test_from_3() {
        let mut t = Tape::from("abcd");
        t.right();
        t.write(Cell::Symbol('X'));
        assert_eq!(t.to_string_with_state("q0"), "aq0Xcd");
    }
}
