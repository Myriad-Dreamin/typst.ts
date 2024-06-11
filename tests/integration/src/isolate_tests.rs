use std::{collections::HashMap, path::Path};

use anyhow::Context;
use ecow::{eco_format, EcoString};
use reflexo::{
    hash::{Fingerprint, FingerprintBuilder},
    vector::ir::{Axes, GroupRef, Page, PathItem, PathStyle, Point, Scalar, VecItem},
};
use typst::{
    diag::StrResult,
    foundations::Value,
    syntax::{
        ast::{self, AstNode},
        LinkedNode, Source,
    },
};
use typst_ts_svg_exporter::{Module, VecDocument};

const ZERO: Fingerprint = Fingerprint::from_u128(0);

type NodeDict = HashMap<EcoString, Node>;

#[derive(Debug, Clone)]
enum Node {
    Content(Fingerprint),
    Dict(NodeDict),
    Value(Value),
}

impl Node {
    fn content(&self) -> Fingerprint {
        match self {
            Node::Content(f) => *f,
            _ => panic!(),
        }
    }
}

#[derive(Debug)]
struct Arguments {
    positional: Vec<Node>,
    named: HashMap<EcoString, Node>,
}

#[derive(Default)]
struct Evaluator {
    items: HashMap<Fingerprint, VecItem>,
    defs: HashMap<EcoString, Node>,

    at: Point,
    page: Option<Page>,

    fingerprint_builder: FingerprintBuilder,
}

impl Evaluator {
    fn store(&mut self, item: VecItem) -> Fingerprint {
        let fingerprint = self.fingerprint_builder.resolve(&item);
        self.items.insert(fingerprint, item);
        fingerprint
    }

    fn eval(&mut self, node: LinkedNode) -> Option<Page> {
        self.page = None;

        self.node(node);

        self.page.take()
    }

    fn node(&mut self, node: LinkedNode) -> Option<Node> {
        use typst::syntax::SyntaxKind::*;
        match node.kind() {
            Code | Markup | ContentBlock | CodeBlock => return self.block(node),
            Text => {}
            Space => {}
            Linebreak => {}
            Parbreak => {}
            Escape => {}
            Shorthand => {}
            SmartQuote => {}
            Strong => {}
            Emph => {}
            Raw => {}
            RawLang => {}
            RawDelim => {}
            RawTrimmed => {}
            Link => {}
            Label => {}
            Ref => {}
            RefMarker => {}
            Heading => {}
            HeadingMarker => {}
            ListItem => {}
            ListMarker => {}
            EnumItem => {}
            EnumMarker => {}
            TermItem => {}
            TermMarker => {}
            Equation => {}
            Math => {}
            MathIdent => {}
            MathAlignPoint => {}
            MathDelimited => {}
            MathAttach => {}
            MathPrimes => {}
            MathFrac => {}
            MathRoot => {}
            Hash => {}
            LeftBrace => {}
            RightBrace => {}
            LeftBracket => {}
            RightBracket => {}
            LeftParen => {}
            RightParen => {}
            Comma => {}
            Semicolon => {}
            Colon => {}
            Star => {}
            Underscore => {}
            Dollar => {}
            Plus => {}
            Minus => {}
            Slash => {}
            Hat => {}
            Prime => {}
            Dot => {}
            Eq => {}
            EqEq => {}
            ExclEq => {}
            Lt => {}
            LtEq => {}
            Gt => {}
            GtEq => {}
            PlusEq => {}
            HyphEq => {}
            StarEq => {}
            SlashEq => {}
            Dots => {}
            Arrow => {}
            Root => {}
            Not => {}
            And => {}
            Or => {}
            None => {}
            Auto => {}
            Let => {}
            Set => {}
            Show => {}
            Context => {}
            If => {}
            Else => {}
            For => {}
            In => {}
            While => {}
            Break => {}
            Continue => {}
            Return => {}
            Import => {}
            Include => {}
            As => {}
            Ident => {
                let i: ast::Ident = node.cast()?;
                let name = i.get().clone();
                return self.defs.get(&name).cloned();
            }
            Bool => {
                let b: ast::Bool = node.cast()?;
                return Some(Node::Value(Value::Bool(b.get())));
            }
            Int => {
                let i: ast::Int = node.cast()?;
                return Some(Node::Value(Value::Int(i.get())));
            }
            Float => {
                let f: ast::Float = node.cast()?;
                return Some(Node::Value(Value::Float(f.get())));
            }
            Numeric => {
                let n: ast::Numeric = node.cast()?;
                // todo: unit
                return Some(Node::Value(Value::Float(n.get().0)));
            }
            Str => {
                let s: ast::Str = node.cast()?;
                return Some(Node::Value(Value::Str(s.get().into())));
            }
            Parenthesized => {
                let p: ast::Parenthesized = node.cast()?;
                return self.node(node.find(p.expr().span())?);
            }
            Array => {}
            Dict => {
                let d: ast::Dict = node.cast()?;
                let mut dict = NodeDict::new();
                for item in d.items() {
                    let (key, value) = match item {
                        ast::DictItem::Named(named) => {
                            let key = named.name().get().clone();
                            let value = self.node(node.find(named.expr().span())?)?;
                            (key, value)
                        }
                        _ => unimplemented!(),
                    };
                    dict.insert(key, value);
                }
                return Some(Node::Dict(dict));
            }
            FuncCall => {
                log::debug!("FuncCall {:?}", node);
                let fc: ast::FuncCall = node.cast()?;
                let callee = match fc.callee() {
                    ast::Expr::Ident(i) => i.get().clone(),
                    _ => unimplemented!(),
                };
                let args = self.args(node.find(fc.args().span())?).at(&node).unwrap();
                return Some(self.call(callee, args).at(&node).unwrap());
            }
            Named => {}
            Keyed => {}
            Unary => {}
            Binary => {}
            FieldAccess => {}
            Args => {}
            Spread => {}
            Closure => {}
            Params => {}
            LetBinding => {
                let b: ast::LetBinding = node.cast()?;
                let i = match b.kind() {
                    ast::LetBindingKind::Normal(ast::Pattern::Normal(ast::Expr::Ident(i))) => i,
                    ast::LetBindingKind::Closure(i) => i,
                    _ => unimplemented!(),
                };
                let init = b.init()?;
                let name = i.get().clone();
                let value = self.node(node.find(init.span())?)?;
                self.defs.insert(name, value);
            }
            SetRule => {}
            ShowRule => {}
            Contextual => {}
            Conditional => {}
            WhileLoop => {}
            ForLoop => {}
            ModuleImport => {}
            ImportItems => {}
            RenamedImportItem => {}
            ModuleInclude => {}
            LoopBreak => {}
            LoopContinue => {}
            FuncReturn => {}
            Destructuring => {}
            DestructAssignment => {}
            LineComment => {}
            BlockComment => {}
            Error => {}
            Eof => {}
        };

        Option::None
    }

