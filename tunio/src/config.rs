use crate::traits::PlatformIfConfigT;
use derive_builder::Builder;

#[derive(Builder, Default)]
pub struct IfConfig<P: PlatformIfConfigT> {
    pub(crate) name: String,

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
