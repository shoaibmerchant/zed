use glow::HasContext;
use glutin::{
    api::egl::{self, context::PossiblyCurrentContext},
    surface::{GlSurface, WindowSurface},
};
use impellers::{Color, DisplayListBuilder, ISize, Paint, PixelFormat, Point, Rect, Size};
use std::sync::Arc;

use crate::{
    Background, Hsla, PrimitiveBatch, hsla,
    platform::blade::{BladeAtlas, BladeContext, GPUIRenderer},
};

pub struct ImpellerConfig {
    pub height: u32,
    pub width: u32,
}

pub struct ImpellerRenderer {
    config: ImpellerConfig,
    blade_gpu: Arc<blade_graphics::Context>,
    impeller_context: impellers::Context,
    glow_context: glow::Context,
    atlas: Arc<BladeAtlas>,
    gl_context: PossiblyCurrentContext,
    gl_surface: egl::surface::Surface<WindowSurface>,
}

impl ImpellerRenderer {
    pub fn new(
        config: ImpellerConfig,
        blade_context: &BladeContext,
        impeller_context: impellers::Context,
        glow_context: glow::Context,
        gl_context: PossiblyCurrentContext,
        gl_surface: egl::surface::Surface<WindowSurface>,
    ) -> anyhow::Result<Self> {
        let atlas = Arc::new(BladeAtlas::new(&blade_context.gpu));

        Ok(Self {
            config,
            atlas,
            blade_gpu: Arc::clone(&blade_context.gpu),
            impeller_context,
            glow_context,
            gl_context,
            gl_surface,
        })
    }
}

impl GPUIRenderer for ImpellerRenderer {
    fn update_drawable_size(&mut self, size: crate::Size<crate::DevicePixels>) {
        println!("ImpellerRenderer::update_drawable_size()");
        todo!()
    }

    fn update_drawable_size_even_if_unchanged(&mut self, size: crate::Size<crate::DevicePixels>) {
        println!("ImpellerRenderer::update_drawable_size_even_if_unchanged()");
        todo!()
    }

    fn update_transparency(&mut self, transparent: bool) {
        println!("ImpellerRenderer::update_transparency()");
    }

    fn viewport_size(&self) -> blade_graphics::Extent {
        println!("ImpellerRenderer::viewport_size()");
        todo!()
    }

    fn sprite_atlas(&self) -> &std::sync::Arc<super::BladeAtlas> {
        &self.atlas
    }

    fn gpu_specs(&self) -> crate::GpuSpecs {
        println!("ImpellerRenderer::gpu_specs()");
        todo!()
    }

    fn destroy(&mut self) {
        println!("ImpellerRenderer::destroy()");
    }

    fn draw(&mut self, scene: &crate::Scene) {
        println!("ImpellerRenderer::draw()");
        let impeller_context = &mut self.impeller_context;
        let glow_context = &self.glow_context;
        let gl_context = &self.gl_context;
        let gl_surface = &self.gl_surface;

        let ImpellerConfig { height, width } = self.config;
        println!("impeller dimensions {} {}", width, height);

        // let dl = {
        //     let mut builder = DisplayListBuilder::new(None);
        //     let mut paint = Paint::default();
        //     paint.set_color(Color::BLACK);
        //     // clear with black first
        //     builder.draw_paint(&paint);
        //     paint.set_color(Color::AIR_FORCE_BLUE);
        //     builder.draw_rect(
        //         &Rect::new(Point::new(100.0, 100.0), Size::new(250.0, 250.0)),
        //         &paint,
        //     );
        //     builder.build().unwrap()
        // };

        let mut surface = unsafe {
            impeller_context.wrap_fbo(
                0,
                PixelFormat::RGBA8888,
                ISize::new(width.into(), height.into()),
            )
        }
        .expect("failed to wrap window's framebuffer");

        let mut dl_builder = DisplayListBuilder::new(None);
        let mut paint = Paint::default();
        paint.set_color(Color::BLACK);
        // clear with black first
        dl_builder.draw_paint(&paint);

        for batch in scene.batches() {
            match batch {
                PrimitiveBatch::Quads(quads) => {
                    let mut q_surface = unsafe {
                        impeller_context.wrap_fbo(
                            0,
                            PixelFormat::RGBA8888,
                            ISize::new(width.into(), height.into()),
                        )
                    }
                    .expect("failed to wrap window's framebuffer");

                    for q in quads.iter() {
                        println!(
                            "drawing q {:?} {:?} {:?}",
                            q.order, q.bounds.origin, q.bounds.size
                        );
                        let origin = q.bounds.origin;
                        let size = q.bounds.size;
                        let hsl_color = q.background.solid;

                        // Convert to RGBA
                        let rgba_color = hsl_color.to_rgb();
                        let color = Color::new_srgba(
                            rgba_color.r,
                            rgba_color.g,
                            rgba_color.b,
                            rgba_color.a,
                        );

                        paint.set_color(color);

                        dl_builder.draw_rect(
                            &Rect::new(
                                Point::new(origin.x.0, origin.y.0),
                                Size::new(size.width.0, size.height.0),
                            ),
                            &paint,
                        );
                    }
                }
                PrimitiveBatch::Paths(paths) => {}
                _ => {}
            }
        }
        let dl = dl_builder.build().unwrap();
        surface
            .draw_display_list(&dl)
            .expect("failed to draw on surface");

        // unsafe {
        //     glow_context.clear_color(1.0, 0.0, 0.0, 1.0);
        //     glow_context.clear(glow::COLOR_BUFFER_BIT);
        // }

        gl_surface
            .swap_buffers(&gl_context)
            .expect("Could not swap buffers");
    }
}
