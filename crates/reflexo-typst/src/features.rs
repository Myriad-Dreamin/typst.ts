use std::sync::LazyLock;

use super::diag::DiagnosticFormat;
use crate::typst::prelude::*;

#[derive(Debug, Clone, Copy)]
pub struct FeatureSlot(u16);

/// The global feature allocator.
/// already used: 1
static ALLOCATOR: LazyLock<std::sync::atomic::AtomicU16> =
    LazyLock::new(|| std::sync::atomic::AtomicU16::new(1));

#[derive(Debug, Default)]
pub struct LazyFeatureSlot(std::sync::OnceLock<FeatureSlot>);

impl From<&LazyFeatureSlot> for FeatureSlot {
    fn from(slot: &LazyFeatureSlot) -> Self {
        *slot.0.get_or_init(|| {
            FeatureSlot(ALLOCATOR.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
        })
    }
}

#[derive(Debug, Default, Clone)]
pub struct FeatureSet {
    features: Vec<EcoString>,
}

impl FeatureSet {
    pub fn configure<T: CompileFeature<V>, V>(self, feature: &T, value: V) -> Self {
        feature.configure(self, value)
    }

    pub fn configure_slot(mut self, slot: impl Into<FeatureSlot>, value: EcoString) -> Self {
        let slot = slot.into().0 as usize;
        if slot >= self.features.len() {
            self.features.resize(slot + 1, "".into());
        }

        self.features[slot] = value;
        self
    }

    fn slot(&self, slot: impl Into<FeatureSlot>) -> Option<&EcoString> {
        let slot = slot.into().0 as usize;

        self.features.get(slot)
    }
}

pub trait CompileFeature<T> {
    fn configure(&self, features: FeatureSet, value: T) -> FeatureSet;
    fn retrieve(&self, features: &FeatureSet) -> T;
}

#[derive(Debug, Clone, Copy)]
pub struct DiagFmtFeature;
const DIAG_FEATURE: FeatureSlot = FeatureSlot(0);
pub static DIAG_FMT_FEATURE: DiagFmtFeature = DiagFmtFeature;

impl CompileFeature<DiagnosticFormat> for DiagFmtFeature {
    fn configure(&self, features: FeatureSet, value: DiagnosticFormat) -> FeatureSet {
        features.configure_slot(
            DIAG_FEATURE,
            match value {
                DiagnosticFormat::Human => "",
                DiagnosticFormat::Short => "s",
            }
            .into(),
        )
    }

    fn retrieve(&self, features: &FeatureSet) -> DiagnosticFormat {
        features
            .slot(DIAG_FEATURE)
            .and_then(|s| (s == "s").then_some(DiagnosticFormat::Short))
            .unwrap_or_default()
    }
}

#[derive(Debug, Default)]
pub struct BuiltinFeature<T>(LazyFeatureSlot, std::marker::PhantomData<T>);

impl<T> BuiltinFeature<T> {
    pub const fn new() -> Self {
        Self(
            LazyFeatureSlot(std::sync::OnceLock::new()),
            std::marker::PhantomData,
        )
    }
}

pub static WITH_COMPILING_STATUS_FEATURE: BuiltinFeature<bool> = BuiltinFeature::<bool>::new();

impl CompileFeature<bool> for BuiltinFeature<bool> {
    fn configure(&self, features: FeatureSet, value: bool) -> FeatureSet {
        features.configure_slot(&self.0, if value { "1" } else { "" }.into())
    }

    fn retrieve(&self, features: &FeatureSet) -> bool {
        features
            .slot(&self.0)
            .and_then(|s| (s == "1").then_some(true))
            .unwrap_or_default()
    }
}
