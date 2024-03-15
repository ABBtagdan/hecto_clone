use crate::highlight;
use crate::HighlightingOptions;
use crate::SearchDirection;
use std::cmp;
use termion::color;
use unicode_segmentation::UnicodeSegmentation;

#[derive(Default)]
pub struct Row {
    string: String,
    len: usize,
    highlighting: Vec<highlight::Type>,
}

impl From<&str> for Row {
    fn from(slice: &str) -> Self {
        Self {
            string: String::from(slice),
            len: slice.graphemes(true).count(),
            highlighting: Vec::new(),
        }
    }
}

impl Row {
    #[must_use]
    pub fn render(&self, start: usize, end: usize) -> String {
        let end = cmp::min(end, self.string.len());
        let start = cmp::min(start, end);
        let mut result = String::new();
        let mut current_highlighting = &highlight::Type::None;
        result.push_str(&format!("{}", termion::color::Fg(current_highlighting.to_color()))[..]);
        for (index, grapheme) in self.string[..]
            .graphemes(true)
            .enumerate()
            .skip(start)
            .take(end - start)
        {
            if let Some(c) = grapheme.chars().next() {
                let highlighting_type = self
                    .highlighting
                    .get(index)
                    .unwrap_or(&highlight::Type::None);
                if current_highlighting != highlighting_type {
                    current_highlighting = highlighting_type;
                    result.push_str(
                        &format!("{}", termion::color::Fg(highlighting_type.to_color()))[..],
                    );
                }
                if c == '\t' {
                    result.push_str(" ");
                } else {
                    result.push(c);
                }
            }
        }
        result.push_str(&format!("{}", termion::color::Fg(color::Reset))[..]);
        result
    }

    pub fn highlight_match(&mut self, word: Option<&str>) {
        if let Some(word) = word {
            if word.is_empty() {
                return;
            }
            let mut index = 0;
            while let Some(search_match) = self.search(word, index, SearchDirection::Forward) {
                if let Some(next_index) = search_match.checked_add(word[..].graphemes(true).count())
                {
                    for i in search_match..next_index {
                        self.highlighting[i] = highlight::Type::Match;
                    }
                    index = next_index;
                } else {
                    break;
                }
            }
        }
    }

    fn highlight_str(
        &mut self,
        index: &mut usize,
        substring: &str,
        chars: &[char],
        hl_type: highlight::Type,
    ) -> bool {
        if substring.is_empty() {
            return false;
        }
        for (substring_index, c) in substring.chars().enumerate() {
            if let Some(next_char) = chars.get(index.saturating_add(substring_index)) {
                if *next_char != c {
                    return false;
                }
            } else {
                return false;
            }
        }
        return false;
    }