    fn block(&mut self, node: LinkedNode) -> Option<Node> {
        let parent_at = self.at;
        let mut group = vec![];

        for child in node.children() {
            // println!("{:?} -> {:?}", child, node);
            let node = self.node(child);
            if let Some(Node::Content(node)) = node {
                if node == ZERO {
                    continue;
                }
                group.push((self.at, node));
            }
        }

        self.at = parent_at;

        // only one item
        if group.len() == 1 && group[0].0 == Point::default() {
            return Some(Node::Content(group[0].1));
        }

        Some(Node::Content(
            self.store(VecItem::Group(GroupRef(group.into()), None)),
        ))
    }

    fn args(&mut self, args: LinkedNode) -> Option<Arguments> {
        let mut positional = vec![];
        let mut named = HashMap::new();

        for arg in args.cast::<ast::Args>()?.items() {
            match arg {
                ast::Arg::Pos(..) => positional.push(self.node(args.find(arg.span())?)?),
                ast::Arg::Named(n) => {
                    let name = n.name().get().clone();
                    let value = self.node(args.find(arg.span())?)?;
                    named.insert(name, value);
                }
                _ => unimplemented!(),
            }
        }

        Some(Arguments { positional, named })
    }

    fn call(&mut self, callee: EcoString, args: Arguments) -> Option<Node> {
        log::debug!("call {callee:?} with {args:?}");

        match callee.as_str() {
            "group" => {
                let hard = args.named.get("hard").map(|n| self.number(n));
                let _ = hard;

                Some(Node::Content(ZERO))
            }
            "page" => {
                let Some(Node::Dict(width_height)) = args.positional.first() else {
                    panic!("page requires width and height");
                };
                let content = args.positional.get(1).unwrap();

                self.page = Some(Page {
                    content: content.content(),
                    size: Axes::new(
                        self.number(width_height.get("w").unwrap()),
                        self.number(width_height.get("h").unwrap()),
                    ),
                });

                Some(Node::Content(ZERO))
            }
            "at" => {
                self.at = Point::new(
                    self.number(args.positional.first().unwrap()),
                    self.number(args.positional.get(1).unwrap()),
                );

                Some(Node::Content(ZERO))
            }
            "rect" => {
                let Node::Dict(d) = args.positional.first().unwrap() else {
                    panic!("rect requires xywh or lrtb");
                };

                // default rectangle style
                let styles = vec![
                    PathStyle::Stroke("#000".into()),
                    PathStyle::StrokeWidth(Scalar(1.)),
                ];

                // xywh or lrtb
                if d.get("w").is_some() {
                    let x = self.number(d.get("x").unwrap()).0;
                    let y = self.number(d.get("y").unwrap()).0;
                    let w = self.number(d.get("w").unwrap()).0;
                    let h = self.number(d.get("h").unwrap()).0;
                    Some(Node::Content(
                        self.store(VecItem::Path(PathItem {
                            d: format!(
                                "M {x} {y} L {xw} {y} L {xw} {yh} L {x} {yh} Z",
                                xw = x + w,
                                yh = y + h
                            )
                            .into(),
                            size: None,
                            styles,
                        })),
                    ))
                } else {
                    Some(Node::Content(
                        self.store(VecItem::Path(PathItem {
                            d: format!(
                                "M {l} {t} L {r} {t} L {r} {b} L {l} {b} Z",
                                l = self.number(d.get("l").unwrap()).0,
                                t = self.number(d.get("t").unwrap()).0,
                                r = self.number(d.get("r").unwrap()).0,
                                b = self.number(d.get("b").unwrap()).0,
                            )
                            .into(),
                            size: None,
                            styles,
                        })),
                    ))
                }
            }
            _ => None,
        }
    }

