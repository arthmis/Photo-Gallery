use app_data::AppState;
use druid::{
    im::{vector, HashSet, Vector},
    AppLauncher, Widget, WindowDesc,
};

// use druid_widget_nursery::navigator::{Navigator, View, ViewController};
use druid_navigator::navigator::Navigator;
use folder_view::folder_navigator;
use log::error;
use main_view::{main_view, AppView};

mod app_commands;
mod app_data;
mod folder_view;
mod main_view;
pub mod widgets;

fn main() {
    let window = WindowDesc::new(navigator).title("Gallery");

    match AppLauncher::with_window(window).use_simple_logger().launch(
        AppState {
            images: HashSet::new(),
            current_image_idx: 0,
            thumbnails: Vector::new(),
            // this will back the Navigator, so it always has to be initialized with something
            views: vector![AppView::MainView],
            all_images: Vector::new(),
            selected_folder: None,
        },
    ) {
        Ok(_) => {}
        Err(err) => {
            error!("There was an error launching the application: {}", err);
        }
    }
}

fn navigator() -> impl Widget<AppState> {
    Navigator::new(AppView::MainView, main_view)
        .with_view_builder(AppView::FolderView, folder_navigator)
}