    fn highlight_char(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.chars() && c == '\'' {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                let closing_index = if *next_char == '\\' {
                    index.saturating_add(3)
                } else {
                    index.saturating_add(2)
                };
                if let Some(closing_char) = chars.get(closing_index) {
                    if *closing_char == '\'' {
                        for _ in 0..=closing_index.saturating_sub(*index) {
                            self.highlighting.push(highlight::Type::Char);
                            *index += 1;
                        }
                        return true;
                    }
                }
            }
        }
        false
    }

    fn highlight_comment(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.comments() && c == '/' && *index < chars.len() {
            if let Some(next_char) = chars.get(index.saturating_add(1)) {
                if *next_char == '/' {
                    for _ in *index..chars.len() {
                        self.highlighting.push(highlight::Type::Comment);
                        *index += 1;
                    }
                    return true;
                }
            }
        }
        false
    }

    fn highlight_string(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.strings() && c == '"' {
            loop {
                self.highlighting.push(highlight::Type::String);
                *index += 1;
                if let Some(next_char) = chars.get(*index) {
                    if *next_char == '\\' {
                        self.highlighting.push(highlight::Type::String);
                        *index += 1;
                        continue;
                    }
                    if *next_char == '"' {
                        break;
                    }
                } else {
                    break;
                }
            }
            self.highlighting.push(highlight::Type::String);
            *index += 1;
            return true;
        }
        false
    }

    fn highlight_number(
        &mut self,
        index: &mut usize,
        opts: &HighlightingOptions,
        c: char,
        chars: &[char],
    ) -> bool {
        if opts.numbers() && c.is_ascii_digit() {
            if *index > 0 {
                let prev_char = chars[*index - 1];
                if !prev_char.is_ascii_punctuation() && !prev_char.is_ascii_whitespace() {
                    return false;
                }
            }
            loop {
                self.highlighting.push(highlight::Type::Number);
                *index += 1;
                if let Some(next_char) = chars.get(*index) {
                    if *next_char != '.' && !next_char.is_ascii_digit() {
                        break;
                    }
                } else {
                    break;
                }
            }
            return true;
        }
        false
    }

    pub fn highlight(&mut self, opts: &HighlightingOptions, word: Option<&str>) {
        self.highlighting = Vec::new();
        let chars: Vec<char> = self.string.chars().collect();
        let mut index = 0;
        while let Some(c) = chars.get(index) {
            if self.highlight_char(&mut index, opts, *c, &chars)
                || self.highlight_string(&mut index, opts, *c, &chars)
                || self.highlight_comment(&mut index, opts, *c, &chars)
                || self.highlight_number(&mut index, opts, *c, &chars)
            {
                continue;
            }
            self.highlighting.push(highlight::Type::None);
            index += 1;
        }
        self.highlight_match(word);
    }

    pub fn insert(&mut self, c: char, at: usize) {
        if at >= self.len() {
            self.string.push(c);
            self.len += 1;
            return;
        }
        let mut result = String::new();
        let mut length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            length += 1;
            if index == at {
                length += 1;
                result.push(c);
            }
            result.push_str(grapheme);
        }
        self.len = length;
        self.string = result;
    }

    pub fn delete(&mut self, at: usize) {
        if at >= self.len() {
            return;
        }
        let mut result = String::new();
        let mut length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index != at {
                length += 1;
                result.push_str(grapheme);
            }
        }
        self.len = length;
        self.string = result;
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.len
    }
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    pub fn append(&mut self, row: &Self) {
        self.string = format!("{}{}", self.string, row.string);
        self.len += row.len;
    }

    pub fn split(&mut self, at: usize) -> Self {
        let mut row = String::new();
        let mut length = 0;
        let mut splitted_row = String::new();
        let mut splitted_length = 0;
        for (index, grapheme) in self.string[..].graphemes(true).enumerate() {
            if index < at {
                length += 1;
                row.push_str(grapheme);
            } else {
                splitted_length += 1;
                splitted_row.push_str(grapheme);
            }
        }
        self.string = row;
        self.len = length;
        Self {
            string: splitted_row,
            len: splitted_length,
            highlighting: Vec::new(),
        }
    }

    pub fn search(
        &self,
        match_string: &str,
        at: usize,
        direction: SearchDirection,
    ) -> Option<usize> {
        if at > self.len || match_string.is_empty() {
            return None;
        }
        let start = if direction == SearchDirection::Forward {
            at
        } else {
            0
        };
        let end = if direction == SearchDirection::Forward {
            self.len
        } else {
            at
        };
        let substring: String = self.string[..]
            .graphemes(true)
            .skip(start)
            .take(end - start)
            .collect();
        let matching_byte_index = if direction == SearchDirection::Forward {
            substring.find(match_string)
        } else {
            substring.rfind(match_string)
        };
        if let Some(matching_byte_index) = matching_byte_index {
            for (grapheme_index, (byte_index, _)) in
                substring[..].grapheme_indices(true).enumerate()
            {
                if matching_byte_index == byte_index {
                    return Some(start + grapheme_index);
                }
            }
        }
        None
    }

    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        self.string.as_bytes()
    }
}
