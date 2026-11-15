// SPDX-FileCopyrightText: 2026 RAprogramm <andrey.rozanov.vl@gmail.com>
// SPDX-License-Identifier: MIT

//! Byte-level CSS scanner producing declarations, at-rules and selectors with
//! exact spans.
//!
//! The scanner is deliberately context-free: it splits the stylesheet into
//! declarations, at-rules and selectors without validating values, so every
//! finding can point at the exact bytes the author wrote. It understands
//! comments, strings and parentheses, which is all that is needed to split CSS
//! reliably at `:`, `;`, `{` and `}`.
//!
//! ```
//! let doc = wl_css::scan::scan(".a:hover { color: red }");
//! assert_eq!(doc.selectors.len(), 1);
//! assert_eq!(doc.decls.len(), 1);
//! ```

use wl_core::span::Span;

/// One `property: value` declaration.
pub struct Decl<'a> {
    /// Property name as written.
    pub prop:       &'a str,
    /// Span of the property name.
    pub prop_span:  Span,
    /// Value as written.
    pub value:      &'a str,
    /// Span of the value.
    pub value_span: Span
}

/// One `@name prelude` at-rule.
pub struct AtRule<'a> {
    /// At-rule name without the leading `@`.
    pub name:         &'a str,
    /// Span of the name (without `@`).
    pub name_span:    Span,
    /// Prelude between the name and the block or semicolon.
    pub prelude:      &'a str,
    /// Span of the prelude.
    pub prelude_span: Span
}

/// Selector of one style rule.
pub struct Selector<'a> {
    /// Selector text as written.
    pub text:   &'a str,
    /// Span of the selector.
    pub span:   Span,
    /// Whether the rule is nested inside another style rule.
    pub nested: bool
}

/// Everything the scanner found in one stylesheet.
pub struct CssDoc<'a> {
    /// Declarations in document order.
    pub decls:           Vec<Decl<'a>>,
    /// At-rules in document order.
    pub at_rules:        Vec<AtRule<'a>>,
    /// Style-rule selectors in document order.
    pub selectors:       Vec<Selector<'a>>,
    /// Total number of style rules and at-rules.
    pub rule_count:      u32,
    /// Number of `@keyframes` at-rules.
    pub keyframes_count: u32
}

/// A `name(args)` function call found inside a value.
pub struct Func<'a> {
    /// Function name.
    pub name:      &'a str,
    /// Span of the name.
    pub name_span: Span,
    /// Text between the parentheses.
    pub args:      &'a str,
    /// Span of the arguments.
    pub args_span: Span
}

/// Splits one CSS block (stylesheet, `<style>` body or `style=""` attribute)
/// into declarations, at-rules and selectors.
///
/// The scanner never fails: malformed input produces fewer items, never a
/// panic.
#[must_use]
pub fn scan(src: &str) -> CssDoc<'_> {
    Scanner::new(src).run()
}

const fn is_trivia(byte: u8) -> bool {
    matches!(byte, b' ' | b'\t' | b'\n' | b'\r' | b'\x0c')
}

/// Whether the byte may appear inside an identifier (property, function, word).
const fn is_ident_char(byte: u8) -> bool {
    byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'%')
}

const fn is_ident_start(byte: u8) -> bool {
    byte.is_ascii_alphabetic() || matches!(byte, b'-' | b'_')
}

/// Byte at index, `0` past the end so match arms stay total.
fn byte_at(bytes: &[u8], index: usize) -> u8 {
    bytes.get(index).copied().unwrap_or(0)
}

fn to_u32(value: usize) -> u32 {
    u32::try_from(value).unwrap_or(u32::MAX)
}

fn to_usize(value: u32) -> usize {
    usize::try_from(value).unwrap_or(usize::MAX)
}

fn span_of(start: usize, end: usize) -> Span {
    Span::new(to_u32(start), to_u32(end))
}

/// Trims surrounding whitespace and shrinks the span accordingly.
fn trimmed(text: &str, span: Span) -> (&str, Span) {
    let trimmed = text.trim();
    let lead = text.len() - text.trim_start().len();
    let trail = text.len() - text.trim_end().len();
    (
        trimmed,
        Span::new(
            span.start.saturating_add(to_u32(lead)),
            span.end.saturating_sub(to_u32(trail))
        )
    )
}

