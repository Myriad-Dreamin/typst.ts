use std::{
    collections::HashMap,
    num::NonZeroUsize,
    sync::{Arc, OnceLock},
};

use dashmap::DashMap;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use reflexo::{
    typst::{TypstDocument, TypstHtmlDocument, TypstPagedDocument},
    vector::ir::{Page, Transform},
};
use reflexo_typst2vec::{ir::Module, IntoTypst};
use typst::{
    introspection::Introspector,
    layout::{Frame, FrameItem, FrameKind},
};

mod ir;

// self.compute(call(frame, &self.isolate.frames), |this| todo!())

pub struct Vec2LinPass {
    rev: NonZeroUsize,
    isolate: Isolate,
}

impl Vec2LinPass {
    /// Interprets the changes in the given module and pages.
    pub fn interpret_changes(&mut self, module: &Module, pages: &[Page]) {
        // // render the document
        // let mut t = CanvasTask::<DefaultExportFeature>::default();

        // let mut ct = t.fork_canvas_render_task(module);

        // let pages: Vec<CanvasPage> = pages
        //     .iter()
        //     .enumerate()
        //     .map(|(idx, Page { content, size })| {
        //         if idx < self.pages.len() && self.pages[idx].content == *content {
        //             return self.pages[idx].clone();
        //         }

        //         CanvasPage {
        //             content: *content,
        //             elem: ct.render_item(content),
        //             size: *size,
        //         }
        //     })
        //     .collect();

        // self.pages = pages;

        todo!()
    }
}

struct Vec2LinWorker<'a> {
    rev: NonZeroUsize,
    isolate: &'a Isolate,
}

impl<'a> Vec2LinWorker<'a> {
    pub fn paged(&self, doc: &TypstPagedDocument) -> Vec<Page> {
        doc.pages
            .par_iter()
            .map(|page| {
                let state = State::new(&doc.introspector, page.frame.size().into_typst());
                let frame = self.frame(state, &page.frame);

                todo!()
            })
            .collect()
    }

    fn frame(&self, state: State, frame: &typst::layout::Frame) -> Arc<ir::Frame> {
        self.compute(call(frame, &self.isolate.frames), |this| {
            let frame_size = match frame.kind() {
                FrameKind::Hard => Some(frame.size().into_typst()),
                FrameKind::Soft => None,
            };
            let mut state = state;
            if let Some(sz) = &frame_size {
                state = state.with_transform(Transform::identity()).with_size(*sz);
            }

            todo!()
        })
    }

    fn compute<T, F>(&self, entry: ComputeEntry<T>, f: F) -> Arc<T>
    where
        F: FnOnce(&Self) -> T,
    {
        entry.get_or_init(|| Arc::new(f(self))).clone()
    }
}

type ComputeEntry<T> = Arc<OnceLock<Arc<T>>>;

type CacheMap<T, P> = (
    DashMap<u128, Arc<OnceLock<Arc<T>>>>,
    std::marker::PhantomData<P>,
);

struct Isolate {
    frames: CacheMap<ir::Frame, typst::layout::Frame>,
}

fn call<P: std::hash::Hash, T>(frame: &P, cache: &CacheMap<T, P>) -> ComputeEntry<T> {
    cache
        .0
        .entry(reflexo::hash::hash128(frame))
        .or_insert_with(|| Arc::new(OnceLock::new()))
        .clone()
}

#[derive(Clone, Copy)]
struct State<'a> {
    introspector: &'a Introspector,
    /// The transform of the current item.
    pub transform: Transform,
    /// The size of the first hard frame in the hierarchy.
    pub size: ir::Size,
}

impl State<'_> {
    fn new(introspector: &Introspector, size: ir::Size) -> State {
        State {
            introspector,
            transform: Transform::identity(),
            size,
        }
    }

    /// Pre translate the current item's transform.
    pub fn pre_translate(self, pos: ir::Point) -> Self {
        self.pre_concat(Transform::from_translate(pos.x, pos.y))
    }

    /// Pre concat the current item's transform.
    pub fn pre_concat(self, transform: ir::Transform) -> Self {
        Self {
            transform: self.transform.pre_concat(transform),
            ..self
        }
    }

    /// Sets the size of the first hard frame in the hierarchy.
    pub fn with_size(self, size: ir::Size) -> Self {
        Self { size, ..self }
    }

    /// Sets the current item's transform.
    pub fn with_transform(self, transform: ir::Transform) -> Self {
        Self { transform, ..self }
    }

    pub fn inv_transform(&self) -> ir::Transform {
        self.transform.invert().unwrap()
    }

    pub fn body_inv_transform(&self) -> ir::Transform {
        ir::Transform::from_scale(self.size.x, self.size.y)
            .post_concat(self.transform.invert().unwrap())
    }
}
