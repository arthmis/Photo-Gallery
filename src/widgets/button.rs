use druid::{
    self,
    widget::{Label, LabelText},
    Affine, Color, Env, Event, LifeCycle, RenderContext, Size,
};

use druid::{Data, Widget};

pub struct Button<T: Data> {
    text: Label<T>,
    // color: druid::Color,
    background_color: druid::Color,
    hover_color: druid::Color,
    active_color: druid::Color,
    text_size: Size,
}

impl<T: Data> Button<T> {
    pub fn new(
        text: impl Into<LabelText<T>>,
        color: Color,
        background_color: Color,
        hover_color: Color,
        active_color: Color,
        font_size: f64,
    ) -> Self {
        Self {
            text: Label::new(text)
                .with_text_color(color)
                .with_text_size(font_size),
            // color,
            background_color,
            hover_color,
            active_color,
            text_size: Size::ZERO,
        }
    }
}

impl<T: Data> Widget<T> for Button<T> {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        _data: &mut T,
        _env: &Env,
    ) {
        match event {
            Event::MouseDown(_) => {
                ctx.set_active(true);
                ctx.request_paint();
            }
            Event::MouseUp(_) => {
                if ctx.is_active() {
                    ctx.set_active(false);
                    ctx.request_paint();
                }
            }
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &T,
        env: &Env,
    ) {
        if let LifeCycle::HotChanged(_) = event {
            ctx.request_paint();
        }
        self.text.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &T,
        data: &T,
        env: &Env,
    ) {
        self.text.update(ctx, old_data, data, env);
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &T,
        env: &Env,
    ) -> druid::Size {
        let (padding_width, padding_height) = (20.0, 20.0);
        let padding = Size::new(padding_width, padding_height);
        let label_bc = bc.shrink(padding).loosen();
        self.text_size = self.text.layout(ctx, &label_bc, data, env);
        // HACK: to make sure we look okay at default sizes when beside a textbox,
        // we make sure we will have at least the same height as the default textbox.
        let min_height = 70.0;
        let baseline = self.text.baseline_offset();
        ctx.set_baseline_offset(baseline + padding_height);

        bc.constrain(Size::new(
            self.text_size.width + padding_width,
            (self.text_size.height + padding_height).max(min_height),
        ))
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &T, env: &Env) {
        let is_active = ctx.is_active();
        let is_hot = ctx.is_hot();
        let size = ctx.size();

        let stroke_width = 0.0;
        let rect = size.to_rect();

        let (border_color, bg_color) = if is_hot {
            if is_active {
                (self.active_color.clone(), self.active_color.clone())
            } else {
                (self.hover_color.clone(), self.hover_color.clone())
            }
        } else {
            (self.background_color.clone(), self.background_color.clone())
        };

        // paint border
        ctx.stroke(rect, &border_color, stroke_width);
        ctx.fill(rect, &bg_color);

        let label_offset = (size.to_vec2() - self.text_size.to_vec2()) / 2.0;
        ctx.with_save(|ctx| {
            ctx.transform(Affine::translate(label_offset));
            self.text.paint(ctx, data, env);
        });
    }
}
// pub struct SvgButton<T> {
//     text: Svg,
//     // color: druid::Color,
//     background_color: druid::Color,
//     hover_color: druid::Color,
//     active_color: druid::Color,
//     text_size: Size,
//     phantom_data: PhantomData<T>,
// }

// impl<T: Data> SvgButton<T> {
//     pub fn new(
//         text: Svg,
//         color: Color,
//         background_color: Color,
//         hover_color: Color,
//         active_color: Color,
//     ) -> Self {
//         Self {
//             text,
//             // color,
//             background_color,
//             hover_color,
//             active_color,
//             text_size: Size::ZERO,
//             phantom_data: PhantomData,
//         }
//     }
// }

// impl<T: Data> Widget<T> for SvgButton<T> {
//     fn event(
//         &mut self,
//         ctx: &mut druid::EventCtx,
//         event: &druid::Event,
//         _data: &mut T,
//         _env: &Env,
//     ) {
//         match event {
//             Event::MouseDown(_) => {
//                 ctx.set_active(true);
//                 ctx.request_paint();
//             }
//             Event::MouseUp(_) => {
//                 if ctx.is_active() {
//                     ctx.set_active(false);
//                     ctx.request_paint();
//                 }
//             }
//             _ => (),
//         }
//     }

//     fn lifecycle(
//         &mut self,
//         ctx: &mut druid::LifeCycleCtx,
//         event: &druid::LifeCycle,
//         data: &T,
//         env: &Env,
//     ) {
//         if let LifeCycle::HotChanged(_) = event {
//             ctx.request_paint();
//         }
//         self.text.lifecycle(ctx, event, data, env)
//     }

//     fn update(
//         &mut self,
//         ctx: &mut druid::UpdateCtx,
//         old_data: &T,
//         data: &T,
//         env: &Env,
//     ) {
//         self.text.update(ctx, old_data, data, env);
//     }

//     fn layout(
//         &mut self,
//         ctx: &mut druid::LayoutCtx,
//         bc: &druid::BoxConstraints,
//         data: &T,
//         env: &Env,
//     ) -> druid::Size {
//         let (padding_width, padding_height) = (20.0, 20.0);
//         // let padding = Size::new(padding_width, padding_height);
//         // let label_bc = bc.shrink(padding).loosen();
//         self.text_size = self.text.layout(ctx, &bc, data, env);
//         // dbg!(&self.text_size);
//         // HACK: to make sure we look okay at default sizes when beside a textbox,
//         // we make sure we will have at least the same height as the default textbox.
//         let min_height = 70.0;
//         // let baseline = self.text.baseline_offset();
//         // ctx.set_baseline_offset(baseline + padding_height);

//         bc.constrain(Size::new(
//             self.text_size.width + padding_width,
//             (self.text_size.height + padding_height).max(min_height),
//         ))
//     }

//     fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &T, env: &Env) {
//         let is_active = ctx.is_active();
//         let is_hot = ctx.is_hot();
//         let size = ctx.size();

//         let stroke_width = 0.0;
//         let rect = size.to_rect();

//         let (border_color, bg_color) = if is_hot {
//             if is_active {
//                 (self.active_color.clone(), self.active_color.clone())
//             } else {
//                 (self.hover_color.clone(), self.hover_color.clone())
//             }
//         } else {
//             (self.background_color.clone(), self.background_color.clone())
//         };

//         // paint border
//         ctx.stroke(rect, &border_color, stroke_width);
//         ctx.fill(rect, &bg_color);

//         let label_offset = (size.to_vec2() - self.text_size.to_vec2()) / 2.0;
//         ctx.with_save(|ctx| {
//             ctx.transform(Affine::translate(label_offset));
//             self.text.paint(ctx, data, env);
//         });
//     }
// }
