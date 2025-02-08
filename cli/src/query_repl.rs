use std::borrow::Cow::{self, Owned};
use std::cell::RefCell;

use reflexo_typst::typst::prelude::*;
use reflexo_typst::{
    CompilerExt, DiagnosticHandler, TypstDocument, TypstPagedDocument, TypstSystemUniverse,
};
use reflexo_typst::{ShadowApiExt, TypstSystemWorld};
use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, CompletionType, Config, EditMode, Editor, KeyEvent};
use rustyline::{Helper, Validator};
use typst::foundations::Bytes;
use typst::World;
use typst_ide::{autocomplete, IdeWorld};

use crate::query::serialize;
use crate::CompileOnceArgs;

#[derive(Helper, Validator)]
struct ReplContext {
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,

    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,

    // typst world state
    driver: RefCell<TypstSystemUniverse>,
    diag_handler: DiagnosticHandler,
}

impl ReplContext {
    fn new(driver: TypstSystemUniverse, diag_handler: DiagnosticHandler) -> Self {
        ReplContext {
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            validator: MatchingBracketValidator::new(),
            driver: RefCell::new(driver),
            diag_handler,
        }
    }
}

// Mix history hinter with fallback completer
impl Hinter for ReplContext {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<String> {
        if let Some(hint) = self.hinter.hint(line, pos, ctx) {
            Some(hint)
        } else {
            self.complete(line, pos, ctx).map_or_else(
                |_| None,
                |v| {
                    v.1.first()
                        .map(|pr| (pr.replacement[(line.len() - v.0)..]).to_string())
                },
            )
        }
    }
}

impl Highlighter for ReplContext {
    fn highlight_prompt<'b, 's: 'b, 'p: 'b>(
        &'s self,
        prompt: &'p str,
        default: bool,
    ) -> Cow<'b, str> {
        // if default {
        //     Borrowed(&self.colored_prompt)
        // } else {
        // }
        let _ = default;
        Owned(format!("\x1b[1;32m{prompt}\x1b[0m"))
    }

    fn highlight_hint<'h>(&self, hint: &'h str) -> Cow<'h, str> {
        Owned("\x1b[90m".to_owned() + hint + "\x1b[m")
    }

    fn highlight<'l>(&self, line: &'l str, pos: usize) -> Cow<'l, str> {
        self.highlighter.highlight(line, pos)
    }

    fn highlight_char(&self, line: &str, pos: usize, forced: bool) -> bool {
        self.highlighter.highlight_char(line, pos, forced)
    }
}

fn to_repl_completion_pair(item: typst_ide::Completion) -> Pair {
    // we does not support code snippet
    // let rep = item.apply.as_ref().unwrap_or(&item.label).into();
    let rep = item.label.clone().into();
    Pair {
        display: item.label.into(),
        replacement: rep,
    }
}

impl Completer for ReplContext {
    type Candidate = Pair;

    fn complete(
        &self,
        line: &str,
        pos: usize,
        _ctx: &rustyline::Context<'_>,
    ) -> rustyline::Result<(usize, Vec<Pair>)> {
        let mut driver = self.driver.borrow_mut();

        // commit line changes

        let entry = driver.entry_file().unwrap().to_err().unwrap();
        let content = std::fs::read_to_string(&entry).map_err(ReadlineError::Io)?;
        let static_prefix = content + "\n#show ";
        let static_prefix_len = static_prefix.len();
        let cursor = static_prefix_len + pos;
        let dyn_content = static_prefix + line;

        #[cfg(feature = "debug-repl")]
        println!("slen: {}, dlen: {}", static_prefix_len, dyn_content.len());

        driver.reset();

        let typst_completions = driver
            .with_shadow_file(
                &entry,
                Bytes::new(dyn_content.as_bytes().to_vec()),
                |driver| {
                    let graph = driver.computation();
                    let doc = graph.pure_compile::<TypstPagedDocument>();
                    let world = &graph.snap.world;
                    let doc = self.diag_handler.report_compiled(world, doc);
                    let main = world.source(world.main()).unwrap();

                    Ok(autocomplete(
                        &IdeWrapper(world),
                        doc.as_deref(),
                        &main,
                        cursor,
                        true,
                    ))
                },
            )
            .ok()
            .flatten();

        let (start, items) = if let Some(res) = typst_completions {
            res
        } else {
            #[cfg(feature = "debug-repl")]
            println!("no results");
            return Ok((0, vec![]));
        };

        #[cfg(feature = "debug-repl")]
        println!(
            "start: {}, pref_len: {}, items: {:?}",
            start,
            static_prefix_len,
            items.iter().map(|t| t.label.clone()).collect::<Vec<_>>()
        );

        if start < static_prefix_len {
            return Ok((0, vec![]));
        }

        let completing_prefix_len = start - static_prefix_len;
        let completing_suffix = &line[completing_prefix_len..];
        let items = items.into_iter().map(to_repl_completion_pair);
        let items = items.filter(|item| {
            if completing_suffix.is_empty() {
                return true;
            }

            // lcs based filtering
            let mut completing_suffix_chars = completing_suffix.chars();
            let mut cur = completing_suffix_chars.next().unwrap();
            for ch in item.replacement.chars() {
                if cur == ch {
                    if let Some(nxt) = completing_suffix_chars.next() {
                        cur = nxt;
                    } else {
                        return true;
                    }
                } else {
                    continue;
                }
            }

            false
        });

        Ok((completing_prefix_len, items.collect()))
    }
}

