use druid::{
    commands::SHOW_OPEN_PANEL,
    lens,
    widget::{Container, Flex, Image, Label, MainAxisAlignment, Painter},
    Color, Command, FileDialogOptions, ImageBuf, LensExt, RenderContext,
    Target, Widget, WidgetExt,
};
use druid_gridview::GridView;

use crate::{
    app_commands::SELECTED_FOLDER,
    app_data::{AppState, GalleryThumbnailController, ImageFolder},
    widgets::{Button, Scroll},
};

use super::MainViewController;

fn image_gridview_builder() -> impl Widget<(ImageFolder, usize)> {
    // this lenses into the image folder found in the tuple
    let thumbnails_lens = lens!((ImageFolder, usize), 0)
        .map(|data| data.thumbnails.clone(), |_folder, _put| ());

    // this will display the folder name
    let folder_name =
        Label::dynamic(|(folder, _idx): &(ImageFolder, usize), _env| {
            folder.name.clone().to_string_lossy().to_string()
        })
        .with_text_color(Color::BLACK)
        .padding(5.)
        .background(Painter::new(|ctx, _data, _env| {
            let is_hot = ctx.is_hot();
            let is_active = ctx.is_active();
            let background_color = if is_active {
                Color::rgb8(0x9f, 0x9f, 0x9f)
            } else if is_hot {
                Color::rgb8(0xdd, 0xdd, 0xdd)
            } else {
                Color::rgb8(0xff, 0xff, 0xff)
            };

            let rect = ctx.size().to_rect();
            ctx.stroke(rect, &background_color, 0.);
            ctx.fill(rect, &background_color);
        }))
        .on_click(|ctx, (_folder, idx), _env| {
            ctx.submit_command(Command::new(
                SELECTED_FOLDER,
                *idx,
                Target::Auto,
            ))
        });

    let thumbnail = Image::new(ImageBuf::empty())
        .controller(GalleryThumbnailController)
        .lens(thumbnails_lens.index(0))
        .fix_size(300., 300.);

    // modify this to have consistent sizes
    Flex::column()
        .with_child(folder_name)
        .with_child(thumbnail)
        .fix_size(300., 300.)
}

pub fn main_view() -> Box<dyn Widget<AppState>> {
    let add_folder_btn = Button::new(
        "+ Add Folder",
        Color::BLACK,
        Color::rgb8(0xff, 0xff, 0xff),
        Color::rgb8(0xdd, 0xdd, 0xdd),
        Color::rgb8(0x9f, 0x9f, 0x9f),
        16.,
    )
    .on_click(|ctx, _data, _env| {
        let file_dialog = FileDialogOptions::new().select_directories();
        ctx.submit_command(SHOW_OPEN_PANEL.with(file_dialog));
    })
    .fix_height(50.);

    let menu_btns = Container::new(
        Flex::row()
            .with_child(add_folder_btn)
            .must_fill_main_axis(true)
            .main_axis_alignment(MainAxisAlignment::End)
            .fix_height(40.),
    )
    .border(Color::rgb8(0xcc, 0xcc, 0xcc), 1.);

    let gallery_list = GridView::new(image_gridview_builder).wrap();
    let layout = Flex::column()
        .with_child(menu_btns)
        .with_child(gallery_list);

    let container = Container::new(layout)
        .background(Color::WHITE)
        .controller(MainViewController);

    Box::new(Scroll::new(container).vertical())
}
