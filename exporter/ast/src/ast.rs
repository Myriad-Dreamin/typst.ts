use std::fmt::Display;
use std::io::{self, Cursor, Write};
use std::sync::Mutex;

use typst::ide::Tag;
use typst::syntax::{LinkedNode, Source, SyntaxKind};
use typst_ts_core::{
    exporter_utils::{map_err, write_to_path},
    DocumentExporter,
};

pub struct AstPathExporter {
    path: Option<std::path::PathBuf>,
}

type VecCallback = Box<dyn FnMut(Vec<u8>) -> typst::diag::SourceResult<()>>;

pub struct AstVecExporter {
    pub vec_cb: Mutex<VecCallback>,
}

pub struct AstExporter {}

impl AstExporter {
    pub fn new_path(path: std::path::PathBuf) -> AstPathExporter {
        AstPathExporter { path: Some(path) }
    }

    pub fn new_vec(vec_cb: VecCallback) -> AstVecExporter {
        AstVecExporter {
            vec_cb: Mutex::new(vec_cb),
        }
    }
}

struct TranslationUnit<'a> {
    path: String,
    source: &'a Source,
}

struct AstWriter<'a, W: io::Write> {
    w: &'a mut W,
    ident: usize,
}

const COMMENT: ansi_term::Color = ansi_term::Color::RGB(0x4d, 0x52, 0x6b);
const STRING: ansi_term::Color = ansi_term::Color::RGB(0x9e, 0xce, 0x6a);
const NUMBER: ansi_term::Color = ansi_term::Color::RGB(0xe0, 0x8f, 0x68);
const KEYWORD: ansi_term::Color = ansi_term::Color::RGB(0xbb, 0x9a, 0xf7);
const OPERATOR: ansi_term::Color = ansi_term::Color::RGB(0xc0, 0xca, 0xf5);
const PUNC: ansi_term::Color = ansi_term::Color::RGB(0xc0, 0xca, 0xf5);
const VARIABLE: ansi_term::Color = ansi_term::Color::RGB(0x0f, 0x4b, 0x6e);
const FUNCTION: ansi_term::Color = ansi_term::Color::RGB(0x7a, 0xa2, 0xf7);
const MARKED: ansi_term::Color = ansi_term::Color::RGB(0x7d, 0xcf, 0xff);

impl<'a, W: io::Write> AstWriter<'a, W> {
    fn write_num_repr<T: Display>(&mut self, sk: SyntaxKind, ast: T) -> Option<()> {
        self.painted(NUMBER, format!("Num({:?}, {})", sk, ast));
        Some(())
    }

    fn painted(&mut self, c: ansi_term::Color, s: String) {
        self.w.write_fmt(format_args!("{}", c.paint(s))).unwrap();
    }

    fn write_repr(&mut self, ast: &LinkedNode) {
        let k = ast.kind();
        if let Some(hl) = typst::ide::highlight(ast) {
            match hl {
                Tag::Comment => {
                    self.painted(COMMENT, format!("Ct::{:?}", k));
                    return;
                }
                Tag::Escape => {
                    self.w.write_fmt(format_args!("Escape::{:?}", k)).unwrap();
                    return;
                }
                Tag::Keyword => {
                    self.painted(KEYWORD, format!("Kw::{:?}", k));
                    return;
                }
                Tag::Operator => {
                    self.painted(OPERATOR, format!("Op::{:?}", k));
                    return;
                }
                Tag::Punctuation => {
                    self.painted(PUNC, format!("Punc::{:?}", k));
                    return;
                }
                Tag::Function => {
                    self.painted(FUNCTION, format!("Fn::({:?})", ast));
                    return;
                }
                Tag::String => {
                    let wrapped: typst::syntax::ast::Str = ast.cast().unwrap();
                    self.painted(STRING, format!("Str(\"{}\")", wrapped.get()));
                    return;
                }
                Tag::Number => {
                    ast.cast::<typst::syntax::ast::Numeric>()
                        .map(|v| {
                            let with_unit = v.get();
                            self.write_num_repr(k, format!("{}{:?}", with_unit.0, with_unit.1))
                        })
                        .or_else(|| {
                            ast.cast::<typst::syntax::ast::Int>()
                                .map(|v| self.write_num_repr(k, v.get()))
                        })
                        .or_else(|| {
                            ast.cast::<typst::syntax::ast::Float>()
                                .map(|v| self.write_num_repr(k, v.get()))
                        });

                    return;
                }
                Tag::Interpolated => {
                    self.painted(VARIABLE, format!("Var::({:?})", ast));
                    return;
                }
                _ => {}
            }
        }

        if k == SyntaxKind::Ident {
            self.painted(MARKED, format!("Marked::({:?})", ast));
            return;
        }
        self.painted(MARKED, format!("Marked::{:?}", k));
    }

