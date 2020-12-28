use std::{
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use druid::{
    self,
    piet::{ImageFormat, InterpolationMode},
    widget::{Image, Label, LabelText},
    Affine, Color, Env, Event, FileInfo, ImageBuf, LifeCycle, RenderContext,
    Selector, Size,
};

use druid::{Data, Lens, Widget, WidgetExt};
use image::imageops::thumbnail;

use crate::{AppState, Thumbnail};
#[derive(Clone, Data, Lens)]
pub struct DisplayImage {
    pub image: Rc<Image>,
}

impl Widget<AppState> for DisplayImage {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut AppState,
        env: &Env,
    ) {
        let open_selector: Selector<AppState> =
            Selector::new("druid-builtin.open-file-path");
        let select_image_selector: Selector<usize> =
            Selector::new("select_thumbnail");
        match event {
            Event::Command(open) if open.is(open_selector) => {
                dbg!("got open command in display image");
                // I don't know if this is right
                // if I don't return here, the application crashes everytime
                // I close it because of unwrap() and can't find selector
                // is the command being sent periodically?
                let payload: &FileInfo = open.get_unchecked(Selector::new(
                    "druid-builtin.open-file-path",
                ));

                let path = payload.path();
                let mut paths: Vec<PathBuf> = std::fs::read_dir(path)
                    .unwrap()
                    .map(|path| path.unwrap().path())
                    .collect();

                data.images = Arc::new(paths);
                data.current_image = 0;
                data.create_thumbnails();

                ctx.request_layout();
                ctx.request_paint();
            }
            Event::Command(select_image)
                if select_image.is(select_image_selector) =>
            {
                let index = select_image.get_unchecked(select_image_selector);
                data.current_image = *index;

                ctx.request_layout();
                ctx.request_paint();
            }
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &AppState,
        env: &Env,
    ) {
        // todo!()
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &AppState,
        data: &AppState,
        env: &Env,
    ) {
        if data.images.is_empty() {
            return;
        }
        if data.current_image != old_data.current_image
            || data.images != old_data.images
        {
            let image =
                image::io::Reader::open(&data.images[data.current_image])
                    .unwrap()
                    .decode()
                    .unwrap()
                    .into_rgb8();
            let (width, height) = image.dimensions();
            let image = ImageBuf::from_raw(
                image.into_raw(),
                ImageFormat::Rgb,
                width as usize,
                height as usize,
            );
            // dbg!(width, height);
            let image = Image::new(image)
                .interpolation_mode(InterpolationMode::Bilinear);
            self.image = Rc::new(image);
            ctx.request_layout();
            ctx.request_paint();
        }
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &AppState,
        env: &Env,
    ) -> druid::Size {
        Rc::get_mut(&mut self.image)
            .unwrap()
            .layout(ctx, bc, data, env)
    }

    fn paint(&mut self, ctx: &mut druid::PaintCtx, data: &AppState, env: &Env) {
        Rc::get_mut(&mut self.image).unwrap().paint(ctx, data, env);
    }
}

pub struct Button<T: Data> {
    text: Label<T>,
    color: druid::Color,
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
    ) -> Self {
        Self {
            text: Label::new(text)
                .with_text_color(color.clone())
                .with_text_size(30.0),
            color,
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
        data: &mut T,
        env: &Env,
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
