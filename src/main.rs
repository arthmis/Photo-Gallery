// #![allow(warnings)]
use std::{path::PathBuf, sync::Arc};

use data::{AppState, AppView};
use druid::{
    commands,
    im::{vector, Vector},
    AppLauncher, FileDialogOptions, LocalizedString, MenuItem, Widget,
    WindowDesc,
};

// use druid_widget_nursery::navigator::{Navigator, View, ViewController};
use druid_navigator::navigator::Navigator;
use view::{folder_navigator, main_view};

mod data;
pub mod scroll;
pub mod scroll_component;
mod view;
pub mod widget;
pub use scroll::Scroll;

const IMAGE_FOLDER: &str = "D:\\My Pictures";

fn main() {
    // let menu = {
    //     let menu = druid::MenuDesc::empty();
    //     let open = {
    //         let open_folder_options = FileDialogOptions::new()
    //             .select_directories()
    //             .default_name("Open");
    //         MenuItem::new(
    //             LocalizedString::new("Open Folder"),
    //             commands::SHOW_OPEN_PANEL.with(open_folder_options),
    //         )
    //     };
    //     menu.append(open).append_separator()
    // };
    // let window = WindowDesc::new(ui_builder).menu(menu).title("Gallery");
    let window = WindowDesc::new(navigator).title("Gallery");
    // let window = WindowDesc::new(test_ui).menu(menu).title("Gallery");

    AppLauncher::with_window(window)
        // .use_simple_logger()
        .launch(AppState {
            // images: Arc::new(Vec::new()),
            images: Arc::new(vec![PathBuf::from(IMAGE_FOLDER)]),
            current_image_idx: 0,
            thumbnails: Vector::new(),
            // this will back the Navigator, so it always has to be initialized with something
            views: vector![AppView::MainView],
            all_images: Vector::new(),
            selected_folder: None,
            // test_text: vector![
            //     "Hello".to_string(),
            //     "Test".to_string(),
            //     "Another".to_string()
            // ],
        })
        .unwrap();
}

fn navigator() -> impl Widget<AppState> {
    Navigator::new(AppView::MainView, main_view)
        // .with_view_builder(AppView::ImageView, image_view_builder)
        // .with_view_builder(AppView::FolderView, folder_view)
        .with_view_builder(AppView::FolderView, folder_navigator)
}

// fn test_ui() -> impl Widget<AppState> {
//     let list = List::new(|| {
//         Label::dynamic(|data: &String, _env| data.clone())
//             .fix_size(2000., 150.)
//             .padding(15.)
//             .background(Color::BLUE)
//     })
//     .horizontal()
//     .fix_height(200.)
//     .lens(AppState::test_text);
//     let thumbnails = Scroll::new(list).horizontal();
//     let layout = {
//         let layout = Flex::column().with_child(thumbnails);
//         let layout = Flex::row().with_child(layout).background(Color::WHITE);
//         layout
//     };

//     let container = Container::new(layout)
//         .background(Color::WHITE)
//         .fix_height(2_000.);
//     Scroll::new(container).vertical()
// }

// fn search_for_images() {}
