use std::sync::mpsc::sync_channel;
use std::sync::mpsc::Receiver;
use std::sync::mpsc::SyncSender;
use std::{path::PathBuf, sync::Arc};

use druid::{
    self,
    im::Vector,
    piet::ImageFormat,
    widget::{Image, Label, LabelText},
    Affine, Color, Env, Event, ExtEventSink, ImageBuf, LifeCycle,
    RenderContext, Selector, Size, Target, WidgetId,
};

use druid::{Data, Widget};
use image::{imageops::thumbnail, RgbImage};

use crate::{
    data::{FolderGalleryState, Thumbnail},
    AppState,
};

pub const OPEN_SELECTOR: Selector<AppState> =
    Selector::new("druid-builtin.open-file-path");
pub const SELECT_IMAGE_SELECTOR: Selector<usize> =
    Selector::new("select_thumbnail");
pub const FINISHED_READING_FOLDER: Selector<(
    Arc<Vec<PathBuf>>,
    Vector<Thumbnail>,
)> = Selector::new("finish_reading_folder");
pub const FINISHED_READING_IMAGE: Selector<()> =
    Selector::new("finished_reading_image");
pub const CREATED_THUMBNAIL: Selector<Thumbnail> =
    Selector::new("created_thumbnail");

pub fn create_thumbnails(paths: Vec<PathBuf>) -> Arc<Vec<Thumbnail>> {
    let mut new_images = Vec::new();
    for path in paths.iter() {
        let image = image::io::Reader::open(path)
            .unwrap()
            .decode()
            .unwrap()
            .into_rgb8();
        let (width, height) = image.dimensions();
        // dbg!(width, height);
        let (new_width, new_height) = {
            let max_height = 150.0;
            let scale = max_height / height as f64;
            let scaled_width = width as f64 * scale;
            let scaled_height = height as f64 * scale;
            (scaled_width.trunc() as u32, scaled_height.trunc() as u32)
        };
        let image = thumbnail(&image, new_width, new_height);
        let (width, height) = image.dimensions();
        let image = ImageBuf::from_raw(
            image.into_raw(),
            ImageFormat::Rgb,
            width as usize,
            height as usize,
        );
        new_images.push(Thumbnail {
            index: new_images.len(),
            image,
        });
    }
    Arc::new(new_images)
}

pub fn read_images(sink: ExtEventSink, path: PathBuf) {
    std::thread::spawn(move || {
        // let paths: Vec<PathBuf> = std::fs::read_dir(path)
        //     .unwrap()
        //     .map(|path| path.unwrap().path())
        //     .collect();
        // TODO all of this assumes the folder has only images that won't fail decoding
        // I will have to figure out which files are images and only send those back to the
        // app state
        // I could send the image and its path together when building the thumbnails
        let mut paths = Vec::new();
        let mut thumbnails = Vector::new();
        for (i, path) in std::fs::read_dir(path).unwrap().enumerate() {
            let path = path.unwrap().path();
            paths.push(path);
            thumbnails.push_back(Thumbnail {
                index: i,
                image: ImageBuf::empty(),
            })
        }

        // let thumbnails = Arc::new(Vec::with_capacity(paths.len()));
        sink.submit_command(
            FINISHED_READING_FOLDER,
            (Arc::new(paths.clone()), thumbnails),
            // paths,
            Target::Auto,
        )
        .unwrap();
        // let thumbnails = create_thumbnails(paths.clone());
        // this might have paths that are not images so I might need to change this in the future
        for (index, path) in paths.iter().enumerate() {
            let thumbnail = {
                let image = image::io::Reader::open(path)
                    .unwrap()
                    .decode()
                    .unwrap()
                    .into_rgb8();

                let (width, height) = image.dimensions();
                let (new_width, new_height) = {
                    let max_height = 150.0;
                    let scale = max_height / height as f64;
                    let scaled_width = width as f64 * scale;
                    let scaled_height = height as f64 * scale;
                    (scaled_width.trunc() as u32, scaled_height.trunc() as u32)
                };
                let image = thumbnail(&image, new_width, new_height);

                let (width, height) = image.dimensions();
                let image = ImageBuf::from_raw(
                    image.into_raw(),
                    ImageFormat::Rgb,
                    width as usize,
                    height as usize,
                );
                Thumbnail { index, image }
            };
            sink.submit_command(
                CREATED_THUMBNAIL,
                thumbnail,
                // paths,
                Target::Auto,
            )
            .unwrap();
        }
    });
}
pub struct DisplayImage {
    pub image: Arc<Image>,
    sender: SyncSender<RgbImage>,
    receiver: Receiver<RgbImage>,
}

