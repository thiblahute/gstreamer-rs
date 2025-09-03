// Take a look at the license at the top of the repository in the LICENSE file.

use glib::prelude::*;
use gst_base::subclass::prelude::*;

use crate::GLMixerPad;

use super::gl_base_mixer_pad::GLBaseMixerPadImpl;

pub trait GLMixerPadImpl: GLBaseMixerPadImpl + ObjectSubclass<Type: IsA<GLMixerPad>> {
    // GLMixerPad doesn't have additional virtual methods beyond GLBaseMixerPad
    // This trait exists to provide the proper inheritance chain and type for custom GLMixerPads
}

pub trait GLMixerPadImplExt: GLMixerPadImpl {
    // No additional parent methods to call since GLMixerPad doesn't override any
}

impl<T: GLMixerPadImpl> GLMixerPadImplExt for T {}

unsafe impl<T: GLMixerPadImpl> IsSubclassable<T> for GLMixerPad {
    fn class_init(klass: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(klass);
    }
}
