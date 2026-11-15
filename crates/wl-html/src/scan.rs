// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Lightweight source scanner producing exact byte spans for tags,
//! attributes, raw-text elements and comments.
//!
//! `html5ever` tokens carry no positions, so this scanner walks the source
//! once in O(n) and its events are aligned with the parsed DOM in document
//! order. The scanner is deliberately permissive: malformed markup yields
//! events, never errors.

use wl_core::Span;

/// Attribute occurrence in the source.
#[derive(Debug, Clone)]
pub struct ScanAttr {
    /// Lowercased attribute name.
    pub name:       String,
    /// Decoded attribute value.
    pub value:      String,
    /// Span of the whole attribute.
    pub span:       Span,
    /// Span of the value without quotes.
    pub value_span: Option<Span>
}

/// Start tag occurrence in the source.
#[derive(Debug, Clone)]
pub struct StartTag {
    /// Lowercased tag name.
    pub name:         String,
    /// Attributes in source order.
    pub attrs:        Vec<ScanAttr>,
    /// Span of the whole tag, from `<` to `>`.
    pub span:         Span,
    /// Span of the tag name.
    pub name_span:    Span,
    /// Whether the tag ended with `/>`.
    pub self_closing: bool,
    /// Content span for raw-text elements.
    pub raw_text:     Option<Span>
}

/// One lexical event in document order.
#[derive(Debug, Clone)]
pub enum ScanEvent {
    /// A start tag.
    Start(StartTag),
    /// An end tag.
    End {
        /// Lowercased tag name.
        name: String,
        /// Span of the whole end tag.
        span: Span
    },
    /// A comment.
    Comment {
        /// Span of the whole comment.
        span:      Span,
        /// Span of the text inside the delimiters.
        text_span: Span
    },
    /// A doctype or bogus markup declaration.
    Doctype {
        /// Span of the declaration.
        span: Span
    },
    /// A run of text between markup.
    Text {
        /// Span of the text.
        span: Span
    }
}

/// Elements whose content is raw text, terminated by the matching end tag.
const RAW_TEXT: [&str; 8] = [
    "script", "style", "textarea", "title", "xmp", "iframe", "noembed", "noframes"
];

/// Scans `source` into events in document order.
///
/// # Examples
///
/// ```
/// use wl_html::scan::{ScanEvent, scan};
/// let events = scan("<b>x</b>");
/// assert!(matches!(events.first(), Some(ScanEvent::Start(_))));
/// ```
#[must_use]
pub fn scan(source: &str) -> Vec<ScanEvent> {
    Scanner {
        src:    source.as_bytes(),
        text:   source,
        pos:    0,
        events: Vec::new()
    }
    .run()
}

/// Builds a span from byte offsets; inputs are capped far below `u32::MAX`.
fn sp(start: usize, end: usize) -> Span {
    Span::new(
        u32::try_from(start).unwrap_or(u32::MAX),
        u32::try_from(end).unwrap_or(u32::MAX)
    )
}

struct Scanner<'a> {
    src:    &'a [u8],
    text:   &'a str,
    pos:    usize,
    events: Vec<ScanEvent>
}

