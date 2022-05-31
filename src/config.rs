use crate::traits::PlatformIfConfigT;
use derive_builder::Builder;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum Layer {
    /// TAP, Ethernet-like interface with L2 capabilities
    L2,
    /// TUN, point-to-point IP interface
    L3,
}

#[derive(Builder)]
pub struct IfConfig<P: PlatformIfConfigT> {
    /// Interface name on Unix and interface alias on Windows.
    pub name: String,
    /// Interface type: TUN or TAP.
    #[builder(default = "Layer::L3")]
    pub layer: Layer,

    #[allow(dead_code)]
    #[builder(setter(custom))]
    #[builder(default = "P::default()")]
    pub platform: P,
}

impl<P: PlatformIfConfigT + Clone> IfConfigBuilder<P> {
    /// Platform-specific settings
    pub fn platform<F, E>(&mut self, f: F) -> Result<&mut Self, E>
    where
        F: Fn(P::Builder) -> Result<P, E>,
    {
        let builder = P::Builder::default();
        self.platform = Some(f(builder)?);
        Ok(self)
    }
}
