use std::num::NonZeroU32;

use glutin::api::egl;
use glutin::api::egl::context::PossiblyCurrentContext;
use glutin::api::egl::display::Display;
use glutin::api::egl::surface::Surface;
use glutin::config::{ConfigTemplateBuilder, GlConfig};
use glutin::display::GlDisplay;
use glutin::prelude::NotCurrentGlContext;
use glutin::surface::{SurfaceAttributesBuilder, WindowSurface};
use glutin::{
    api::egl::{config::Config, context::NotCurrentContext},
    context::{ContextApi, ContextAttributesBuilder, Version},
    display::GetGlDisplay,
};
use raw_window_handle::{RawDisplayHandle, RawWindowHandle};

pub fn create_gl_display(raw_display_handle: RawDisplayHandle) -> Display {
    let gl_display =
        unsafe { Display::new(raw_display_handle).expect("Failed to create EGL Display") };
    gl_display
}

pub fn get_gl_config(gl_display: &Display) -> egl::config::Config {
    let template = ConfigTemplateBuilder::new().with_alpha_size(8).build();

    let config = unsafe { gl_display.find_configs(template) }
        .unwrap()
        .reduce(|config, acc| {
            if config.num_samples() > acc.num_samples() {
                config
            } else {
                acc
            }
        })
        .expect("No available configs");

    config
}

// pub fn get_gl_config(
//     raw_window_handle: RawWindowHandle,
//     (width, height): (u32, u32),
//     gl_display: &Display,
// ) -> (Surface<WindowSurface>, PossiblyCurrentContext) {
//     let template = ConfigTemplateBuilder::new().with_alpha_size(8).build();

//     let config = unsafe { gl_display.find_configs(template) }
//         .unwrap()
//         .reduce(|config, acc| {
//             if config.num_samples() > acc.num_samples() {
//                 config
//             } else {
//                 acc
//             }
//         })
//         .expect("No available configs");

//     let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));

//     // Since glutin by default tries to create OpenGL core context, which may not be
//     // present we should try gles.
//     let fallback_context_attributes = ContextAttributesBuilder::new()
//         .with_context_api(ContextApi::Gles(None))
//         .build(Some(raw_window_handle));
//     let mut not_current_gl_context = Some(unsafe {
//         gl_display
//             .create_context(&config, &context_attributes)
//             .unwrap_or_else(|_| {
//                 gl_display
//                     .create_context(&config, &fallback_context_attributes)
//                     .expect("failed to create context")
//             })
//     });

//     let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
//         raw_window_handle,
//         NonZeroU32::new(width).unwrap(),
//         NonZeroU32::new(height).unwrap(),
//     );

//     let gl_surface = unsafe {
//         gl_display
//             .create_window_surface(&config, &attrs)
//             .expect("Failed to create OpenGl surface")
//     };

//     let gl_context = not_current_gl_context
//         .take()
//         .unwrap()
//         .make_current(&gl_surface)
//         .expect("Failed to make newly created OpenGL context current");

//     (gl_surface, gl_context)
// }

pub fn create_gl_context(
    raw_window_handle: RawWindowHandle,
    (width, height): (u32, u32),
    gl_config: &Config,
) -> (PossiblyCurrentContext, Surface<WindowSurface>) {
    // The context creation part.
    let context_attributes = ContextAttributesBuilder::new().build(Some(raw_window_handle));

    // Since glutin by default tries to create OpenGL core context, which may not be
    // present we should try gles.
    let fallback_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::Gles(None))
        .build(Some(raw_window_handle));

    // There are also some old devices that support neither modern OpenGL nor GLES.
    // To support these we can try and create a 2.1 context.
    let legacy_context_attributes = ContextAttributesBuilder::new()
        .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 1))))
        .build(Some(raw_window_handle));

    // Reuse the uncurrented context from a suspended() call if it exists, otherwise
    // this is the first time resumed() is called, where the context still
    // has to be created.
    let gl_display = gl_config.display();

    let not_current_gl_context = unsafe {
        gl_display
            .create_context(gl_config, &context_attributes)
            .unwrap_or_else(|_| {
                gl_display
                    .create_context(gl_config, &fallback_context_attributes)
                    .unwrap_or_else(|_| {
                        gl_display
                            .create_context(gl_config, &legacy_context_attributes)
                            .expect("failed to create context")
                    })
            })
    };

    let attrs = SurfaceAttributesBuilder::<WindowSurface>::new().build(
        raw_window_handle,
        NonZeroU32::new(width).unwrap(),
        NonZeroU32::new(height).unwrap(),
    );

    let gl_surface = unsafe {
        gl_display
            .create_window_surface(&gl_config, &attrs)
            .expect("Failed to create OpenGl surface")
    };

    let gl_context = not_current_gl_context
        .make_current(&gl_surface)
        .expect("Failed to make newly created OpenGL context current");

    (gl_context, gl_surface)
}
