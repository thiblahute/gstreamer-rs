// This example demonstrates how to implement a custom OpenGL mixer based on GLMixer.
#![allow(clippy::non_send_fields_in_send_ty)]

use anyhow::{Context, Error};
use gst::prelude::*;
use gst_gl::prelude::*;

use std::sync::Once;

static GL_INIT: Once = Once::new();

#[path = "../examples-common.rs"]
mod examples_common;

/// Fragment shader for alpha blending multiple input textures
const FRAGMENT_SHADER: &str = r#"#version 150
// Vertex shader provides texture coordinates
in vec2 v_texcoord;

// Input texture samplers
uniform sampler2D tex1;
uniform sampler2D tex2;

// Alpha values for blending
uniform float alpha1;
uniform float alpha2;

// Output color
out vec4 fragColor;

void main() {
    vec4 color1 = texture(tex1, v_texcoord);
    vec4 color2 = texture(tex2, v_texcoord);
 
    // Alpha blend: mix the two textures with their respective alpha values
    vec4 blended1 = color1 * alpha1;
    vec4 blended2 = color2 * alpha2;
 
    // Combine the alpha-blended colors
    fragColor = blended1 + blended2 * (1.0 - alpha1);
}
"#;

// Our custom OpenGL mixer element is defined in this module.
mod gl_mixer {
    use gst::subclass::prelude::*;
    use gst_gl::prelude::*;
    use gst_gl::subclass::prelude::*;

    // In the imp submodule we include the actual implementation of the mixer.
    mod imp {
        use std::sync::Mutex;

        use super::*;
        use gst::CAT_DEFAULT;
        use gst_gl::subclass::prelude::*;
        use gst_gl::subclass::GLMixerMode;

        // Custom GLMixerPad with alpha property
        mod mixer_pad {
            use super::*;

            #[derive(Clone)]
            struct Settings {
                alpha: f32,
            }

            impl Default for Settings {
                fn default() -> Self {
                    Self { alpha: 1.0 }
                }
            }

            #[derive(Default)]
            pub struct SimpleGLMixerPad {
                settings: std::sync::Mutex<Settings>,
            }

            #[glib::object_subclass]
            impl ObjectSubclass for SimpleGLMixerPad {
                const NAME: &'static str = "SimpleGLMixerPad";
                type Type = super::SimpleGLMixerPad;
                type ParentType = gst_gl::GLMixerPad;
            }

            impl ObjectImpl for SimpleGLMixerPad {
                fn properties() -> &'static [glib::ParamSpec] {
                    static PROPERTIES: std::sync::OnceLock<Vec<glib::ParamSpec>> =
                        std::sync::OnceLock::new();
                    PROPERTIES.get_or_init(|| {
                        vec![glib::ParamSpecFloat::builder("alpha")
                            .nick("Alpha")
                            .blurb("Alpha value for this pad (0.0 = transparent, 1.0 = opaque)")
                            .minimum(0.0)
                            .maximum(1.0)
                            .default_value(1.0)
                            .build()]
                    })
                }

                fn set_property(&self, _id: usize, value: &glib::Value, pspec: &glib::ParamSpec) {
                    match pspec.name() {
                        "alpha" => {
                            self.settings.lock().unwrap().alpha = value.get().unwrap();
                        }
                        _ => unimplemented!(),
                    }
                }

                fn property(&self, _id: usize, pspec: &glib::ParamSpec) -> glib::Value {
                    match pspec.name() {
                        "alpha" => self.settings.lock().unwrap().alpha.to_value(),
                        _ => unimplemented!(),
                    }
                }
            }

            impl GstObjectImpl for SimpleGLMixerPad {}

            impl PadImpl for SimpleGLMixerPad {}

            impl AggregatorPadImpl for SimpleGLMixerPad {}

            impl VideoAggregatorPadImpl for SimpleGLMixerPad {}

            impl GLBaseMixerPadImpl for SimpleGLMixerPad {}

            impl GLMixerPadImpl for SimpleGLMixerPad {}

            impl SimpleGLMixerPad {
                pub fn alpha(&self) -> f32 {
                    self.settings.lock().unwrap().alpha
                }
            }
        }

        // Define the pad wrapper type
        glib::wrapper! {
            pub struct SimpleGLMixerPad(ObjectSubclass<mixer_pad::SimpleGLMixerPad>) @extends gst_gl::GLMixerPad, gst_gl::GLBaseMixerPad, gst_video::VideoAggregatorPad, gst_base::AggregatorPad, gst::Pad, gst::Object;
        }

