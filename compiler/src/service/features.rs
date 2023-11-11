use ecow::EcoString;
use once_cell::sync::Lazy;

#[derive(Debug, Clone, Copy)]
pub struct FeatureSlot(u16);

/// The global feature allocator.
/// already used: 1
static ALLOCATOR: Lazy<std::sync::atomic::AtomicU16> =
    Lazy::new(|| std::sync::atomic::AtomicU16::new(1));

#[derive(Default)]
pub struct LazyFeatureSlot(once_cell::sync::OnceCell<FeatureSlot>);

impl From<&LazyFeatureSlot> for FeatureSlot {
    fn from(slot: &LazyFeatureSlot) -> Self {
        *slot.0.get_or_init(|| {
            FeatureSlot(ALLOCATOR.fetch_add(1, std::sync::atomic::Ordering::SeqCst))
        })
    }
}

#[derive(Default)]
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

#[derive(Default)]
pub struct BuiltinFeature<T>(LazyFeatureSlot, std::marker::PhantomData<T>);

impl<T> BuiltinFeature<T> {
    pub const fn new() -> Self {
        Self(
            LazyFeatureSlot(once_cell::sync::OnceCell::new()),
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
