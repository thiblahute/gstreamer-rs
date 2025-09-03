use glib::{prelude::*, translate::*};

use crate::{ffi, GLMixerPad};

pub trait GLMixerPadExtManual: 'static {
    /// Get the current texture ID for this pad.
    ///
    /// This returns the GL texture ID that contains the current input
    /// texture for this mixer pad, which can be used in the process_textures
    /// implementation to access the input texture.
    ///
    /// # Returns
    ///
    /// The GL texture ID (texture name) for the current input texture,
    /// or 0 if no texture is currently available.
    fn current_texture(&self) -> u32;
}

impl<O: IsA<GLMixerPad>> GLMixerPadExtManual for O {
    fn current_texture(&self) -> u32 {
        unsafe {
            let pad: *mut ffi::GstGLMixerPad = self.as_ref().to_glib_none().0;
            (*pad).current_texture
        }
    }
}
