// Take a look at the license at the top of the repository in the LICENSE file.

use glib::{prelude::*, translate::*};

use crate::{ffi, GLShader};

pub trait GLShaderExtManual: IsA<GLShader> + 'static {
    #[doc(alias = "gst_gl_shader_set_uniform_matrix_2fv")]
    fn set_uniform_matrix_2fv(&self, name: &str, count: i32, transpose: bool, value: &[f32]) {
        let len = (count * 4) as usize;
        assert!(
            value.len() >= len,
            "Need at least {} values, got {}",
            len,
            value.len()
        );
        unsafe {
            ffi::gst_gl_shader_set_uniform_matrix_2fv(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                count,
                transpose.into_glib(),
                value.as_ptr(),
            );
        }
    }

    #[doc(alias = "gst_gl_shader_set_uniform_matrix_2x3fv")]
    fn set_uniform_matrix_2x3fv(&self, name: &str, count: i32, transpose: bool, value: &[f32]) {
        let len = (count * 6) as usize;
        assert!(
            value.len() >= len,
            "Need at least {} values, got {}",
            len,
            value.len()
        );
        unsafe {
            ffi::gst_gl_shader_set_uniform_matrix_2x3fv(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                count,
                transpose.into_glib(),
                value.as_ptr(),
            );
        }
    }

    #[doc(alias = "gst_gl_shader_set_uniform_matrix_2x4fv")]
    fn set_uniform_matrix_2x4fv(&self, name: &str, count: i32, transpose: bool, value: &[f32]) {
        let len = (count * 8) as usize;
        assert!(
            value.len() >= len,
            "Need at least {} values, got {}",
            len,
            value.len()
        );
        unsafe {
            ffi::gst_gl_shader_set_uniform_matrix_2x4fv(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                count,
                transpose.into_glib(),
                value.as_ptr(),
            );
        }
    }

    #[doc(alias = "gst_gl_shader_set_uniform_matrix_3fv")]
    fn set_uniform_matrix_3fv(&self, name: &str, count: i32, transpose: bool, value: &[f32]) {
        let len = (count * 9) as usize;
        assert!(
            value.len() >= len,
            "Need at least {} values, got {}",
            len,
            value.len()
        );
        unsafe {
            ffi::gst_gl_shader_set_uniform_matrix_3fv(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                count,
                transpose.into_glib(),
                value.as_ptr(),
            );
        }
    }

    #[doc(alias = "gst_gl_shader_set_uniform_matrix_3x2fv")]
    fn set_uniform_matrix_3x2fv(&self, name: &str, count: i32, transpose: bool, value: &[f32]) {
        let len = (count * 6) as usize;
        assert!(
            value.len() >= len,
            "Need at least {} values, got {}",
            len,
            value.len()
        );
        unsafe {
            ffi::gst_gl_shader_set_uniform_matrix_3x2fv(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                count,
                transpose.into_glib(),
                value.as_ptr(),
            );
        }
    }

    #[doc(alias = "gst_gl_shader_set_uniform_matrix_3x4fv")]
    fn set_uniform_matrix_3x4fv(&self, name: &str, count: i32, transpose: bool, value: &[f32]) {
        let len = (count * 12) as usize;
        assert!(
            value.len() >= len,
            "Need at least {} values, got {}",
            len,
            value.len()
        );
        unsafe {
            ffi::gst_gl_shader_set_uniform_matrix_3x4fv(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                count,
                transpose.into_glib(),
                value.as_ptr(),
            );
        }
    }

    #[doc(alias = "gst_gl_shader_set_uniform_matrix_4fv")]
    fn set_uniform_matrix_4fv(&self, name: &str, count: i32, transpose: bool, value: &[f32]) {
        let len = (count * 16) as usize;
        assert!(
            value.len() >= len,
            "Need at least {} values, got {}",
            len,
            value.len()
        );
        unsafe {
            ffi::gst_gl_shader_set_uniform_matrix_4fv(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                count,
                transpose.into_glib(),
                value.as_ptr(),
            );
        }
    }

    #[doc(alias = "gst_gl_shader_set_uniform_matrix_4x2fv")]
    fn set_uniform_matrix_4x2fv(&self, name: &str, count: i32, transpose: bool, value: &[f32]) {
        let len = (count * 8) as usize;
        assert!(
            value.len() >= len,
            "Need at least {} values, got {}",
            len,
            value.len()
        );
        unsafe {
            ffi::gst_gl_shader_set_uniform_matrix_4x2fv(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                count,
                transpose.into_glib(),
                value.as_ptr(),
            );
        }
    }

    #[doc(alias = "gst_gl_shader_set_uniform_matrix_4x3fv")]
    fn set_uniform_matrix_4x3fv(&self, name: &str, count: i32, transpose: bool, value: &[f32]) {
        let len = (count * 12) as usize;
        assert!(
            value.len() >= len,
            "Need at least {} values, got {}",
            len,
            value.len()
        );
        unsafe {
            ffi::gst_gl_shader_set_uniform_matrix_4x3fv(
                self.as_ref().to_glib_none().0,
                name.to_glib_none().0,
                count,
                transpose.into_glib(),
                value.as_ptr(),
            );
        }
    }
}

impl<O: IsA<GLShader>> GLShaderExtManual for O {}
