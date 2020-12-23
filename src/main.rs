#![allow(warnings)]
use std::{
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use druid::{
    commands,
    piet::{ImageFormat, InterpolationMode},
    text::EditableText,
    text::TextLayout,
    widget::{
        Align, Button, Container, Controller, CrossAxisAlignment, FillStrat,
        Flex, FlexParams, Image, Label, LensWrap, List, ListIter,
        MainAxisAlignment, Padding, SizedBox, TextBox,
    },
    AppLauncher, Color, Command, Data, Env, FileDialogOptions, ImageBuf, Lens,
    LifeCycle, LocalizedString, MenuDesc, MenuItem, Selector, Target, Widget,
    WidgetExt, WindowDesc,
};
use image::{imageops::thumbnail, RgbImage};

pub mod widget;
use crate::widget::*;
#[derive(Clone, Data, Lens)]
pub struct AppState {
    images: Arc<Vec<PathBuf>>,
    current_image: usize,
    thumbnails: Arc<Vec<Thumbnail>>,
}

// #[derive(Clone, Data, Lens, Debug)]
#[derive(Clone, Data, Lens)]
pub struct Thumbnail {
    // image: Rc<RgbImage>,
    image: Rc<ImageBuf>,
}

impl ListIter<Thumbnail> for AppState {
    fn for_each(&self, mut cb: impl FnMut(&Thumbnail, usize)) {
        for (i, item) in self.thumbnails.iter().enumerate() {
            cb(item, i);
        }
    }

    fn for_each_mut(&mut self, mut cb: impl FnMut(&mut Thumbnail, usize)) {
        // TODO
    }

    fn data_len(&self) -> usize {
        self.thumbnails.len()
    }
}

struct ThumbnailController;

impl Controller<Thumbnail, Image> for ThumbnailController {
    fn event(
        &mut self,
        child: &mut Image,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut Thumbnail,
        env: &Env,
    ) {
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut Image,
        ctx: &mut druid::LifeCycleCtx,
        event: &LifeCycle,
        data: &Thumbnail,
        env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {
                child.image_data = ImageBuf {
                    format: data.format.clone(),
                    width: data.width.clone(),
                    height: data.height.clone(),
                    pixels: data.pixels.clone(),
                }
            }
            _ => {}
        }
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut Image,
        ctx: &mut druid::UpdateCtx,
        old_data: &Thumbnail,
        data: &Thumbnail,
        env: &Env,
    ) {
        child.update(ctx, old_data, data, env)
    }
}

impl AppState {
    pub fn create_thumbnails(&mut self) {
        let mut new_images = Vec::new();
        for path in self.images.iter() {
            let image = image::io::Reader::open(path)
                .unwrap()
                .decode()
                .unwrap()
                .into_rgb8();
            let (width, height) = image.dimensions();
            // dbg!(width, height);
            let (new_width, new_height) = {
                let max_height = 150.0;
                let scale = max_height / image.height() as f64;
                let scaled_width = image.width() as f64 * scale;
                let scaled_height = image.height() as f64 * scale;
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
            // dbg!(width, height);
            // let image = Image::new(image)
            //     .interpolation_mode(InterpolationMode::Bilinear);
            new_images.push(Thumbnail {
                image: Rc::new(image),
            });
        }
        self.thumbnails = Arc::new(new_images);
    }
}

const IMAGE_FOLDER: &str = "./images";

fn main() {
    let root = move || {
        let (button_width, button_height) = (50.0, 1000.0);
        let font_color = Color::rgb8(0, 0, 0);
        let bg_color = Color::rgb8(0xff, 0xff, 0xff);
        let hover_color = Color::rgb8(0xcc, 0xcc, 0xcc);
        let active_color = Color::rgb8(0x90, 0x90, 0x90);

        let left_button = crate::widget::Button::new(
            "❮",
            font_color.clone(),
            bg_color.clone(),
            hover_color.clone(),
            active_color.clone(),
        )
        .on_click(|_ctx, data: &mut AppState, _env| {
            if data.images.is_empty() {
                return;
            }
            if data.current_image == 0 {
                data.current_image = data.images.len() - 1;
            } else {
                data.current_image -= 1;
            }
            // dbg!(data.current_image);
            // dbg!(&data.images[data.current_image]);
        })
        .fix_size(button_width, button_height);

        let right_button = crate::widget::Button::new(
            "❯",
            font_color,
            bg_color,
            hover_color,
            active_color,
        )
        .on_click(|_ctx, data: &mut AppState, _env| {
            if data.images.is_empty() {
                return;
            }
            if data.current_image == data.images.len() - 1 {
                data.current_image = 0
            } else {
                data.current_image += 1;
            }
            // dbg!(data.current_image);
            // dbg!(&data.images[data.current_image]);
        })
        .fix_size(button_width, button_height);

        // let image = ImageBuf::from_raw(
        //     raw_pixels.to_vec(),
        //     ImageFormat::Rgb,
        //     width as usize,
        //     height as usize,
        // );
        let image = ImageBuf::empty();
        let image = Image::new(image)
            .interpolation_mode(InterpolationMode::Bilinear)
            .fill_mode(FillStrat::Contain);
        let image = DisplayImage {
            image: Rc::new(image),
        };
        let image_view = Flex::row()
            .must_fill_main_axis(true)
            .with_child(left_button)
            .with_flex_child(image, FlexParams::new(1.0, None))
            .with_child(right_button)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .main_axis_alignment(MainAxisAlignment::SpaceBetween);

        let film_strip_list =
            // List::new(|| Image::new(ImageBuf::empty()).lens(Thumbnail::image))
            List::new(|| {
                dbg!("adding a widget");
                let controller = ThumbnailController{};
                // Image::new(ImageBuf::empty()).lens(Thumbnail::image)
                Image::new(ImageBuf::empty()).controller(controller).lens(Thumbnail::image)
            })
                .horizontal()
                .with_spacing(10.0)
                .lens(AppState::thumbnails);

        let film_strip_view = Flex::row()
            .must_fill_main_axis(true)
            .with_child(film_strip_list)
            .fix_height(150.0)
            .background(Color::rgb8(0x99, 0x99, 0x99));
        let layout = Flex::column()
            .must_fill_main_axis(true)
            .with_flex_child(image_view, FlexParams::new(1.0, None))
            .with_child(film_strip_view);
        Container::new(layout).background(druid::Color::rgb8(255, 255, 255))
    };

    let menu = {
        let menu = druid::MenuDesc::empty();
        let open = {
            let open_folder_options = FileDialogOptions::new()
                .select_directories()
                .default_name("Open");
            MenuItem::new(
                LocalizedString::new("Open Folder"),
                commands::SHOW_OPEN_PANEL.with(open_folder_options),
            )
        };
        menu.append(open).append_separator()
    };
    let window = WindowDesc::new(root).menu(menu).title("Gallery");

    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(AppState {
            images: Arc::new(Vec::new()),
            current_image: 0,
            thumbnails: Arc::new(Vec::new()),
        })
        .unwrap();
}
