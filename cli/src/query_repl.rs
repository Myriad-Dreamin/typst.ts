use std::borrow::Cow::{self, Owned};
use typst_ts_compiler::service::CompileDriver;
use typst_ts_core::exporter_builtins::GroupExporter;
use typst_ts_core::exporter_utils::map_err;

use rustyline::completion::{Completer, FilenameCompleter};
use rustyline::error::ReadlineError;
use rustyline::highlight::{Highlighter, MatchingBracketHighlighter};
use rustyline::hint::{Hinter, HistoryHinter};
use rustyline::history::FileHistory;
use rustyline::validate::MatchingBracketValidator;
use rustyline::{Cmd, CompletionType, Config, EditMode, Editor, KeyEvent};
use rustyline::{Completer, Helper, Validator};

use crate::query::serialize;
use crate::CompileOnceArgs;

use typst::doc::Document;
// use typst::ide::autocomplete;

#[derive(Helper, Completer, Validator)]
struct ReplHelper {
    #[rustyline(Completer)]
    completer: FilenameCompleter,
    #[rustyline(Validator)]
    validator: MatchingBracketValidator,

    highlighter: MatchingBracketHighlighter,
    hinter: HistoryHinter,
}

impl ReplHelper {
    fn new() -> Self {
        ReplHelper {
            completer: FilenameCompleter::new(),
            highlighter: MatchingBracketHighlighter::new(),
            hinter: HistoryHinter {},
            validator: MatchingBracketValidator::new(),
        }
    }
}

// Mix history hinter with fallback completer
impl Hinter for ReplHelper {
    type Hint = String;

    fn hint(&self, line: &str, pos: usize, ctx: &rustyline::Context<'_>) -> Option<String> {
        if let Some(hint) = self.hinter.hint(line, pos, ctx) {
            Some(hint)
        } else {
            self.completer.complete(line, pos, ctx).map_or_else(
                |_| None,
                |v| {
                    v.1.first()
                        .map(|pr| (pr.replacement[(line.len() - v.0)..]).to_string())
                },
            )
        }
    }
}

impl Highlighter for ReplHelper {
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

// To debug rustyline:
// RUST_LOG=rustyline=debug cargo run --example example 2> debug.log
pub fn start_repl_test(args: CompileOnceArgs) -> rustyline::Result<()> {
    let config = Config::builder()
        .history_ignore_space(true)
        .completion_type(CompletionType::List)
        .edit_mode(EditMode::Emacs)
        .build();

    let mut rl = Editor::with_config(config)?;
    rl.set_helper(Some(ReplHelper::new()));
    rl.bind_sequence(KeyEvent::alt('n'), Cmd::HistorySearchForward);
    rl.bind_sequence(KeyEvent::alt('p'), Cmd::HistorySearchBackward);
    if rl.load_history("history.txt").is_err() {
        println!("No previous history.");
    }
    repl_worker(args, &mut rl)?;
    rl.append_history("history.txt")
}

fn repl_worker(
    args: CompileOnceArgs,
    rl: &mut Editor<ReplHelper, FileHistory>,
) -> rustyline::Result<()> {
    loop {
        let readline = rl.readline("typst.ts> ");
        match readline {
            Ok(line) => {
                rl.add_history_entry(line.as_str())?;
                repl_process_line(args.clone(), line);
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

    Ok(())
}

fn repl_process_line(args: CompileOnceArgs, line: String) {
    let exporter = GroupExporter::<Document>::new(vec![]);

    // TODO: avoid immediate exiting
    let mut driver = crate::compile::create_driver(args.clone(), exporter);
    let compiled = driver.with_compile_diag::<false, _>(|driver: &mut CompileDriver| {
        let doc = driver.compile()?;
        driver.query(line, &doc).map_err(map_err)
    });

    if let Some(compiled) = compiled {
        let serialized = serialize(&compiled, "json").unwrap();
        println!("{serialized}");
    }
}
