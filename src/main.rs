use data::{AppState, AppView};
use druid::{
    im::{vector, HashSet, Vector},
    AppLauncher, Widget, WindowDesc,
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
mod app_commands;

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
        .use_simple_logger()
        .launch(AppState {
            images: HashSet::new(),
            current_image_idx: 0,
            thumbnails: Vector::new(),
            // this will back the Navigator, so it always has to be initialized with something
            views: vector![AppView::MainView],
            all_images: Vector::new(),
            selected_folder: None,
        })
        .unwrap();
}

fn navigator() -> impl Widget<AppState> {
    Navigator::new(AppView::MainView, main_view)
        .with_view_builder(AppView::FolderView, folder_navigator)
}