/// Skips a string literal starting at `pos`, returns the index after it.
fn skip_string(bytes: &[u8], pos: usize) -> usize {
    let quote = byte_at(bytes, pos);
    let mut i = pos + 1;
    while i < bytes.len() {
        let byte = byte_at(bytes, i);
        if byte == b'\\' {
            i += 2;
        } else if byte == quote {
            return i + 1;
        } else {
            i += 1;
        }
    }
    bytes.len()
}

/// Iterates `name(args)` function calls in a value, innermost included.
pub(crate) fn functions(value: &str) -> Vec<Func<'_>> {
    let bytes = value.as_bytes();
    let mut out = Vec::new();
    let mut i = 0_usize;
    while i < bytes.len() {
        let byte = byte_at(bytes, i);
        if byte == b'"' || byte == b'\'' {
            i = skip_string(bytes, i);
            continue;
        }
        if is_ident_start(byte) {
            let start = i;
            while i < bytes.len() && is_ident_char(byte_at(bytes, i)) {
                i += 1;
            }
            if byte_at(bytes, i) == b'(' {
                let name = &value[start..i];
                let open = i;
                let mut depth = 0_usize;
                let mut j = open;
                while j < bytes.len() {
                    match byte_at(bytes, j) {
                        b'(' => depth += 1,
                        b')' => {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        b'"' | b'\'' => {
                            j = skip_string(bytes, j);
                            continue;
                        }
                        _ => {}
                    }
                    j += 1;
                }
                let close = j.min(bytes.len());
                out.push(Func {
                    name,
                    name_span: span_of(start, i),
                    args: &value[open + 1..close],
                    args_span: span_of(open + 1, close)
                });
            }
            continue;
        }
        i += 1;
    }
    out
}

/// Iterates number-plus-unit tokens such as `50vh` in a value.
pub(crate) fn lengths(value: &str) -> Vec<(&str, Span)> {
    let bytes = value.as_bytes();
    let mut out = Vec::new();
    let mut i = 0_usize;
    while i < bytes.len() {
        let byte = byte_at(bytes, i);
        let digit =
            byte.is_ascii_digit() || (byte == b'.' && byte_at(bytes, i + 1).is_ascii_digit());
        if !digit {
            i += 1;
            continue;
        }
        let start = i;
        while i < bytes.len() && matches!(byte_at(bytes, i), b'0'..=b'9' | b'.') {
            i += 1;
        }
        let unit_start = i;
        while i < bytes.len() && byte_at(bytes, i).is_ascii_alphabetic() {
            i += 1;
        }
        if i > unit_start {
            out.push((&value[start..i], span_of(start, i)));
        }
    }
    out
}

/// Iterates identifier words (letters, digits, `-`, `_`, `%`) in a text.
pub(crate) fn words(text: &str) -> Vec<(&str, Span)> {
    let bytes = text.as_bytes();
    let mut out = Vec::new();
    let mut i = 0_usize;
    while i < bytes.len() {
        if is_ident_char(byte_at(bytes, i)) {
            let start = i;
            while i < bytes.len() && is_ident_char(byte_at(bytes, i)) {
                i += 1;
            }
            out.push((&text[start..i], span_of(start, i)));
        } else {
            i += 1;
        }
    }
    out
}

struct Scanner<'a> {
    src:   &'a str,
    bytes: &'a [u8],
    pos:   usize
}

impl<'a> Scanner<'a> {
    const fn new(src: &'a str) -> Self {
        Self {
            src,
            bytes: src.as_bytes(),
            pos: 0
        }
    }

