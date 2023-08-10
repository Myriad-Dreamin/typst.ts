use std::borrow::Cow::{self, Owned};
use std::cell::RefCell;
use typst_ts_compiler::service::CompileDriver;
use typst_ts_core::exporter_builtins::GroupExporter;
use typst_ts_core::exporter_utils::map_err;

use rustyline::completion::{Completer, Pair};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, CompletionType, Config, EditMode, Editor, KeyEvent};
use rustyline::{Helper, Validator};

use crate::query::serialize;
use crate::CompileOnceArgs;

use typst::ide::autocomplete;
use typst::{doc::Document, World};

#[derive(Helper, Validator)]
struct ReplContext {
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,

    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,

    // typst world state
    driver: RefCell<CompileDriver>,
}

impl ReplContext {
    fn new(driver: CompileDriver) -> Self {
        ReplContext {
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            validator: MatchingBracketValidator::new(),
            driver: RefCell::new(driver),
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

    fn highlight_char(&self, line: &str, pos: usize) -> bool {
        self.highlighter.highlight_char(line, pos)
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
        let map_io_err = |e| ReadlineError::Io(e);

        // commit line changes
        let main_id = driver.main_id();

        let content = std::fs::read_to_string(&driver.entry_file).map_err(map_io_err)?;
        println!("content: {} \"{}\"", content.len(), content);

        let static_prefix = content + "\r\n" + "#show ";
        let static_prefix_len = static_prefix.len();
        let dyn_content = static_prefix + line;

        let file = driver.entry_file.clone();

        driver
            .with_shadow_file(&file, |driver| {
                driver.world.reset();

                match driver
                    .world
                    .resolve_with(&driver.entry_file, main_id, &dyn_content)
                {
                    Ok(()) => {
                        driver.world.main = main_id;
                    }
                    Err(e) => return Err(map_err(e)),
                }

                let source = driver.world.main();
                let cursor = static_prefix_len + pos;

                let res = autocomplete(&driver.world, &[], &source, cursor, true);
                match res {
                    Some((start, items)) => {
                        println!("start: {} cursor: {} slen: {}", start, cursor, static_prefix_len);
                        let start = start.saturating_sub(static_prefix_len);
                        Ok((
                            start,
                            items
                                .into_iter()
                                .map(|item| {
                                    let rep =
                                        item.apply.as_ref().unwrap_or(&item.label).to_string();
                                    Pair {
                                        display: item.label.to_string(),
                                        replacement: rep,
                                    }
                                })
                                .collect(),
                        ))
                    }
                    None => {
                        Ok((0, vec![]))
                    }
                }
            })
            .map_err(|err| {
                println!("error: {:?}", err);
                err
            })
            .map_or(Ok((0, vec![])), |res| Ok(res))
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

    let exporter = GroupExporter::<Document>::new(vec![]);
    let driver = crate::compile::create_driver(args.clone(), exporter);

    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(ReplContext::new(driver)));
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
        let compiled =
            self.driver
                .borrow_mut()
                .with_compile_diag::<false, _>(|driver: &mut CompileDriver| {
                    let doc = driver.compile()?;
                    driver.query(line, &doc).map_err(map_err)
                });

        if let Some(compiled) = compiled {
            let serialized = serialize(&compiled, "json").unwrap();
            println!("{serialized}");
        }
    }
}
