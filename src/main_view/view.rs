use druid::{
    commands::SHOW_OPEN_PANEL,
    lens,
    widget::{
        Container, Controller, CrossAxisAlignment, Flex, Image, Label,
        MainAxisAlignment, Painter,
    },
    Color, Command, Cursor, FileDialogOptions, ImageBuf, LensExt,
    RenderContext, Target, Widget, WidgetExt,
};
use druid_gridview::GridView;

use crate::{
    app_commands::SELECTED_FOLDER,
    app_data::{AppState, GalleryThumbnailController, ImageFolder},
    widgets::{Button, Scroll},
};

use super::MainViewController;

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

    let gallery_list = GridView::new(image_gridview_builder)
        .wrap()
        .with_spacing(30.)
        .padding(20.);
    let layout = Flex::column()
        .with_child(menu_btns)
        .with_child(gallery_list)
        .cross_axis_alignment(CrossAxisAlignment::Start);

    let container = Container::new(layout).controller(MainViewController);

    Box::new(
        Scroll::new(container)
            .vertical()
            .expand_height()
            .background(Color::WHITE),
    )
}

fn image_gridview_builder() -> impl Widget<(ImageFolder, usize)> {
    // this lenses into the image folder found in the tuple
    // let thumbnails_lens = lens!((ImageFolder, usize), 0)
    //     .map(|data| data.thumbnails.clone(), |_folder, _put| ());
    let folder_thumbnail_lens = lens!((ImageFolder, usize), 0)
        .map(|data| data.folder_thumbnail.clone(), |_folder, _put| ());

    // this will display the folder name
    let folder_name =
        Label::dynamic(|(folder, _idx): &(ImageFolder, usize), _env| {
            folder
                .name
                .file_name()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
        .with_text_color(Color::BLACK)
        .padding(5.);

    let thumbnail = Image::new(ImageBuf::empty())
        .controller(GalleryThumbnailController)
        .lens(folder_thumbnail_lens)
        .fix_size(250., 250.);

    Flex::column()
        .with_child(folder_name)
        .with_child(thumbnail)
        .background(Painter::new(|ctx, _data, _env| {
            let is_hot = ctx.is_hot();
            let is_active = ctx.is_active();
            let (background_color, border_color, border_width) = if is_active {
                (
                    Color::rgb8(0x9f, 0x9f, 0x9f),
                    Color::rgb8(0x16, 0x69, 0xdd),
                    6.,
                )
            } else if is_hot {
                (
                    Color::rgb8(0xdd, 0xdd, 0xdd),
                    Color::rgb8(0x2a, 0x82, 0xfc),
                    3.,
                )
            } else {
                (
                    Color::rgb8(0xff, 0xff, 0xff),
                    Color::rgb8(0xaa, 0xaa, 0xaa),
                    1.,
                )
            };

            let rect = ctx.size().to_rect();
            ctx.stroke(rect, &border_color, border_width);
            ctx.fill(rect, &background_color);
        }))
        .controller(FolderThumbnailController)
        .on_click(|ctx, (_folder, idx), _env| {
            ctx.submit_command(Command::new(
                SELECTED_FOLDER,
                *idx,
                Target::Auto,
            ))
        })
}

struct FolderThumbnailController;

impl Controller<(ImageFolder, usize), Container<(ImageFolder, usize)>>
    for FolderThumbnailController
{
    fn event(
        &mut self,
        child: &mut Container<(ImageFolder, usize)>,
        ctx: &mut druid::EventCtx,
        event: &druid::Event,
        data: &mut (ImageFolder, usize),
        env: &druid::Env,
    ) {
        if ctx.is_hot() {
            ctx.set_cursor(&Cursor::OpenHand);
        }
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut Container<(ImageFolder, usize)>,
        ctx: &mut druid::LifeCycleCtx,
        event: &druid::LifeCycle,
        data: &(ImageFolder, usize),
        env: &druid::Env,
    ) {
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut Container<(ImageFolder, usize)>,
        ctx: &mut druid::UpdateCtx,
        old_data: &(ImageFolder, usize),
        data: &(ImageFolder, usize),
        env: &druid::Env,
    ) {
        child.update(ctx, old_data, data, env)
    }
}