        // This is the private data of our mixer.
        #[derive(Default)]
        pub struct SimpleGLMixer {
            shader: Mutex<Option<gst_gl::GLShader>>,
        }

        impl SimpleGLMixer {
            fn initialize_gl(&self, context: &gst_gl::GLContext) -> Result<(), gst::LoggableError> {
                crate::GL_INIT.call_once(|| {
                    gst::info!(
                        gst::CAT_DEFAULT,
                        imp = self,
                        "Initializing OpenGL function pointers"
                    );

                    // Load OpenGL function pointers using the GStreamer GL context
                    gl::load_with(|name| context.proc_address(name) as *const std::ffi::c_void);
                });
                Ok(())
            }

            fn create_shader(&self, context: &gst_gl::GLContext) -> Result<(), gst::LoggableError> {
                const GL_VERTEX_SHADER: u32 = 0x8B31;
                const GL_FRAGMENT_SHADER: u32 = 0x8B30;
                let shader = gst_gl::GLShader::new(context);

                // Create vertex shader that matches GStreamer's glvideomixer
                let vertex_src = r#"#version 150
                    in vec4 a_position;
                    in vec2 a_texcoord;
                    uniform mat4 u_transformation;
                    out vec2 v_texcoord;

                    void main() {
                        gl_Position = u_transformation * a_position;
                        v_texcoord = a_texcoord;
                    }
                "#;

                let vertex = gst_gl::GLSLStage::with_strings(
                    context,
                    GL_VERTEX_SHADER,
                    gst_gl::GLSLVersion::_150,
                    gst_gl::GLSLProfile::empty(), // NONE profile
                    &[vertex_src],
                );
                vertex.compile().unwrap();
                shader.attach_unlocked(&vertex)?;

                gst::debug!(gst::CAT_DEFAULT, imp = self, "Compiling fragment shader");

                // Create and compile fragment shader
                let fragment = gst_gl::GLSLStage::with_strings(
                    context,
                    GL_FRAGMENT_SHADER,
                    gst_gl::GLSLVersion::_150,
                    gst_gl::GLSLProfile::empty(), // NONE profile
                    &[super::super::FRAGMENT_SHADER],
                );
                fragment.compile().unwrap();
                shader.attach_unlocked(&fragment)?;
                shader.link().unwrap();

                gst::info!(
                    gst::CAT_DEFAULT,
                    imp = self,
                    "Successfully compiled and linked shader"
                );

                *self.shader.lock().unwrap() = Some(shader);
                Ok(())
            }
        }

        // This trait registers our type with the GObject object system and
        // provides the entry points for creating a new instance and setting
        // up the class data.
        #[glib::object_subclass]
        impl ObjectSubclass for SimpleGLMixer {
            const NAME: &'static str = "SimpleGLMixer";
            type Type = super::SimpleGLMixer;
            type ParentType = gst_gl::GLMixer;
            type Interfaces = (gst::ChildProxy,);
        }

        // Implementation of glib::Object virtual methods.
        impl ObjectImpl for SimpleGLMixer {}

        // Implementation of gst::Object virtual methods.
        impl GstObjectImpl for SimpleGLMixer {}

        // Implementation of gst::Element virtual methods.
        impl ElementImpl for SimpleGLMixer {
            // The element specific metadata. This information is what is visible from
            // gst-inspect-1.0 and can also be programmatically retrieved from the gst::Registry
            // after initial registration without having to load the plugin in memory.
            fn metadata() -> Option<&'static gst::subclass::ElementMetadata> {
                static ELEMENT_METADATA: std::sync::OnceLock<gst::subclass::ElementMetadata> =
                    std::sync::OnceLock::new();

                Some(ELEMENT_METADATA.get_or_init(|| {
                    gst::subclass::ElementMetadata::new(
                        "Simple GL Mixer",
                        "Mixer/Video",
                        "A simple OpenGL-based video mixer",
                        "GStreamer Rust Examples",
                    )
                }))
            }

            fn pad_templates() -> &'static [gst::PadTemplate] {
                static PAD_TEMPLATES: std::sync::OnceLock<Vec<gst::PadTemplate>> =
                    std::sync::OnceLock::new();

