use ffi::{GstGLBaseMixer, GstGLBaseMixerClass};
use glib::translate::*;
use gst::{result_from_gboolean, LoggableError, CAT_RUST};
use gst_video::subclass::prelude::*;

use crate::{ffi, prelude::*, GLBaseMixer};

pub trait GLBaseMixerImpl: VideoAggregatorImpl {
    fn supported_gl_api() -> crate::GLAPI {
        crate::GLAPI::OPENGL | crate::GLAPI::OPENGL3 | crate::GLAPI::GLES1 | crate::GLAPI::GLES2
    }

    fn gl_start(&self) -> Result<(), LoggableError> {
        self.parent_gl_start()
    }

    fn gl_stop(&self) {
        self.parent_gl_stop()
    }
}

pub trait GLBaseMixerImplExt: GLBaseMixerImpl {
    fn parent_gl_start(&self) -> Result<(), LoggableError> {
        unsafe {
            let data = Self::type_data();
            let parent_class = data.as_ref().parent_class() as *mut GstGLBaseMixerClass;

            (*parent_class)
                .gl_start
                .map(|f| {
                    result_from_gboolean!(
                        f(self.obj().unsafe_cast_ref::<GLBaseMixer>().to_glib_none().0),
                        CAT_RUST,
                        "Parent function `gl_start` failed"
                    )
                })
                .unwrap_or(Ok(()))
        }
    }

    fn parent_gl_stop(&self) {
        unsafe {
            let data = Self::type_data();
            let parent_class = data.as_ref().parent_class() as *mut GstGLBaseMixerClass;

            if let Some(f) = (*parent_class).gl_stop {
                f(self.obj().unsafe_cast_ref::<GLBaseMixer>().to_glib_none().0)
            }
        }
    }
}

impl<T: GLBaseMixerImpl> GLBaseMixerImplExt for T {}

unsafe impl<T: GLBaseMixerImpl> IsSubclassable<T> for GLBaseMixer {
    fn class_init(klass: &mut glib::Class<Self>) {
        Self::parent_class_init::<T>(klass);
        let klass = klass.as_mut();
        klass.supported_gl_api = T::supported_gl_api().into_glib();
        klass.gl_start = Some(gl_start::<T>);
        klass.gl_stop = Some(gl_stop::<T>);
    }
}

unsafe extern "C" fn gl_start<T: GLBaseMixerImpl>(ptr: *mut GstGLBaseMixer) -> glib::ffi::gboolean {
    let instance = &*(ptr as *mut T::Instance);
    let imp = instance.imp();

    gst::panic_to_error!(imp, false, {
        match imp.gl_start() {
            Ok(()) => true,
            Err(err) => {
                err.log_with_imp(imp);
                false
            }
        }
    })
    .into_glib()
}

unsafe extern "C" fn gl_stop<T: GLBaseMixerImpl>(ptr: *mut GstGLBaseMixer) {
    let instance = &*(ptr as *mut T::Instance);
    let imp = instance.imp();

    gst::panic_to_error!(imp, (), { imp.gl_stop() })
}
