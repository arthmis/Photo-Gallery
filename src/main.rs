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

#[derive(Clone, Data, Lens, Debug)]
pub struct AppState {
    images: Arc<Vec<PathBuf>>,
    current_image_idx: usize,
    thumbnails: Arc<Vec<Thumbnail>>,
}

pub struct AppStateController;

impl Controller<AppState, Container<AppState>> for AppStateController {
    fn event(
        &mut self,
        child: &mut Container<AppState>,
        ctx: &mut druid::EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::Command(open) if open.is(OPEN_SELECTOR) => {
                dbg!("got open command in display image");
                // I don't know if this is right
                // if I don't return here, the application crashes everytime
                // I close it because of unwrap() and can't find selector
                // is the command being sent periodically?
                let payload: &FileInfo = open.get_unchecked(Selector::new(
                    "druid-builtin.open-file-path",
                ));

                let path = payload.path();
                let sink = ctx.get_external_handle();
                read_images(sink, path.to_owned());

                ctx.request_layout();
                ctx.request_paint();
            }
            Event::Command(select_image)
                if select_image.is(SELECT_IMAGE_SELECTOR) =>
            {
                let index = select_image.get_unchecked(SELECT_IMAGE_SELECTOR);
                data.current_image_idx = *index;

                ctx.request_layout();
                ctx.request_paint();
            }
            Event::Command(paths) if paths.is(FINISHED_READING_FOLDER) => {
                let (paths, thumbnails) =
                    paths.get_unchecked(FINISHED_READING_FOLDER).clone();
                data.images = paths;
                data.current_image_idx = 0;
                data.thumbnails = thumbnails;
            }
            _ => (),
        }
        child.event(ctx, event, data, env);
    }
}

impl ListIter<(usize, Thumbnail)> for AppState {
    fn for_each(&self, mut cb: impl FnMut(&(usize, Thumbnail), usize)) {
        for (i, item) in self.thumbnails.iter().enumerate() {
            cb(&(self.current_image_idx, item.to_owned()), i);
        }
    }

    fn for_each_mut(
        &mut self,
        mut cb: impl FnMut(&mut (usize, Thumbnail), usize),
    ) {
        let mut new_data = Vec::with_capacity(self.thumbnails.len());
        let mut any_changed = false;
        for (i, item) in self.thumbnails.iter().enumerate() {
            let mut owned_item = item.to_owned();
            cb(&mut (self.current_image_idx, owned_item.clone()), i);
            if !any_changed && !item.same(&owned_item) {
                any_changed = true;
            }
            new_data.push(owned_item);
        }
        if any_changed {
            self.thumbnails = Arc::new(new_data);
        }
    }

    fn data_len(&self) -> usize {
        self.thumbnails.len()
    }
}

pub struct ListController;
impl Controller<AppState, List<(usize, Thumbnail)>> for ListController {
    fn event(
        &mut self,
        child: &mut List<(usize, Thumbnail)>,
        ctx: &mut EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        let open_selector: Selector<AppState> =
            Selector::new("druid-builtin.open-file-path");
        let select_image_selector: Selector<usize> =
            Selector::new("select_thumbnail");
        match event {
            Event::Command(open) if open.is(open_selector) => {
                dbg!("got open command in list controller");
                // I don't know if this is right
                // if I don't return here, the application crashes everytime
                // I close it because of unwrap() and can't find selector
                // is the command being sent periodically?
                // let payload: &FileInfo = open.get_unchecked(Selector::new(
                //     "druid-builtin.open-file-path",
                // ));

                // let path = payload.path();
                // let mut paths: Vec<PathBuf> = std::fs::read_dir(path)
                //     .unwrap()
                //     .map(|path| path.unwrap().path())
                //     .collect();

                // data.images = Arc::new(paths);
                // data.current_image = 0;
                // data.create_thumbnails();

                ctx.request_layout();
                ctx.request_paint();
            }
            Event::Command(select_image)
                if select_image.is(select_image_selector) =>
            {
                // let index = select_image.get_unchecked(select_image_selector);
                // data.current_image = *index;

                dbg!("got select command in list controller");
                ctx.request_layout();
                ctx.request_paint();
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut List<(usize, Thumbnail)>,
        ctx: &mut druid::LifeCycleCtx,
        event: &LifeCycle,
        data: &AppState,
        env: &Env,
    ) {
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut List<(usize, Thumbnail)>,
        ctx: &mut druid::UpdateCtx,
        old_data: &AppState,
        data: &AppState,
        env: &Env,
    ) {
        child.update(ctx, old_data, data, env)
    }
}

#[derive(Clone, Data, Lens, Debug)]
pub struct Thumbnail {
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
            current_image_idx: 0,
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
        if data.current_image_idx == 0 {
            data.current_image_idx = data.images.len() - 1;
        } else {
            data.current_image_idx -= 1;
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
        if data.current_image_idx == data.images.len() - 1 {
            data.current_image_idx = 0
        } else {
            data.current_image_idx += 1;
        }
    })
    .fix_size(button_width, button_height);

    let image = Image::new(ImageBuf::empty())
        .interpolation_mode(InterpolationMode::Bilinear)
        .fill_mode(FillStrat::Contain);
    let image =
        DisplayImage::new(image).padding(Insets::new(0.0, 5.0, 0.0, 5.0));

    let image_view = Flex::row()
        .must_fill_main_axis(true)
        .with_child(left_button)
        .with_flex_child(image, FlexParams::new(1.0, None))
        .with_child(right_button)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .main_axis_alignment(MainAxisAlignment::SpaceBetween);

    let film_strip_list: List<(usize, Thumbnail)> = List::new(|| {
        Image::new(ImageBuf::empty())
            .controller(ThumbnailController {})
            .fix_size(150.0, 150.0)
            .padding(15.0)
            .background(Painter::new(
                |ctx, (current_image, data): &(usize, Thumbnail), env| {
                    let is_hot = ctx.is_hot();
                    let is_active = ctx.is_active();
                    let is_selected = *current_image == data.index;

                    let background_color = if is_selected {
                        Color::rgb8(0x9e, 0x9e, 0x9e)
                    } else if is_active {
                        Color::rgb8(0x87, 0x87, 0x87)
                    } else if is_hot {
                        Color::rgb8(0xc4, 0xc4, 0xc4)
                    } else {
                        Color::rgb8(0xee, 0xee, 0xee)
                    };

                    let rect = ctx.size().to_rect();
                    ctx.stroke(rect, &background_color, 0.0);
                    ctx.fill(rect, &background_color);
                },
            ))
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

    let film_strip_view = Scroll::new(film_strip_list)
        .horizontal()
        .background(Color::rgb8(0xee, 0xee, 0xee));

    let layout = Flex::column()
        .must_fill_main_axis(true)
        .with_flex_child(image_view, FlexParams::new(1.0, None))
        .with_child(film_strip_view);
    Container::new(layout)
        .background(druid::Color::rgb8(255, 255, 255))
        .controller(AppStateController {})
}