                PAD_TEMPLATES.get_or_init(|| {
                    // Create caps for GL memory
                    let mut caps = gst::Caps::builder_full()
                        .structure_with_features(
                            gst::Structure::builder("video/x-raw")
                                .field("format", gst::List::new(["RGBA"]))
                                .field("width", gst::IntRange::new(1, i32::MAX))
                                .field("height", gst::IntRange::new(1, i32::MAX))
                                .field(
                                    "framerate",
                                    gst::FractionRange::new(
                                        gst::Fraction::new(0, 1),
                                        gst::Fraction::new(i32::MAX, 1),
                                    ),
                                )
                                .build(),
                            gst_gl::CAPS_FEATURES_MEMORY_GL_MEMORY.clone(),
                        )
                        .build();

                    // Create sink pad template using our custom pad type
                    vec![
                        gst::PadTemplate::with_gtype(
                            "sink_%u",
                            gst::PadDirection::Sink,
                            gst::PadPresence::Request,
                            &caps,
                            SimpleGLMixerPad::static_type(),
                        )
                        .unwrap(),
                        gst::PadTemplate::with_gtype(
                            "src",
                            gst::PadDirection::Src,
                            gst::PadPresence::Always,
                            &caps,
                            gst_base::AggregatorPad::static_type(),
                        )
                        .unwrap(),
                    ]
                })
            }

            fn request_new_pad(
                &self,
                templ: &gst::PadTemplate,
                name: Option<&str>,
                caps: Option<&gst::Caps>,
            ) -> Option<gst::Pad> {
                let element = self.obj();
                let pad = self.parent_request_new_pad(templ, name, caps)?;
                element.child_added(&pad, &pad.name());
                Some(pad)
            }
        }

        // Implementation of gst_base::Aggregator virtual methods.
        impl AggregatorImpl for SimpleGLMixer {}

        // Implementation of gst_video::VideoAggregator virtual methods.
        impl VideoAggregatorImpl for SimpleGLMixer {}

        // Implementation of gst_gl::GLBaseMixer virtual methods.
        impl GLBaseMixerImpl for SimpleGLMixer {
            fn supported_gl_api() -> gst_gl::GLAPI {
                gst_gl::GLAPI::OPENGL | gst_gl::GLAPI::OPENGL3 | gst_gl::GLAPI::GLES2
            }

            fn gl_start(&self) -> Result<(), gst::LoggableError> {
                gst::info!(gst::CAT_DEFAULT, imp = self, "GL context started");

                if let Some(context) =
                    GLBaseMixerExt::context(&*self.obj().upcast_ref::<gst_gl::GLBaseMixer>())
                {
                    // Initialize OpenGL function pointers
                    self.initialize_gl(&context)?;
                    // Create shader
                    self.create_shader(&context)?;
                } else {
                    return Err(gst::loggable_error!(
                        gst::CAT_DEFAULT,
                        "No GL context available"
                    ));
                }

                Ok(())
            }

            fn gl_stop(&self) {
                gst::info!(gst::CAT_DEFAULT, imp = self, "GL context stopped");
            }
        }

        impl GLMixerImpl for SimpleGLMixer {
            const MODE: GLMixerMode = GLMixerMode::Textures;

            fn process_textures(
                &self,
                out_tex: &gst_gl::GLMemory,
            ) -> Result<(), gst::LoggableError> {
                gst::debug!(
                    gst::CAT_DEFAULT,
                    imp = self,
                    "Processing textures with shader-based alpha blending"
                );

                // Get the element instance (the GLMixer)
                let obj = self.obj();
                let mixer = obj.upcast_ref::<gst_gl::GLMixer>();

                // Get the GL context from the GLBaseMixer
                let context = GLBaseMixerExt::context(mixer).ok_or_else(|| {
                    gst::loggable_error!(gst::CAT_DEFAULT, "No GL context available")
                })?;

                let framebuffer = mixer.framebuffer();

                let sink_pads = mixer.sink_pads();
                gst::debug!(
                    gst::CAT_DEFAULT,
                    imp = self,
                    "Found {} sink pads",
                    sink_pads.len()
                );

                // Get input pads with valid textures
                let mut input_pads = Vec::new();
                for pad in sink_pads.iter() {
                    let gl_pad = pad.downcast_ref::<gst_gl::GLMixerPad>().unwrap();
                    let texture_id = gl_pad.current_texture();
                    if texture_id != 0 {
                        input_pads.push(gl_pad);
                        gst::debug!(
                            gst::CAT_DEFAULT,
                            imp = self,
                            "Found input texture: {}",
                            texture_id
                        );
                    }
                }

                if input_pads.is_empty() {
                    gst::debug!(gst::CAT_DEFAULT, imp = self, "No input textures available");
                    return Ok(());
                }

                // Get alpha values from individual pads
                let input_textures: Vec<(u32, f32)> = input_pads
                    .iter()
                    .map(|pad| {
                        let texture_id = pad.current_texture();
                        // Try to downcast to our custom pad type to get alpha
                        let alpha = if let Some(custom_pad) = pad.downcast_ref::<SimpleGLMixerPad>()
                        {
                            custom_pad.imp().alpha()
                        } else {
                            1.0 // Default alpha if not custom pad
                        };
                        (texture_id, alpha)
                    })
                    .collect();

                // Store values we need after the closure
                let num_textures = input_pads.len();

                // Get the output texture dimensions from the mixer's video info
                let video_info = self.obj().video_info().ok_or_else(|| {
                    gst::loggable_error!(gst::CAT_DEFAULT, "No video info available")
                })?;
                let tex_width = video_info.width() as i32;
                let tex_height = video_info.height() as i32;

                // Clone necessary objects for the closure
                let out_tex = out_tex.clone();
                let shader_guard = self.shader.lock().unwrap().clone();

                // Use gst_gl_context_thread_add to run GL operations in the proper thread
                // This follows the same pattern as the C implementation
                context.thread_add(move |_context| {
                    gst::info!(
                        gst::CAT_DEFAULT,
                        obj = mixer,
                        "GL thread: Processing {} input textures with per-pad alpha values",
                        input_textures.len()
                    );

                    // Get the shader - it should be available since gl_start was called
                    let shader = match shader_guard.as_ref() {
                        Some(s) => s,
                        None => {
                            gst::error!(
                                gst::CAT_DEFAULT,
                                obj = mixer,
                                "No shader available in GL thread"
                            );
                            return;
                        }
                    };

                    // Get mutable access to GLMemory for framebuffer rendering
                    // This is safe because we're in the GL thread context and own this reference
                    let out_tex_ref =
                        unsafe { &mut *(out_tex.as_ptr() as *mut gst_gl::GLMemoryRef) };

                    // Use framebuffer to render with our shader
                    framebuffer.draw_to_texture(out_tex_ref, || {
                        gst::debug!(
                            gst::CAT_DEFAULT,
                            obj = mixer,
                            "Rendering with shader to texture ID {}",
                            out_tex.texture_id()
                        );

                        // Bind and use our shader
                        shader.use_();

                        // Set up shader uniforms and per-pad alpha values
                        if let Some(&(_tex1_id, alpha1)) = input_textures.get(0) {
                            shader.set_uniform_1i("tex1", 0); // Use texture unit 0
                            shader.set_uniform_1f("alpha1", alpha1);
                        }

                        if let Some(&(_tex2_id, alpha2)) = input_textures.get(1) {
                            shader.set_uniform_1i("tex2", 1); // Use texture unit 1
                            shader.set_uniform_1f("alpha2", alpha2);
                        } else if input_textures.len() == 1 {
                            // Only one input - set second texture to same as first with zero alpha
                            shader.set_uniform_1i("tex2", 0);
                            shader.set_uniform_1f("alpha2", 0.0);
                        }

                        // Set up transformation matrix like GStreamer does
                        // Identity matrix for fullscreen (no scaling/translation)
                        #[rustfmt::skip]
                        let identity_matrix: [f32; 16] = [
                            1.0, 0.0, 0.0, 0.0,
                            0.0, 1.0, 0.0, 0.0,
                            0.0, 0.0, 1.0, 0.0,
                            0.0, 0.0, 0.0, 1.0,
                        ];
                        shader.set_uniform_matrix_4fv(
                            "u_transformation",
                            1,
                            false,
                            &identity_matrix,
                        );

                        // Bind input textures to OpenGL texture units using gl crate
                        unsafe {
                            // Bind first input texture
                            if let Some(&(tex1_id, _alpha1)) = input_textures.get(0) {
                                gl::ActiveTexture(gl::TEXTURE0);
                                gl::BindTexture(gl::TEXTURE_2D, tex1_id);
                            }

                            // Bind second input texture
                            if let Some(&(tex2_id, _alpha2)) = input_textures.get(1) {
                                gl::ActiveTexture(gl::TEXTURE1);
                                gl::BindTexture(gl::TEXTURE_2D, tex2_id);
                            } else if input_textures.len() == 1 {
                                // Use first texture for both samplers if only one available
                                gl::ActiveTexture(gl::TEXTURE1);
                                gl::BindTexture(gl::TEXTURE_2D, input_textures[0].0);
                            }
                        }

                        // Now render the actual texture blending using a fullscreen quad
                        unsafe {
                            // Set viewport to match output texture size
                            gl::Viewport(0, 0, tex_width, tex_height);

                            // Clear to transparent black
                            gl::ClearColor(0.0, 0.0, 0.0, 0.0);
                            gl::Clear(gl::COLOR_BUFFER_BIT);

                            // Enable blending for alpha compositing
                            gl::Enable(gl::BLEND);
                            gl::BlendFunc(gl::SRC_ALPHA, gl::ONE_MINUS_SRC_ALPHA);

                            // Set up basic rendering state
                            gl::Disable(gl::DEPTH_TEST);
                            gl::Disable(gl::CULL_FACE);

                            // Use the same vertex layout as GStreamer's glvideomixer
                            // Position (x,y,z) + TexCoord (u,v) = 5 floats per vertex
                            let vertices: [f32; 20] = [
                                // x,    y,    z,    u,    v
                                -1.0, -1.0, 0.0, 0.0, 0.0, // Bottom-left
                                1.0, -1.0, 0.0, 1.0, 0.0, // Bottom-right
                                1.0, 1.0, 0.0, 1.0, 1.0, // Top-right
                                -1.0, 1.0, 0.0, 0.0, 1.0, // Top-left
                            ];

                            let indices: [u32; 6] = [
                                0, 1, 2, // First triangle
                                2, 3, 0, // Second triangle
                            ];

                            // Generate and bind vertex array and buffers
                            let mut vao = 0;
                            let mut vbo = 0;
                            let mut ebo = 0;

                            gl::GenVertexArrays(1, &mut vao);
                            gl::GenBuffers(1, &mut vbo);
                            gl::GenBuffers(1, &mut ebo);

                            gl::BindVertexArray(vao);

                            // Upload vertex data
                            gl::BindBuffer(gl::ARRAY_BUFFER, vbo);
                            gl::BufferData(
                                gl::ARRAY_BUFFER,
                                (vertices.len() * std::mem::size_of::<f32>()) as isize,
                                vertices.as_ptr() as *const std::ffi::c_void,
                                gl::STATIC_DRAW,
                            );

                            // Upload index data
                            gl::BindBuffer(gl::ELEMENT_ARRAY_BUFFER, ebo);
                            gl::BufferData(
                                gl::ELEMENT_ARRAY_BUFFER,
                                (indices.len() * std::mem::size_of::<u32>()) as isize,
                                indices.as_ptr() as *const std::ffi::c_void,
                                gl::STATIC_DRAW,
                            );

                            // Set up vertex attributes to match our custom vertex shader
                            // Get attribute locations from the shader
                            let position_loc = shader.attribute_location("a_position");
                            let texcoord_loc = shader.attribute_location("a_texcoord");

                            // Position attribute - 3 floats (x,y,z)
                            if position_loc >= 0 {
                                gl::VertexAttribPointer(
                                    position_loc as u32,
                                    3,
                                    gl::FLOAT,
                                    gl::FALSE,
                                    (5 * std::mem::size_of::<f32>()) as i32,
                                    std::ptr::null(),
                                );
                                gl::EnableVertexAttribArray(position_loc as u32);
                            }

                            // Texture coordinate attribute - 2 floats (u,v)
                            if texcoord_loc >= 0 {
                                gl::VertexAttribPointer(
                                    texcoord_loc as u32,
                                    2,
                                    gl::FLOAT,
                                    gl::FALSE,
                                    (5 * std::mem::size_of::<f32>()) as i32,
                                    (3 * std::mem::size_of::<f32>()) as *const std::ffi::c_void,
                                );
                                gl::EnableVertexAttribArray(texcoord_loc as u32);
                            }

                            // Draw the quad
                            gl::DrawElements(gl::TRIANGLES, 6, gl::UNSIGNED_INT, std::ptr::null());

                            // Cleanup
                            gl::BindVertexArray(0);
                            gl::DeleteVertexArrays(1, &vao);
                            gl::DeleteBuffers(1, &vbo);
                            gl::DeleteBuffers(1, &ebo);
                        }

                        gst::debug!(
                            gst::CAT_DEFAULT,
                            obj = mixer,
                            "Configured shader uniforms: alpha1={}, alpha2={}",
                            input_textures
                                .get(0)
                                .map(|(_, alpha)| *alpha)
                                .unwrap_or(0.0),
                            input_textures
                                .get(1)
                                .map(|(_, alpha)| *alpha)
                                .unwrap_or(0.0)
                        );

                        gst::debug!(gst::CAT_DEFAULT, obj = mixer, "Completed shader rendering");
                    });

                    gst::info!(
                        gst::CAT_DEFAULT,
                        obj = mixer,
                        "GL thread: Successfully rendered {} textures with alpha blending",
                        input_textures.len()
                    );
                });

                gst::info!(
                    gst::CAT_DEFAULT,
                    imp = self,
                    "Queued shader-based blending of {} input textures",
                    num_textures
                );

                Ok(())
            }
        }

        // Implementation of gst::ChildProxy virtual methods.
        // This allows accessing the pads and their properties from gst-launch.
        impl ChildProxyImpl for SimpleGLMixer {
            fn children_count(&self) -> u32 {
                self.obj().num_sink_pads() as u32
            }

            fn child_by_name(&self, name: &str) -> Option<glib::Object> {
                let object = self.obj();
                object
                    .sink_pads()
                    .into_iter()
                    .find(|p| p.name() == name)
                    .map(|p| p.upcast())
            }

            fn child_by_index(&self, index: u32) -> Option<glib::Object> {
                self.obj()
                    .sink_pads()
                    .into_iter()
                    .nth(index as usize)
                    .map(|pad| pad.upcast())
            }
        }
    }

    // The public Rust wrapper type for our element
    glib::wrapper! {
        pub struct SimpleGLMixer(ObjectSubclass<imp::SimpleGLMixer>) @extends gst_gl::GLMixer, gst_gl::GLBaseMixer, gst_video::VideoAggregator, gst_base::Aggregator, gst::Element, gst::Object, @implements gst::ChildProxy;
    }

    // Registers the type with the GObject type system.
    pub fn register(plugin: &gst::Plugin) -> Result<(), glib::BoolError> {
        gst::Element::register(
            Some(plugin),
            "simpleglmixer",
            gst::Rank::NONE,
            SimpleGLMixer::static_type(),
        )
    }
}

