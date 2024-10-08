// SPDX-FileCopyrightText: 2024 k4leg <pOgtq@yandex.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub const BLANK_CHAR: char = '\u{03BB}'; // Lambda.

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum Cell {
    Blank,
    Symbol(char),
}

impl From<Cell> for char {
    fn from(value: Cell) -> Self {
        match value {
            Cell::Blank => BLANK_CHAR,
            Cell::Symbol(ch) => ch,
        }
    }
}

impl From<char> for Cell {
    fn from(value: char) -> Self {
        match value {
            BLANK_CHAR => Self::Blank,
            _ => Self::Symbol(value),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        assert_eq!(Cell::Blank, Cell::Blank);
    }

    #[test]
    fn test_2() {
        assert_eq!(Cell::Symbol('a'), Cell::Symbol('a'));
    }

    #[test]
    fn test_3() {
        assert_ne!(Cell::Symbol('a'), Cell::Symbol('b'));
    }

    #[test]
    fn test_4() {
        assert_ne!(Cell::Symbol('a'), Cell::Blank);
    }

    #[test]
    fn test_5() {
        let x: char = Cell::Blank.into();
        assert_eq!(x, BLANK_CHAR);
    }

    #[test]
    fn test_6() {
        let x: char = Cell::Symbol('a').into();
        assert_eq!(x, 'a');
    }

    #[test]
    fn test_7() {
        let x: Cell = 'a'.into();
        assert_eq!(x, Cell::Symbol('a'));
    }

    #[test]
    fn test_8() {
        let x: Cell = BLANK_CHAR.into();
        assert_eq!(x, Cell::Blank);
    }
}
