// #![allow(warnings)]
use std::sync::Arc;

use data::{AppState, AppView};
use druid::{
    commands, im::Vector, AppLauncher, FileDialogOptions, LocalizedString,
    MenuItem, Widget, WindowDesc,
};

// use druid_widget_nursery::navigator::{Navigator, View, ViewController};
use druid_navigator::navigator::Navigator;
use view::{main_view, ui_builder};

mod data;
pub mod scroll;
pub mod scroll_component;
mod view;
pub mod widget;
pub use scroll::Scroll;

// const IMAGE_FOLDER: &str = "./images - Copy";

fn main() {
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
    // let window = WindowDesc::new(ui_builder).menu(menu).title("Gallery");
    let window = WindowDesc::new(navigator).menu(menu).title("Gallery");

    AppLauncher::with_window(window)
        .use_simple_logger()
        .launch(AppState {
            images: Arc::new(Vec::new()),
            current_image_idx: 0,
            thumbnails: Vector::new(),
            views: Vector::new(),
        })
        .unwrap();
}

fn navigator() -> impl Widget<AppState> {
    Navigator::new(AppView::MainView, main_view)
        .with_view_builder(AppView::ImageView, ui_builder)
}
