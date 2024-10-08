// SPDX-FileCopyrightText: 2024 k4leg <pOgtq@yandex.com>
//
// SPDX-License-Identifier: GPL-3.0-or-later

pub struct ExhaustiveWordsIter<'a> {
    chars: &'a [char],
    current: Vec<usize>,
    length: usize,
    finished: bool,
}

impl<'a> ExhaustiveWordsIter<'a> {
    fn new(chars: &'a [char], length: usize) -> Self {
        Self {
            chars,
            current: vec![0; length],
            length,
            finished: length == 0,
        }
    }
}

impl<'a> Iterator for ExhaustiveWordsIter<'a> {
    type Item = String;

    fn next(&mut self) -> Option<Self::Item> {
        if self.finished {
            return None;
        }
        let res: String = self.current.iter().map(|&i| self.chars[i]).collect();
        for i in (0..self.length).rev() {
            if self.current[i] < self.chars.len() - 1 {
                self.current[i] += 1;
                return Some(res);
            }
            self.current[i] = 0;
        }
        self.finished = true;
        Some(res)
    }
}

pub trait GetExhaustiveWords<'a> {
    fn get_exhaustive_words(&'a self, n: usize) -> ExhaustiveWordsIter<'a>;
}

impl<'a> GetExhaustiveWords<'a> for [char] {
    fn get_exhaustive_words(&'a self, n: usize) -> ExhaustiveWordsIter<'a> {
        ExhaustiveWordsIter::new(self, n)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_1() {
        let s = &['a', 'b'];
        let mut i = s.get_exhaustive_words(2);
        assert_eq!(i.next().unwrap(), "aa");
        assert_eq!(i.next().unwrap(), "ab");
        assert_eq!(i.next().unwrap(), "ba");
        assert_eq!(i.next().unwrap(), "bb");
        assert_eq!(i.next(), None);
    }

    #[test]
    fn test_2() {
        let s = ['a', 'b'];
        let mut i = s.get_exhaustive_words(3);
        assert_eq!(i.next().unwrap(), "aaa");
        assert_eq!(i.next().unwrap(), "aab");
        assert_eq!(i.next().unwrap(), "aba");
        assert_eq!(i.next().unwrap(), "abb");
        assert_eq!(i.next().unwrap(), "baa");
        assert_eq!(i.next().unwrap(), "bab");
        assert_eq!(i.next().unwrap(), "bba");
        assert_eq!(i.next().unwrap(), "bbb");
        assert_eq!(i.next(), None);
    }

    #[test]
    fn test_3() {
        let s = &['a', 'b', 'c'];
        let mut i = s.get_exhaustive_words(2);
        assert_eq!(i.next().unwrap(), "aa");
        assert_eq!(i.next().unwrap(), "ab");
        assert_eq!(i.next().unwrap(), "ac");
        assert_eq!(i.next().unwrap(), "ba");
        assert_eq!(i.next().unwrap(), "bb");
        assert_eq!(i.next().unwrap(), "bc");
        assert_eq!(i.next().unwrap(), "ca");
        assert_eq!(i.next().unwrap(), "cb");
        assert_eq!(i.next().unwrap(), "cc");
        assert_eq!(i.next(), None);
    }

    #[test]
    fn test_4() {
        let s = &['a', 'b', 'c'];
        let mut i = s.get_exhaustive_words(3);
        assert_eq!(i.next().unwrap(), "aaa");
        assert_eq!(i.next().unwrap(), "aab");
        assert_eq!(i.next().unwrap(), "aac");
        assert_eq!(i.next().unwrap(), "aba");
        assert_eq!(i.next().unwrap(), "abb");
        assert_eq!(i.next().unwrap(), "abc");
        assert_eq!(i.next().unwrap(), "aca");
        assert_eq!(i.next().unwrap(), "acb");
        assert_eq!(i.next().unwrap(), "acc");
        assert_eq!(i.next().unwrap(), "baa");
        assert_eq!(i.next().unwrap(), "bab");
        assert_eq!(i.next().unwrap(), "bac");
        assert_eq!(i.next().unwrap(), "bba");
        assert_eq!(i.next().unwrap(), "bbb");
        assert_eq!(i.next().unwrap(), "bbc");
        assert_eq!(i.next().unwrap(), "bca");
        assert_eq!(i.next().unwrap(), "bcb");
        assert_eq!(i.next().unwrap(), "bcc");
        assert_eq!(i.next().unwrap(), "caa");
        assert_eq!(i.next().unwrap(), "cab");
        assert_eq!(i.next().unwrap(), "cac");
        assert_eq!(i.next().unwrap(), "cba");
        assert_eq!(i.next().unwrap(), "cbb");
        assert_eq!(i.next().unwrap(), "cbc");
        assert_eq!(i.next().unwrap(), "cca");
        assert_eq!(i.next().unwrap(), "ccb");
        assert_eq!(i.next().unwrap(), "ccc");
        assert_eq!(i.next(), None);
    }
}