struct IdeWrapper<'a>(&'a TypstSystemWorld);

impl std::ops::Deref for IdeWrapper<'_> {
    type Target = TypstSystemWorld;

    fn deref(&self) -> &Self::Target {
        self.0
    }
}

impl World for IdeWrapper<'_> {
    fn library(&self) -> &reflexo_typst::LazyHash<typst::Library> {
        self.0.library()
    }

    fn book(&self) -> &reflexo_typst::LazyHash<typst::text::FontBook> {
        self.0.book()
    }

    fn main(&self) -> reflexo_typst::TypstFileId {
        self.0.main()
    }

    fn source(
        &self,
        id: reflexo_typst::TypstFileId,
    ) -> typst::diag::FileResult<typst::syntax::Source> {
        self.0.source(id)
    }

    fn file(
        &self,
        id: reflexo_typst::TypstFileId,
    ) -> typst::diag::FileResult<typst::foundations::Bytes> {
        self.0.file(id)
    }

    fn font(&self, index: usize) -> Option<reflexo_typst::TypstFont> {
        self.0.font(index)
    }

    fn today(&self, offset: Option<i64>) -> Option<reflexo_typst::TypstDatetime> {
        self.0.today(offset)
    }
}

impl IdeWorld for IdeWrapper<'_> {
    fn upcast(&self) -> &dyn World {
        self.0
    }

    fn packages(&self) -> &[(reflexo_typst::package::PackageSpec, Option<EcoString>)] {
        self.0.packages()
    }
}

// To debug rustyline:
// RUST_LOG=rustyline=debug cargo run --example example 2> debug.log
pub fn start_repl_test(args: CompileOnceArgs) -> rustyline::Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();

    let driver = crate::compile::resolve_universe(args.clone());

    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(ReplContext::new(driver, DiagnosticHandler::default())));
    rl.bind_sequence(KeyEvent::alt('n'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    loop {
        let readline = rl.readline("typst.ts> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                rl.helper_mut().unwrap().repl_process_line(line);
            }
            Err(ReadlineError::Interrupted) => {
                println!("Interrupted");
                break;
            }
            Err(ReadlineError::Eof) => {
                println!("Encountered Eof");
                break;
            }
            Err(err) => {
                println!("Error: {err:?}");
                break;
            }
        }
    }

    rl.append_history("history.txt")
}

impl ReplContext {
    fn repl_process_line(&mut self, line: String) {
        let compiled = {
            let graph = self.driver.borrow().computation();
            let doc = graph.pure_compile::<TypstPagedDocument>();
            let doc = self.diag_handler.report_compiled(graph.world(), doc);

            doc.and_then(|doc| {
                graph
                    .query(line, &TypstDocument::Paged(doc))
                    .map_err(|err| self.diag_handler.report(&graph.snap.world, err.iter()))
                    .ok()
            })
        };

        if let Some(compiled) = compiled {
            let serialized = serialize(&compiled, "json").unwrap();
            println!("{serialized}");
        }
    }
}
