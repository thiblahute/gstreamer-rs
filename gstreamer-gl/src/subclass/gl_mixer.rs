use ffi::{GstGLMemory, GstGLMixer, GstGLMixerClass};
use glib::translate::*;
use gst::{result_from_gboolean, LoggableError, CAT_RUST};

use super::prelude::*;
use crate::{ffi, prelude::*, GLMemory, GLMixer};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum GLMixerMode {
    Buffers,
    Textures,
}

pub trait GLMixerImpl: GLBaseMixerImpl {
    const MODE: GLMixerMode;
    const ADD_RGBA_PAD_TEMPLATES: bool = false;

    fn process_buffers(&self, outbuf: &mut gst::BufferRef) -> Result<(), LoggableError> {
        self.parent_process_buffers(outbuf)
    }

    fn process_textures(&self, out_tex: &GLMemory) -> Result<(), LoggableError> {
        self.parent_process_textures(out_tex)
    }
}

pub trait GLMixerImplExt: GLMixerImpl {
    fn parent_process_buffers(&self, outbuf: &mut gst::BufferRef) -> Result<(), LoggableError> {
        unsafe {
            let data = Self::type_data();
            let parent_class = data.as_ref().parent_class() as *mut GstGLMixerClass;

            (*parent_class)
                .process_buffers
                .map(|f| {
                    result_from_gboolean!(
                        f(
                            self.obj().unsafe_cast_ref::<GLMixer>().to_glib_none().0,
                            outbuf.as_mut_ptr(),
                        ),
                        CAT_RUST,
                        "Parent function `process_buffers` failed"
                    )
                })
                .unwrap_or(Ok(()))
        }
    }

    fn parent_process_textures(&self, out_tex: &GLMemory) -> Result<(), LoggableError> {
        unsafe {
            let data = Self::type_data();
            let parent_class = data.as_ref().parent_class() as *mut GstGLMixerClass;

            (*parent_class)
                .process_textures
                .map(|f| {
                    result_from_gboolean!(
                        f(
                            self.obj().unsafe_cast_ref::<GLMixer>().to_glib_none().0,
                            out_tex.to_glib_none().0,
                        ),
                        CAT_RUST,
                        "Parent function `process_textures` failed"
                    )
                })
                .unwrap_or(Ok(()))
        }
    }
}

impl<T: GLMixerImpl> GLMixerImplExt for T {}

unsafe impl<T: GLMixerImpl> IsSubclassable<T> for GLMixer {
    fn class_init(klass: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(klass);
        let klass = klass.as_mut();

        match <T as GLMixerImpl>::MODE {
            GLMixerMode::Buffers => {
                klass.process_buffers = Some(process_buffers::<T>);
                klass.process_textures = None;
            }
            GLMixerMode::Textures => {
                klass.process_buffers = None;
                klass.process_textures = Some(process_textures::<T>);
            }
        }

        if <T as GLMixerImpl>::ADD_RGBA_PAD_TEMPLATES {
            unsafe { ffi::gst_gl_mixer_class_add_rgba_pad_templates(klass) }
        }
    }
}

unsafe extern "C" fn process_buffers<T: GLMixerImpl>(
    ptr: *mut GstGLMixer,
    outbuf: *mut gst::ffi::GstBuffer,
) -> glib::ffi::gboolean {
    let instance = &*(ptr as *mut T::Instance);
    let imp = instance.imp();

    gst::panic_to_error!(imp, false, {
        match imp.process_buffers(&mut *(outbuf as *mut gst::BufferRef)) {
            Ok(()) => true,
            Err(err) => {
                err.log_with_imp(imp);
                false
            }
        }
    })
    .into_glib()
}

unsafe extern "C" fn process_textures<T: GLMixerImpl>(
    ptr: *mut GstGLMixer,
    out_tex: *mut GstGLMemory,
) -> glib::ffi::gboolean {
    let instance = &*(ptr as *mut T::Instance);
    let imp = instance.imp();

    gst::panic_to_error!(imp, false, {
        match imp.process_textures(&from_glib_borrow(out_tex)) {
            Ok(()) => true,
            Err(err) => {
                err.log_with_imp(imp);
                false
            }
        }
    })
    .into_glib()
}