    fn run(mut self) -> CssDoc<'a> {
        let mut doc = CssDoc {
            decls:           Vec::new(),
            at_rules:        Vec::new(),
            selectors:       Vec::new(),
            rule_count:      0,
            keyframes_count: 0
        };
        let mut blocks: Vec<bool> = Vec::new();
        loop {
            self.skip_trivia();
            if self.pos >= self.bytes.len() {
                break;
            }
            match byte_at(self.bytes, self.pos) {
                b'}' => {
                    blocks.pop();
                    self.pos += 1;
                }
                b';' => self.pos += 1,
                b'@' => self.at_rule(&mut doc, &mut blocks),
                _ => self.item(&mut doc, &mut blocks)
            }
        }
        doc
    }

    fn skip_trivia(&mut self) {
        loop {
            while self.pos < self.bytes.len() && is_trivia(byte_at(self.bytes, self.pos)) {
                self.pos += 1;
            }
            if self.starts_comment() {
                self.skip_comment();
            } else {
                break;
            }
        }
    }

    fn starts_comment(&self) -> bool {
        self.bytes
            .get(self.pos..)
            .is_some_and(|rest| rest.starts_with(b"/*"))
    }

    fn skip_comment(&mut self) {
        let end = self
            .bytes
            .get(self.pos + 2..)
            .and_then(|rest| rest.windows(2).position(|w| w == b"*/"))
            .map_or(self.bytes.len(), |offset| self.pos + offset + 4);
        self.pos = end;
    }

    /// Reads text up to one of `stops` at parenthesis depth zero. The stop byte
    /// is not consumed. Comments and strings are skipped but kept in the text.
    fn read_until(&mut self, stops: &[u8]) -> (&'a str, Span, Option<u8>) {
        let start = self.pos;
        let mut parens = 0_usize;
        while self.pos < self.bytes.len() {
            let byte = byte_at(self.bytes, self.pos);
            match byte {
                b'/' if self.starts_comment() => self.skip_comment(),
                b'"' | b'\'' => self.pos = skip_string(self.bytes, self.pos),
                b'(' => {
                    parens += 1;
                    self.pos += 1;
                }
                b')' => {
                    parens = parens.saturating_sub(1);
                    self.pos += 1;
                }
                _ if parens == 0 && stops.contains(&byte) => break,
                _ => self.pos += 1
            }
        }
        let stop = self.bytes.get(self.pos).copied();
        (&self.src[start..self.pos], span_of(start, self.pos), stop)
    }

    fn at_rule(&mut self, doc: &mut CssDoc<'a>, blocks: &mut Vec<bool>) {
        let name_start = self.pos + 1;
        self.pos += 1;
        while self.pos < self.bytes.len() && is_ident_char(byte_at(self.bytes, self.pos)) {
            self.pos += 1;
        }
        let name = &self.src[name_start..self.pos];
        let name_span = span_of(name_start, self.pos);
        let (prelude, prelude_span, stop) = self.read_until(b"{;");
        let (prelude, prelude_span) = trimmed(prelude, prelude_span);
        doc.rule_count += 1;
        if name.eq_ignore_ascii_case("keyframes") || name.ends_with("-keyframes") {
            doc.keyframes_count += 1;
        }
        doc.at_rules.push(AtRule {
            name,
            name_span,
            prelude,
            prelude_span
        });
        match stop {
            Some(b'{') => {
                self.pos += 1;
                blocks.push(true);
            }
            Some(b';') => self.pos += 1,
            _ => self.pos = self.bytes.len()
        }
    }

    fn item(&mut self, doc: &mut CssDoc<'a>, blocks: &mut Vec<bool>) {
        let (raw, raw_span, stop) = self.read_until(b":{};");
        match stop {
            Some(b':') => {
                self.pos += 1;
                let (ahead, ahead_span, ahead_stop) = self.read_until(b"{;}");
                if ahead_stop == Some(b'{') {
                    let end = to_usize(ahead_span.end);
                    self.pos = end + 1;
                    let text = &self.src[to_usize(raw_span.start)..end];
                    Self::push_selector(
                        doc,
                        blocks,
                        text,
                        Span::new(raw_span.start, ahead_span.end)
                    );
                } else {
                    if ahead_stop == Some(b';') {
                        self.pos += 1;
                    }
                    let (prop, prop_span) = trimmed(raw, raw_span);
                    let (value, value_span) = trimmed(ahead, ahead_span);
                    if !prop.is_empty() {
                        doc.decls.push(Decl {
                            prop,
                            prop_span,
                            value,
                            value_span
                        });
                    }
                }
            }
            Some(b'{') => {
                self.pos += 1;
                Self::push_selector(doc, blocks, raw, raw_span);
            }
            Some(b';') => self.pos += 1,
            _ => self.pos = self.bytes.len()
        }
    }

    fn push_selector(doc: &mut CssDoc<'a>, blocks: &mut Vec<bool>, text: &'a str, span: Span) {
        let (text, span) = trimmed(text, span);
        let nested = blocks.last() == Some(&false);
        if !text.is_empty() {
            doc.selectors.push(Selector {
                text,
                span,
                nested
            });
        }
        doc.rule_count += 1;
        blocks.push(false);
    }
}
