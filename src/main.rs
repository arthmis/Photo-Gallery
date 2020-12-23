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
        Align, Button, Container, CrossAxisAlignment, FillStrat, Flex, FlexParams, Image, Label,
        MainAxisAlignment, Padding, SizedBox, TextBox,
    },
    AppLauncher, Color, Command, Data, Env, FileDialogOptions, ImageBuf, Lens, LocalizedString,
    MenuDesc, MenuItem, Selector, Target, Widget, WidgetExt, WindowDesc,
};

pub mod widget;
use crate::widget::*;
#[derive(Clone, Data, Lens)]
pub struct AppState {
    images: Arc<Vec<PathBuf>>,
    current_image: usize,
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

        let right_button =
            crate::widget::Button::new("❯", font_color, bg_color, hover_color, active_color)
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
        let layout = Flex::row()
            .must_fill_main_axis(true)
            .with_child(left_button)
            .with_flex_child(image, FlexParams::new(1.0, None))
            // .with_child(image)
            .with_child(right_button)
            .cross_axis_alignment(CrossAxisAlignment::Center)
            .main_axis_alignment(MainAxisAlignment::SpaceBetween);

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
        })
        .unwrap();
}