impl DisplayImage {
    pub fn new(image: Image) -> Self {
        let image = Arc::new(image);
        let (sender, receiver) = sync_channel(3);

        DisplayImage {
            image,
            sender,
            receiver,
        }
    }
}

impl DisplayImage {
    fn read_image(
        &self,
        sink: ExtEventSink,
        path: PathBuf,
        widget_id: WidgetId,
    ) {
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            let image = image::io::Reader::open(path)
                .unwrap()
                .decode()
                .unwrap()
                .into_rgb8();
            sender.send(image).unwrap();
            sink.submit_command(FINISHED_READING_IMAGE, (), widget_id)
                .unwrap();
        });
    }
}

impl Widget<FolderGalleryState> for DisplayImage {
    fn event(
        &mut self,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        _data: &mut FolderGalleryState,
        _env: &Env,
    ) {
        match event {
            Event::Command(image_selector)
                if image_selector.is(FINISHED_READING_IMAGE) =>
            {
                let image = self.receiver.recv().unwrap();
                let (width, height) = image.dimensions();
                let image = ImageBuf::from_raw(
                    image.into_raw(),
                    ImageFormat::Rgb,
                    width as usize,
                    height as usize,
                );
                self.image = Arc::new(Image::new(image));
                ctx.request_layout();
                ctx.request_paint();
            }
            _ => (),
        }
    }

    fn lifecycle(
        &mut self,
        _ctx: &mut druid::LifeCycleCtx,
        _event: &druid::LifeCycle,
        _data: &FolderGalleryState,
        _env: &Env,
    ) {
    }

    fn update(
        &mut self,
        ctx: &mut druid::UpdateCtx,
        old_data: &FolderGalleryState,
        data: &FolderGalleryState,
        _env: &Env,
    ) {
        if data.paths.is_empty() {
            return;
        }
        if data.selected_image != old_data.selected_image
            || data.paths != old_data.paths
        {
            // let image =
            //     image::io::Reader::open(&data.images[data.current_image_idx])
            //         .unwrap()
            //         .decode()
            //         .unwrap()
            //         .into_rgb8();
            // let (width, height) = image.dimensions();
            // let image = ImageBuf::from_raw(
            //     image.into_raw(),
            //     ImageFormat::Rgb,
            //     width as usize,
            //     height as usize,
            // );
            // let image = Image::new(image)
            //     .interpolation_mode(InterpolationMode::Bilinear);
            // self.image = Arc::new(image);
            let path = data.paths[data.selected_image].as_ref().clone();
            let sink = ctx.get_external_handle();
            // only need to send this payload back to itself
            // after it finishes reading the image on a separate thread
            // only DisplayImage needs to see this payload
            self.read_image(sink, path, ctx.widget_id());
            ctx.request_layout();
            ctx.request_paint();
        }
    }

    fn layout(
        &mut self,
        ctx: &mut druid::LayoutCtx,
        bc: &druid::BoxConstraints,
        data: &FolderGalleryState,
        env: &Env,
    ) -> druid::Size {
        Arc::get_mut(&mut self.image)
            .unwrap()
            .layout(ctx, bc, data, env)
    }

    fn paint(
        &mut self,
        ctx: &mut druid::PaintCtx,
        data: &FolderGalleryState,
        env: &Env,
    ) {
        Arc::get_mut(&mut self.image).unwrap().paint(ctx, data, env);
    }
}

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
    ) -> Self {
        Self {
            text: Label::new(text).with_text_color(color).with_text_size(30.0),
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
