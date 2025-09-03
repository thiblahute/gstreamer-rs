mod gl_base_filter;
#[cfg(feature = "v1_24")]
#[cfg_attr(docsrs, doc(cfg(feature = "v1_24")))]
mod gl_base_mixer;
#[cfg(feature = "v1_24")]
#[cfg_attr(docsrs, doc(cfg(feature = "v1_24")))]
mod gl_base_mixer_pad;
#[cfg(feature = "v1_18")]
#[cfg_attr(docsrs, doc(cfg(feature = "v1_18")))]
mod gl_base_src;
mod gl_filter;
#[cfg(feature = "v1_24")]
#[cfg_attr(docsrs, doc(cfg(feature = "v1_24")))]
mod gl_mixer;
#[cfg(feature = "v1_24")]
#[cfg_attr(docsrs, doc(cfg(feature = "v1_24")))]
mod gl_mixer_pad;

pub use self::gl_filter::GLFilterMode;
#[cfg(feature = "v1_24")]
#[cfg_attr(docsrs, doc(cfg(feature = "v1_24")))]
pub use self::gl_mixer::GLMixerMode;

pub mod prelude {
    #[doc(hidden)]
    pub use gst_video::subclass::prelude::*;

    #[cfg(feature = "v1_24")]
    #[cfg_attr(docsrs, doc(cfg(feature = "v1_24")))]
    pub use super::gl_base_mixer::{GLBaseMixerImpl, GLBaseMixerImplExt};
    #[cfg(feature = "v1_24")]
    #[cfg_attr(docsrs, doc(cfg(feature = "v1_24")))]
    pub use super::gl_base_mixer_pad::{GLBaseMixerPadImpl, GLBaseMixerPadImplExt};
    #[cfg(feature = "v1_18")]
    #[cfg_attr(docsrs, doc(cfg(feature = "v1_18")))]
    pub use super::gl_base_src::{GLBaseSrcImpl, GLBaseSrcImplExt};
    #[cfg(feature = "v1_24")]
    #[cfg_attr(docsrs, doc(cfg(feature = "v1_24")))]
    pub use super::gl_mixer::{GLMixerImpl, GLMixerImplExt};
    #[cfg(feature = "v1_24")]
    #[cfg_attr(docsrs, doc(cfg(feature = "v1_24")))]
    pub use super::gl_mixer_pad::{GLMixerPadImpl, GLMixerPadImplExt};
    pub use super::{
        gl_base_filter::{GLBaseFilterImpl, GLBaseFilterImplExt},
        gl_filter::{GLFilterImpl, GLFilterImplExt},
    };
}