    fn number(&self, unwrap: &Node) -> Scalar {
        match unwrap {
            Node::Value(Value::Int(n)) => Scalar(*n as f32),
            Node::Value(Value::Float(n)) => Scalar(*n as f32),
            Node::Value(Value::Length(l)) => Scalar(l.abs.to_raw() as f32),
            _ => panic!(),
        }
    }
}

trait At<T> {
    fn at(self, node: &LinkedNode) -> StrResult<T>;
}

impl<T> At<T> for Option<T> {
    fn at(self, node: &LinkedNode) -> StrResult<T> {
        match self {
            Some(v) => Ok(v),
            None => Err(eco_format!("Expected value at {node:?}")),
        }
    }
}

#[test]
fn test() {
    snapshot_testing("playground", |mut doc, path| {
        let mut pass = typst_ts_core::vector::pass::VecIsolatePass::default();
        for page in &doc.pages {
            let result = pass.page(&doc.module, page);
            println!("{page:?} -> {result:?}");
        }

        let debug_output = typst_ts_svg_exporter::render_vector_svg_html(&mut doc, None);
        let target_dir = typst_ts_test_common::artifact_dir().join("isolate");
        std::fs::create_dir_all(&target_dir).unwrap();
        std::fs::write(
            target_dir
                .join(path.file_name().unwrap_or_default())
                .with_extension("html"),
            debug_output,
        )
        .unwrap();
    });
}

fn snapshot_testing(name: &str, f: impl Fn(VecDocument, &Path)) {
    let mut settings = insta::Settings::new();
    settings.set_prepend_module_to_snapshot(false);
    settings.set_snapshot_path(format!("fixtures/{name}/snaps"));
    settings.bind(|| {
        let glob_path = format!("fuzzers/fixtures/{name}/*.refl");
        insta::glob!(
            insta::_macro_support::get_cargo_workspace(env!("CARGO_MANIFEST_DIR")).as_path(),
            &glob_path,
            |path| {
                let contents = std::fs::read_to_string(path).unwrap();
                #[cfg(windows)]
                let contents = contents.replace("\r\n", "\n");
                let world = Source::detached(contents);
                let mut evaluator = Evaluator::default();
                let page = evaluator
                    .eval(LinkedNode::new(world.root()))
                    .with_context(|| format!("Failed to evaluate document {path:?}"))
                    .unwrap();

                let doc = VecDocument {
                    module: Module {
                        fonts: Default::default(),
                        glyphs: Default::default(),
                        items: evaluator.items.into_iter().collect(),
                    },
                    pages: vec![page],
                };

                f(doc, path);
            }
        );
    });
}
