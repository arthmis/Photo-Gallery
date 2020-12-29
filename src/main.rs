#![allow(warnings)]
use std::{
    path::{Path, PathBuf},
    rc::Rc,
    sync::Arc,
};

use druid::{
    commands, lens,
    piet::{ImageFormat, InterpolationMode},
    text::EditableText,
    text::TextLayout,
    widget::{
        Align, Button, Container, Controller, CrossAxisAlignment, FillStrat,
        Flex, FlexParams, Image, Label, LensWrap, List, ListIter,
        MainAxisAlignment, Padding, Painter, Scroll, SizedBox, TextBox,
    },
    AppDelegate, AppLauncher, Color, Command, Data, Env, Event, EventCtx,
    FileDialogOptions, FileInfo, ImageBuf, Insets, Lens, LensExt, LifeCycle,
    LocalizedString, MenuDesc, MenuItem, RenderContext, Selector, Target,
    Widget, WidgetExt, WindowDesc,
};
use image::{imageops::thumbnail, RgbImage};

pub mod widget;
use crate::widget::*;
use druid_lens_compose::ComposeLens;

#[derive(Clone, Data, Lens, Debug, ComposeLens)]
pub struct AppState {
    images: Arc<Vec<PathBuf>>,
    current_image: usize,
    thumbnails: Arc<Vec<Thumbnail>>,
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
            new_images.push(Thumbnail {
                is_selected: false,
                current_image: 0,
                index: new_images.len(),
                image: Arc::new(image),
            });
        }
        self.thumbnails = Arc::new(new_images);
    }
}
// impl<S: Data, T: Data> ListIter<(S, &T)> for (S, Arc<Vec<T>>) {}
impl ListIter<(usize, Thumbnail)> for AppState {
    fn for_each(&self, mut cb: impl FnMut(&(usize, Thumbnail), usize)) {
        for (i, item) in self.thumbnails.iter().enumerate() {
            // let mut owned_item = item.to_owned();
            // if self.current_image == i {
            //     owned_item.is_selected = true;
            // } else {
            //     owned_item.is_selected = false;
            // }
            // owned_item.current_image = self.current_image;
            cb(&(self.current_image, item.clone()), i);
            // cb(item, i);
        }
    }

    fn for_each_mut(
        &mut self,
        mut cb: impl FnMut(&mut (usize, Thumbnail), usize),
    ) {
        // let mut new_data = Vec::with_capacity(self.thumbnails.len());
        // let mut any_changed = false;
        for (i, item) in self.thumbnails.iter().enumerate() {
            let mut owned_item = item.to_owned();
            // if self.current_image == i {
            //     owned_item.is_selected = true;
            // } else {
            //     owned_item.is_selected = false;
            // }
            // owned_item.current_image = self.current_image;
            cb(&mut (self.current_image, owned_item), i);
            // if !any_changed && !item.same(&owned_item) {
            //     any_changed = true;
            // }
            // new_data.push(owned_item);
        }
        // if any_changed {
        //     self.thumbnails = Arc::new(new_data);
        // }
    }

    fn data_len(&self) -> usize {
        self.thumbnails.len()
    }
}

#[derive(Clone, Data, Lens, Debug, ComposeLens)]
pub struct Thumbnail {
    is_selected: bool,
    current_image: usize,
    index: usize,
    image: Arc<ImageBuf>,
}
struct ThumbnailController;

impl Controller<(usize, Thumbnail), Image> for ThumbnailController {
    fn event(
        &mut self,
        child: &mut Image,
        ctx: &mut druid::EventCtx,
        event: &Event,
        data: &mut (usize, Thumbnail),
        env: &Env,
    ) {
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut Image,
        ctx: &mut druid::LifeCycleCtx,
        event: &LifeCycle,
        data: &(usize, Thumbnail),
        env: &Env,
    ) {
        match event {
            LifeCycle::WidgetAdded => {
                child.set_image_data(data.1.image.as_ref().clone());
                ctx.request_layout();
                ctx.request_paint();
            }
            _ => {}
        }
        child.lifecycle(ctx, event, data, env)
    }
}

// pub struct ListController;

// impl Controller<AppState, List<Thumbnail>> for ListController {
//     fn event(
//         &mut self,
//         child: &mut List<Thumbnail>,
//         ctx: &mut druid::EventCtx,
//         event: &Event,
//         data: &mut AppState,
//         env: &Env,
//     ) {
//         // dbg!("received list event");
//         child.event(ctx, event, data, env)
//     }

//     fn lifecycle(
//         &mut self,
//         child: &mut List<Thumbnail>,
//         ctx: &mut druid::LifeCycleCtx,
//         event: &LifeCycle,
//         data: &AppState,
//         env: &Env,
//     ) {
//         // dbg!("received list lifecycle");
//         child.lifecycle(ctx, event, data, env)
//     }

//     fn update(
//         &mut self,
//         child: &mut List<Thumbnail>,
//         ctx: &mut druid::UpdateCtx,
//         old_data: &AppState,
//         data: &AppState,
//         env: &Env,
//     ) {
//         dbg!("received list update");
//         dbg!(old_data.current_image, data.current_image);
//         if old_data.current_image != data.current_image {
//             ctx.request_layout();
//             ctx.request_paint();
//         }
//         child.update(ctx, old_data, data, env)
//     }
// }