fn create_pipeline() -> Result<gst::Pipeline, Error> {
    let pipeline_str = "gltestsrc pattern=smpte ! simpleglmixer name=mixer sink_0::alpha=0.5 sink_1::alpha=1.0 ! glimagesink gltestsrc pattern=mandelbrot ! mixer.";

    let pipeline = gst::parse::launch(pipeline_str)?
        .downcast::<gst::Pipeline>()
        .expect("Expected a pipeline");

    pipeline.debug_to_dot_file_with_ts(gst::DebugGraphDetails::all(), "linked");

    Ok(pipeline)
}

fn example_main() -> Result<(), Error> {
    gst::init()?;

    // Register our custom mixer element
    gst::Element::register(
        None,
        "simpleglmixer",
        gst::Rank::NONE,
        gl_mixer::SimpleGLMixer::static_type(),
    )?;

    let main_loop = glib::MainLoop::new(None, false);
    let pipeline = create_pipeline()?;

    let bus = pipeline.bus().unwrap();
    let _bus_watch_guard = bus.add_watch({
        let main_loop = main_loop.clone();
        move |_, msg| {
            use gst::MessageView;

            match msg.view() {
                MessageView::Eos(..) => main_loop.quit(),
                MessageView::Error(err) => {
                    println!(
                        "Error from {:?}: {} ({:?})",
                        err.src().map(|s| s.path_string()),
                        err.error(),
                        err.debug()
                    );
                    main_loop.quit();
                }
                _ => (),
            }
            glib::ControlFlow::Continue
        }
    })?;

    pipeline.set_state(gst::State::Playing)?;

    main_loop.run();
    pipeline.debug_to_dot_file_with_ts(gst::DebugGraphDetails::all(), "done");

    pipeline.set_state(gst::State::Null)?;

    Ok(())
}

fn main() -> Result<(), Error> {
    examples_common::run(example_main)
}
