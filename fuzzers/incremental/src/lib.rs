//! based on project <https://github.com/frozolotl/typst-mutilate>
//! LICENSE: European Union Public License 1.2
use rand::{seq::IteratorRandom, Rng, SeedableRng};
use rand_xoshiro::Xoshiro256PlusPlus;
use std::{
    borrow::Cow,
    io::{self, Write},
    ops::Range,
};

use typst_syntax::{ast, SyntaxKind, SyntaxNode};

pub fn mutate(code: String) -> io::Result<String> {
    let mut tr = Mutator::build_context()?;

    let syntax = typst_syntax::parse(&code);
    let errors = syntax.errors();
    if !errors.is_empty() {
        panic!("Syntax errors: {errors:?}");
    }

    tr.mutate_syntax(&syntax)?;
    Ok(String::from_utf8(tr.output).unwrap())
}

// unicode range
const JPWORD_RANGE: Range<u32> = 0x3040..0x30ff;
const HANZI_RANGE: Range<u32> = 0x4e00..0x9fa5;
const ENGLISH_RANGE: Range<u32> = 0x61..0x7b;

struct Mutator {
    rng: Xoshiro256PlusPlus,
    aggressive: bool,
    // language: Lang,
    output: Vec<u8>,
}

impl Mutator {
    fn build_context() -> io::Result<Self> {
        let rng = Xoshiro256PlusPlus::from_rng(rand::thread_rng()).unwrap();
        Ok(Self {
            rng,
            aggressive: false,
            output: Vec::new(),
            // language,
        })
    }

    fn mutate_syntax(&mut self, syntax: &SyntaxNode) -> io::Result<()> {
        match syntax.kind() {
            SyntaxKind::Text => self.mutate_text(syntax.text()),
            SyntaxKind::LineComment => {
                write!(self.output, "//")?;
                let content = &syntax.text()[2..];
                write!(self.output, "{content}")?;
                // self.translate_text(content)?;
                Ok(())
            }
            SyntaxKind::BlockComment => {
                write!(self.output, "/*")?;
                let content = &syntax.text()[2..syntax.text().len() - 2];
                write!(self.output, "{content}")?;
                // self.translate_text(content)?;
                write!(self.output, "*/")?;
                Ok(())
            }
            SyntaxKind::Str if self.aggressive => {
                write!(self.output, "\"")?;
                let content = &syntax.text()[1..syntax.text().len() - 1];
                self.mutate_text(content)?;
                write!(self.output, "\"")?;
                Ok(())
            }
            SyntaxKind::Raw => {
                let raw: ast::Raw = syntax.cast().unwrap();
                let backticks = syntax.text().split(|c| c != '`').next().unwrap();
                write!(self.output, "{backticks}")?;

                let mut text = syntax
                    .text()
                    .trim_start_matches('`')
                    .strip_suffix(backticks)
                    .unwrap();
                if let Some(lang) = raw.lang() {
                    text = text.strip_prefix(lang.get().as_str()).unwrap();
                    write!(self.output, "{lang}", lang = lang.get())?;
                }

                // will not translate text inside raw blocks
                write!(self.output, "{text}")?;
                // self.translate_text(text, self.output)?;
                write!(self.output, "{backticks}")?;
                Ok(())
            }
            SyntaxKind::Link => {
                let (scheme, rest) = syntax.text().split_once(':').unwrap();
                write!(self.output, "{scheme}:")?;
                self.mutate_text(rest)
            }
            SyntaxKind::Equation
            | SyntaxKind::Math
            | SyntaxKind::MathIdent
            | SyntaxKind::MathAlignPoint
            | SyntaxKind::MathDelimited
            | SyntaxKind::MathFrac
            | SyntaxKind::MathPrimes
            | SyntaxKind::MathRoot
            | SyntaxKind::ModuleInclude
            | SyntaxKind::ModuleImport => self.write_node(syntax),
            _ if syntax.children().next().is_some() => {
                for child in syntax.children() {
                    self.mutate_syntax(child)?;
                }
                Ok(())
            }
            _ => self.write_node(syntax),
        }
    }

    fn mutate_text(&mut self, text: &str) -> io::Result<()> {
        if self.rng.gen_bool(0.96) {
            write!(self.output, "{text}")?;
            return Ok(());
        }

        let mutated = if self.rng.gen_bool(0.33) {
            Cow::Owned(text.repeat(50.min(1000 / text.len()).max(1)))
        } else if self.rng.gen_bool(0.33) {
            Cow::Owned(text.chars().take(50).collect::<String>())
        } else {
            Cow::Borrowed(text)
        }
        .chars()
        .flat_map(|c| {
            if c.is_ascii_punctuation() || c.is_whitespace() {
                return Some(c);
            }

            let res = (0..3).choose(&mut self.rng).unwrap();
            let w = match res {
                0 => ENGLISH_RANGE.choose(&mut self.rng).unwrap(),
                1 => HANZI_RANGE.choose(&mut self.rng).unwrap(),
                2 => JPWORD_RANGE.choose(&mut self.rng).unwrap(),
                _ => unreachable!(),
            };
            char::from_u32(w)
        })
        .collect::<String>();
        write!(self.output, "{mutated}")?;
        Ok(())
    }

    fn write_node(&mut self, syntax: &SyntaxNode) -> io::Result<()> {
        if syntax.children().next().is_some() {
            for child in syntax.children() {
                self.write_node(child)?;
            }
        } else {
            write!(self.output, "{}", syntax.text())?;
        }
        Ok(())
    }
}