const IMAGE_FOLDER: &str = "./images - Copy";

fn main() {
    // let mut paths: Vec<PathBuf> = std::fs::read_dir(IMAGE_FOLDER)
    //     .unwrap()
    //     .map(|path| path.unwrap().path())
    //     .collect();
    // let mut thumbnails = Vec::new();
    // for path in paths.iter() {
    //     let image = image::io::Reader::open(path)
    //         .unwrap()
    //         .decode()
    //         .unwrap()
    //         .into_rgb8();
    //     let (width, height) = image.dimensions();
    //     let image = ImageBuf::from_raw(
    //         image.into_raw(),
    //         ImageFormat::Rgb,
    //         width as usize,
    //         height as usize,
    //     );
    //     thumbnails.push(Thumbnail {
    //         image: Arc::new(image),
    //     })
    // }

    // let images = Arc::new(paths);
    // let current_image = 0;

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
    let window = WindowDesc::new(ui_builder).menu(menu).title("Gallery");

    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(AppState {
            images: Arc::new(Vec::new()),
            current_image: 0,
            thumbnails: Arc::new(Vec::new()),
        })
        .unwrap();
}

fn ui_builder() -> impl Widget<AppState> {
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
        // let image = Image::new()
        .interpolation_mode(InterpolationMode::Bilinear)
        .fill_mode(FillStrat::Contain);
    let image = DisplayImage {
        image: Rc::new(image),
    }
    .padding(Insets::new(0.0, 5.0, 0.0, 5.0));
    let image_view = Flex::row()
        .must_fill_main_axis(true)
        .with_child(left_button)
        .with_flex_child(image, FlexParams::new(1.0, None))
        .with_child(right_button)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .main_axis_alignment(MainAxisAlignment::SpaceBetween);

    let film_strip_list = List::new(|| {
        Image::new(ImageBuf::empty())
            .controller(ThumbnailController {})
            .fix_size(150.0, 150.0)
            .padding(15.0)
            .background(Painter::new(
                |ctx, (current_image, data): &(usize, Thumbnail), env| {
                    let is_hot = ctx.is_hot();
                    let is_active = ctx.is_active();
                    // let is_selected = data.is_selected;
                    let is_selected = *current_image == data.index;

                    // Color::rgb8(0x00, 0x75, 0xfc);
                    let background_color = if is_active {
                        Color::rgb8(0x87, 0x87, 0x87)
                    } else if is_hot {
                        Color::rgb8(0xc4, 0xc4, 0xc4)
                    } else if is_selected {
                        Color::rgb8(0x9e, 0x9e, 0x9e)
                    } else {
                        Color::rgb8(0xee, 0xee, 0xee)
                    };

                    let rect = ctx.size().to_rect();
                    ctx.stroke(rect, &background_color, 0.0);
                    ctx.fill(rect, &background_color);
                },
            ))
            // .border(Color::rgb8(255, 255, 255), 1.0)
            .on_click(
                |event: &mut EventCtx,
                 (current_image, data): &mut (usize, Thumbnail),
                 env| {
                    let select_image =
                        Selector::new("select_thumbnail").with(data.index);
                    event.submit_command(select_image);
                },
            )
    })
    .horizontal();
    // .lens(AppState::thumbnails);
    // .lens(
    //     AppStateLensBuilder::new()
    //         // .images(AppState::images)
    //         .current_image(AppState::current_image)
    //         .thumbnails(AppState::thumbnails)
    //         .build(),
    // );

    // let lens: AppStateLens<
    //     app_state_derived_lenses::images,
    //     app_state_derived_lenses::current_image,
    //     app_state_derived_lenses::thumbnails,
    // > = AppStateLensBuilder::new()
    //     .images(AppState::images)
    //     .current_image(AppState::current_image)
    //     .thumbnails(AppState::thumbnails)
    //     .build();
    // .lens(
    //     AppStateLensBuilder::new()
    //         .thumbnails(AppState::thumbnails)
    //         .images(AppState::images)
    //         .build(),
    // );
    // .lens(lens::Identity.map(
    //     // Expose shared data with children data
    //     |d: &AppState| (d.clone(), d.current_image),
    //     |d: &mut AppState, (new_d, _): (AppState, usize)| {
    //         // If shared data was changed reflect the changes in our AppData
    //         *d = new_d;
    //     },
    // ));
    // .controller(ListController {});

    let film_strip_view = Scroll::new(
        // Flex::row()
        // .must_fill_main_axis(true)
        // .with_child(film_strip_list)
        film_strip_list,
    )
    .horizontal()
    .fix_height(150.0)
    .background(Color::rgb8(0xee, 0xee, 0xee));
    // .padding(5.0);
    let layout = Flex::column()
        .must_fill_main_axis(true)
        .with_flex_child(image_view, FlexParams::new(1.0, None))
        .with_child(film_strip_view);
    Container::new(layout).background(druid::Color::rgb8(255, 255, 255))
}