impl Scanner<'_> {
    fn run(mut self) -> Vec<ScanEvent> {
        let mut text_start = 0_usize;
        while self.pos < self.src.len() {
            if self.at(self.pos) == Some(b'<') {
                self.flush_text(text_start, self.pos);
                if self.starts_with(b"<!--") {
                    self.comment();
                } else if self.starts_with(b"</") {
                    self.end_tag();
                } else if self.starts_with(b"<!") || self.starts_with(b"<?") {
                    self.bogus();
                } else if self
                    .at(self.pos + 1)
                    .is_some_and(|b| b.is_ascii_alphabetic())
                {
                    self.start_tag();
                } else {
                    self.pos += 1;
                    self.flush_text(self.pos - 1, self.pos);
                }
                text_start = self.pos;
            } else {
                self.pos += 1;
            }
        }
        self.flush_text(text_start, self.pos);
        self.events
    }

    fn at(&self, index: usize) -> Option<u8> {
        self.src.get(index).copied()
    }

    fn starts_with(&self, pattern: &[u8]) -> bool {
        self.src.get(self.pos..self.pos + pattern.len()) == Some(pattern)
    }

    fn flush_text(&mut self, start: usize, end: usize) {
        if end > start {
            self.events.push(ScanEvent::Text {
                span: sp(start, end)
            });
        }
    }

    fn skip_ws(&self, mut index: usize) -> usize {
        while self.at(index).is_some_and(|b| b.is_ascii_whitespace()) {
            index += 1;
        }
        index
    }

    fn comment(&mut self) {
        let start = self.pos;
        let body = start + 4;
        let body_end = find_bytes(self.src, b"-->", body).unwrap_or(self.src.len());
        let end = if body_end + 3 <= self.src.len() {
            body_end + 3
        } else {
            self.src.len()
        };
        self.events.push(ScanEvent::Comment {
            span:      sp(start, end),
            text_span: sp(body, body_end)
        });
        self.pos = end;
    }

    fn bogus(&mut self) {
        let start = self.pos;
        let end = find_byte(self.src, b'>', start + 2).map_or(self.src.len(), |i| i + 1);
        self.events.push(ScanEvent::Doctype {
            span: sp(start, end)
        });
        self.pos = end;
    }

    fn end_tag(&mut self) {
        let start = self.pos;
        let name_start = start + 2;
        let mut index = name_start;
        while let Some(byte) = self.at(index) {
            if is_name_end(byte) {
                break;
            }
            index += 1;
        }
        let name = self.text[name_start..index].to_ascii_lowercase();
        index = find_byte(self.src, b'>', index).map_or(self.src.len(), |i| i + 1);
        self.events.push(ScanEvent::End {
            name,
            span: sp(start, index)
        });
        self.pos = index;
    }

    fn start_tag(&mut self) {
        let start = self.pos;
        let name_start = start + 1;
        let mut index = name_start;
        while let Some(byte) = self.at(index) {
            if is_name_end(byte) {
                break;
            }
            index += 1;
        }
        let name = self.text[name_start..index].to_ascii_lowercase();
        let name_span = sp(name_start, index);
        let mut attrs = Vec::new();
        let mut self_closing = false;
        loop {
            index = self.skip_ws(index);
            match self.at(index) {
                None => break,
                Some(b'>') => {
                    index += 1;
                    break;
                }
                Some(b'/') => {
                    if self.at(index + 1) == Some(b'>') {
                        self_closing = true;
                        index += 2;
                        break;
                    }
                    index += 1;
                    continue;
                }
                _ => {}
            }
            let attr_start = index;
            let attr_name_start = index;
            while let Some(byte) = self.at(index) {
                if is_attr_end(byte) {
                    break;
                }
                index += 1;
            }
            let attr_name = self.text[attr_name_start..index].to_ascii_lowercase();
            index = self.skip_ws(index);
            let mut value = String::new();
            let mut value_span = None;
            if self.at(index) == Some(b'=') {
                index = self.skip_ws(index + 1);
                match self.at(index) {
                    Some(quote @ (b'"' | b'\'')) => {
                        let value_start = index + 1;
                        let value_end =
                            find_byte(self.src, quote, value_start).unwrap_or(self.src.len());
                        value = decode_entities(&self.text[value_start..value_end]);
                        value_span = Some(sp(value_start, value_end));
                        index = value_end + 1;
                    }
                    Some(b'>') | None => {}
                    _ => {
                        let value_start = index;
                        while let Some(byte) = self.at(index) {
                            if byte.is_ascii_whitespace() || byte == b'>' {
                                break;
                            }
                            index += 1;
                        }
                        value = decode_entities(&self.text[value_start..index]);
                        value_span = Some(sp(value_start, index));
                    }
                }
            }
            attrs.push(ScanAttr {
                name: attr_name,
                value,
                span: sp(attr_start, index),
                value_span
            });
        }
        let span = sp(start, index);
        let (raw_text, end_tag) = if self_closing {
            self.pos = index;
            (None, None)
        } else {
            self.raw_text_range(&name, index)
        };
        self.events.push(ScanEvent::Start(StartTag {
            name: name.clone(),
            attrs,
            span,
            name_span,
            self_closing,
            raw_text
        }));
        if let Some((close, end)) = end_tag {
            self.events.push(ScanEvent::End {
                name,
                span: sp(close, end)
            });
        }
    }

    /// Locates raw text (and its end tag) for raw-text elements, returning
    /// the content span and the end-tag range without publishing events.
    fn raw_text_range(
        &mut self,
        name: &str,
        content_start: usize
    ) -> (Option<Span>, Option<(usize, usize)>) {
        if !RAW_TEXT.contains(&name) {
            self.pos = content_start;
            return (None, None);
        }
        if name == "plaintext" {
            let span = sp(content_start, self.src.len());
            self.pos = self.src.len();
            return (Some(span), None);
        }
        let needle = format!("</{name}");
        let Some(close) = find_ci(self.src, needle.as_bytes(), content_start) else {
            let span = sp(content_start, self.src.len());
            self.pos = self.src.len();
            return (Some(span), None);
        };
        let content = sp(content_start, close);
        let end = find_byte(self.src, b'>', close).map_or(self.src.len(), |i| i + 1);
        self.pos = end;
        (Some(content), Some((close, end)))
    }
}