    fn write_ast(&mut self, src: &Source, ast: &LinkedNode) {
        let rng = src.range(ast.span());
        let start = rng.start;
        let end = rng.end;
        let start_end = [start, end]
            .iter()
            .map(|s| {
                (
                    src.byte_to_line(*s).map(|l| l + 1).unwrap_or(0),
                    src.byte_to_column(*s).unwrap_or(0),
                )
            })
            .collect::<Vec<_>>();
        self.w.write_all("s: ".as_bytes()).unwrap();
        self.write_repr(ast);
        let head = format!(
            " <{:?}:{:?}~{:?}:{:?}>",
            start_end[0].0, start_end[0].1, start_end[1].0, start_end[1].1
        );
        self.w.write_all(head.as_bytes()).unwrap();
        if ast.children().next().is_none() {
            return;
        }
        self.write_ast_children(src, ast);
    }

    fn write_ident(&mut self) {
        self.w.write_all(&[b'\n']).unwrap();
        for _i in 0..self.ident {
            self.w.write_all(&[b' ']).unwrap();
        }
    }

    fn write_ast_children(&mut self, src: &Source, ast: &LinkedNode) {
        self.write_ident();
        self.w.write_all("  c:".as_bytes()).unwrap();
        self.ident += 2;
        for ch in ast.children() {
            if ch.kind() == SyntaxKind::Space {
                continue;
            }
            self.write_ident();
            self.w.write_all("- ".as_bytes()).unwrap();
            self.write_ast(src, &ch);
        }
        self.ident -= 2;
    }

    fn write_ast_root(&mut self, src: &Source) {
        let ast = LinkedNode::new(src.root());
        self.w.write_all("s: ".as_bytes()).unwrap();
        self.write_repr(&ast);
        self.write_ast_children(src, &ast);
    }
}

impl DocumentExporter for AstPathExporter {
    fn export(
        &self,
        world: &dyn typst::World,
        _output: &typst::doc::Document,
    ) -> typst::diag::SourceResult<()> {
        let mut result = Vec::<TranslationUnit>::new();

        fn collect_translation_unit<'a>(result: &mut Vec<TranslationUnit<'a>>, src: &'a Source) {
            result.push(TranslationUnit {
                path: src.path().display().to_string(),
                source: src,
            });
        }
        collect_translation_unit(&mut result, world.main());

        let t = Vec::<u8>::new();
        let mut writer = Cursor::new(t);

        for tu in result {
            writer
                .write_all("---\n".as_bytes())
                .map_err(|e| map_err(world, e))?;
            writer
                .write_fmt(format_args!("path: {}\nast:\n  ", tu.path))
                .map_err(|e| map_err(world, e))?;
            let mut w = AstWriter {
                w: &mut writer,
                ident: 0,
            };
            w.write_ast_root(tu.source);
        }

        write_to_path(world, self.path.clone(), writer.get_ref())
    }
}

impl DocumentExporter for AstVecExporter {
    fn export(
        &self,
        world: &dyn typst::World,
        _output: &typst::doc::Document,
    ) -> typst::diag::SourceResult<()> {
        let mut result = Vec::<TranslationUnit>::new();

        fn collect_translation_unit<'a>(result: &mut Vec<TranslationUnit<'a>>, src: &'a Source) {
            result.push(TranslationUnit {
                path: src.path().display().to_string(),
                source: src,
            });
        }
        collect_translation_unit(&mut result, world.main());

        let t = Vec::<u8>::new();
        let mut writer = Cursor::new(t);

        for tu in result {
            writer
                .write_all("---\n".as_bytes())
                .map_err(|e| map_err(world, e))?;
            writer
                .write_fmt(format_args!("path: {}\nast:\n  ", tu.path))
                .map_err(|e| map_err(world, e))?;
            let mut w = AstWriter {
                w: &mut writer,
                ident: 0,
            };
            w.write_ast_root(tu.source);
        }

        writer.flush().unwrap();

        let mut vec_cb = self.vec_cb.lock().unwrap();
        (vec_cb.as_mut())(writer.into_inner())
    }
}
