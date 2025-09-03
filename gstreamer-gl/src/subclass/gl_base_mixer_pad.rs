// Take a look at the license at the top of the repository in the LICENSE file.

use glib::prelude::*;
use gst_base::subclass::prelude::*;
use gst_video::subclass::prelude::*;

use crate::GLBaseMixerPad;

pub trait GLBaseMixerPadImpl:
    VideoAggregatorPadImpl + ObjectSubclass<Type: IsA<GLBaseMixerPad>>
{
    // GLBaseMixerPad doesn't have additional virtual methods beyond VideoAggregatorPad
    // This trait exists to provide the proper inheritance chain for GLMixerPad
}

pub trait GLBaseMixerPadImplExt: GLBaseMixerPadImpl {
    // No additional parent methods to call since GLBaseMixerPad doesn't override any
}

impl<T: GLBaseMixerPadImpl> GLBaseMixerPadImplExt for T {}

unsafe impl<T: GLBaseMixerPadImpl> IsSubclassable<T> for GLBaseMixerPad {
    fn class_init(klass: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(klass);
    }
}
