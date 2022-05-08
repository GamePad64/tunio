use crate::traits::PlatformIfConfigT;
use derive_builder::Builder;

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum Layer {
    L2,
    L3,
}

#[derive(Builder)]
pub struct IfConfig<P: PlatformIfConfigT> {
    pub(crate) name: String,
    #[builder(default = "Layer::L3")]
    pub(crate) layer: Layer,

    #[allow(dead_code)]
    #[builder(setter(custom))]
    pub(crate) platform: P,
}

impl<P: PlatformIfConfigT + Clone> IfConfigBuilder<P> {
    pub fn platform<F>(&mut self, f: F) -> &mut Self
    where
        F: Fn(P::Builder) -> P,
    {
        let builder = P::Builder::default();
        self.platform = Some(f(builder));
        self
    }
}