const fn is_name_end(byte: u8) -> bool {
    byte.is_ascii_whitespace() || byte == b'>' || byte == b'/'
}

const fn is_attr_end(byte: u8) -> bool {
    byte.is_ascii_whitespace() || byte == b'=' || byte == b'>' || byte == b'/'
}

fn find_byte(source: &[u8], needle: u8, from: usize) -> Option<usize> {
    source
        .get(from..)?
        .iter()
        .position(|&byte| byte == needle)
        .map(|offset| offset + from)
}

fn find_bytes(source: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    let window = source.get(from..)?;
    if window.len() < needle.len() {
        return None;
    }
    window
        .windows(needle.len())
        .position(|candidate| candidate == needle)
        .map(|offset| offset + from)
}

/// Case-insensitive search for an end-tag prefix followed by a delimiter.
fn find_ci(source: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    let first = *needle.first()?;
    let mut start = from;
    while let Some(offset) = find_byte(source, first, start) {
        let window = source.get(offset..offset + needle.len())?;
        let matched = window
            .iter()
            .zip(needle)
            .all(|(byte, want)| byte.to_ascii_lowercase() == *want);
        if matched
            && source
                .get(offset + needle.len())
                .is_some_and(|&next| next.is_ascii_whitespace() || next == b'>' || next == b'/')
        {
            return Some(offset);
        }
        start = offset + 1;
    }
    None
}

/// Decodes the basic character references that appear in widget attributes.
fn decode_entities(text: &str) -> String {
    if !text.contains('&') {
        return text.to_owned();
    }
    let mut out = String::with_capacity(text.len());
    let bytes = text.as_bytes();
    let mut index = 0_usize;
    while let Some(&byte) = bytes.get(index) {
        if byte != b'&' {
            let start = index;
            while let Some(&next) = bytes.get(index) {
                if next == b'&' {
                    break;
                }
                index += 1;
            }
            out.push_str(&text[start..index]);
            continue;
        }
        if let Some((decoded, consumed)) = parse_entity(&text[index..]) {
            out.push_str(&decoded);
            index += consumed;
        } else {
            out.push('&');
            index += 1;
        }
    }
    out
}

fn parse_entity(rest: &str) -> Option<(String, usize)> {
    let end = rest.get(1..=10)?.find(';')?;
    let body = rest.get(1..end)?;
    let code = if let Some(hex) = body.strip_prefix("#x").or_else(|| body.strip_prefix("#X")) {
        u32::from_str_radix(hex, 16).ok()?
    } else if let Some(dec) = body.strip_prefix('#') {
        dec.parse::<u32>().ok()?
    } else {
        let named = match body {
            "amp" => "&",
            "lt" => "<",
            "gt" => ">",
            "quot" => "\"",
            "apos" => "'",
            "nbsp" => "\u{a0}",
            _ => return None
        };
        return Some((named.to_owned(), end + 1));
    };
    Some((char::from_u32(code)?.to_string(), end + 1))
}
