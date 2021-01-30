use std::{path::Path, thread};

use druid::{
    piet::{ImageFormat, InterpolationMode},
    widget::{
        Container, Controller, CrossAxisAlignment, FillStrat, Flex, FlexParams,
        Image, Label, MainAxisAlignment, Painter, Scope,
    },
    Color, Command, Env, Event, ImageBuf, LensExt, RenderContext, Target,
    Widget, WidgetExt,
};
use druid_gridview::GridView;
use druid_navigator::navigator::Navigator;
use druid_widget_nursery::DynamicSizedBox;
use image::{imageops::thumbnail, io::Reader, ImageError, RgbImage};

use crate::{
    app_commands::{
        CREATED_THUMBNAIL, POP_FOLDER_VIEW, POP_VIEW,
        PUSH_VIEW_WITH_SELECTED_IMAGE, SELECT_IMAGE_SELECTOR,
    },
    app_data::{AppState, Thumbnail},
    folder_view::{
        DisplayImageController, FolderGalleryState, FolderView,
        FolderViewController, GalleryTransfer,
    },
    widgets::{Button, Scroll},
};

use super::FolderThumbnailController;

pub fn folder_navigator() -> Box<dyn Widget<AppState>> {
    let navigator = Navigator::new(FolderView::Folder, folder_view_main)
        .with_view_builder(FolderView::SingleImage, image_view_builder);

    let scope = Scope::from_function(
        FolderGalleryState::new,
        GalleryTransfer,
        navigator,
    );

    Box::new(scope)
}

pub fn folder_view_main() -> Box<dyn Widget<FolderGalleryState>> {
    // let left_arrow_svg = include_str!("..\\icons\\arrow-left-short.svg")
    //     .parse::<SvgData>()
    //     .unwrap();
    // let left_svg = Svg::new(left_arrow_svg.clone()).fill_mode(FillStrat::Fill);
    // let back_button = SvgButton::new(
    //     left_svg,
    let back_button = Button::new(
        "←",
        Color::BLACK,
        Color::rgb8(0xff, 0xff, 0xff),
        Color::rgb8(0xcc, 0xcc, 0xcc),
        Color::rgb8(0x90, 0x90, 0x90),
        16.,
    )
    .on_click(|ctx, _data, _env| {
        ctx.submit_command(Command::new(POP_VIEW, (), Target::Auto));
    });

    let title = Label::dynamic(|data: &String, _env| data.clone())
        .with_text_color(Color::BLACK)
        .lens(FolderGalleryState::name.map(
            |data| data.to_string_lossy().to_owned().to_string(),
            |_path, _data_path| (),
        ));

    let header = Flex::row()
        .with_child(back_button)
        .with_spacer(10.)
        .with_flex_child(title, 1.0)
        .main_axis_alignment(MainAxisAlignment::Start);

    let gallery = GridView::new(|| {
        Image::new(ImageBuf::empty())
            .interpolation_mode(InterpolationMode::NearestNeighbor)
            .controller(FolderThumbnailController)
            .fix_size(150., 150.)
            .padding(5.)
            .background(Painter::new(|ctx, (_thumbnail, _selected), _env| {
                let is_hot = ctx.is_hot();
                let is_active = ctx.is_active();
                let background_color = if is_active {
                    Color::rgb8(0x90, 0x90, 0x90)
                } else if is_hot {
                    Color::rgb8(0xcc, 0xcc, 0xcc)
                } else {
                    Color::rgb8(0xff, 0xff, 0xff)
                };
                let rect = ctx.size().to_rect();
                ctx.stroke(rect, &background_color, 0.0);
                ctx.fill(rect, &background_color);
            }))
            .on_click(|ctx, data, _env| {
                ctx.submit_command(Command::new(
                    PUSH_VIEW_WITH_SELECTED_IMAGE,
                    (FolderView::SingleImage, data.1),
                    Target::Auto,
                ));
            })
    })
    .wrap();

    let gallery = gallery.align_left();
    let gallery =
        DynamicSizedBox::new(Scroll::new(gallery).vertical().expand_width())
            .with_width(0.95);

    let layout = Flex::column()
        .with_child(header)
        .with_flex_child(gallery, 1.0)
        .expand_width()
        .background(Color::WHITE)
        .controller(FolderViewController)
        // TODO: this shouldn't do any work if this folder was loaded previously
        .on_added(|_self, ctx, data, _env| {
            let handle = ctx.get_external_handle();
            let image_paths = data.paths.clone();
            thread::spawn(move || {
                for (i, path) in image_paths.iter().enumerate() {
                    let thumbnail =
                        create_thumbnail_from_path(&path, i).unwrap();
                    handle
                        .submit_command(
                            CREATED_THUMBNAIL,
                            thumbnail,
                            Target::Auto,
                        )
                        .unwrap();
                }
            });
        });
    Box::new(layout)
}

pub fn image_view_builder() -> Box<dyn Widget<FolderGalleryState>> {
    let button_width = 50.0;
    let back_button = Button::new(
        "←",
        Color::BLACK,
        Color::rgb8(0xff, 0xff, 0xff),
        Color::rgb8(0xcc, 0xcc, 0xcc),
        Color::rgb8(0x90, 0x90, 0x90),
        16.,
    )
    .on_click(|ctx, _data, _env| {
        ctx.submit_command(Command::new(POP_FOLDER_VIEW, (), Target::Auto));
    })
    .align_left()
    .fix_width(button_width);

    let font_color = Color::rgb8(0, 0, 0);
    let bg_color = Color::rgb8(0xff, 0xff, 0xff);
    let hover_color = Color::rgb8(0xcc, 0xcc, 0xcc);
    let active_color = Color::rgb8(0x90, 0x90, 0x90);

    let left_button = Button::new(
        "❮",
        font_color.clone(),
        bg_color.clone(),
        hover_color.clone(),
        active_color.clone(),
        16.,
    )
    .on_click(|_ctx, data: &mut FolderGalleryState, _env| {
        if data.paths.is_empty() || data.selected_image == 0 {
            return;
        }

        data.selected_image -= 1;
    })
    .fix_width(button_width)
    .expand_height();

    let right_button =
        Button::new("❯", font_color, bg_color, hover_color, active_color, 16.)
            .on_click(|_ctx, data: &mut FolderGalleryState, _env| {
                if data.paths.is_empty()
                    || data.selected_image == data.paths.len() - 1
                {
                    return;
                }
                data.selected_image += 1;
            })
            .fix_width(button_width)
            .expand_height();

    let image = Image::new(ImageBuf::empty())
        .interpolation_mode(InterpolationMode::Bilinear)
        .fill_mode(FillStrat::Contain)
        .controller(DisplayImageController::new());

    let left_side_buttons = Flex::column()
        .with_child(back_button)
        .with_flex_child(left_button, 1.0);
    let image_view = Flex::row()
        .must_fill_main_axis(true)
        .with_child(left_side_buttons)
        .with_flex_child(image, FlexParams::new(1.0, None))
        .with_child(right_button)
        .cross_axis_alignment(CrossAxisAlignment::Center)
        .main_axis_alignment(MainAxisAlignment::SpaceBetween);

    let layout = Flex::column()
        .must_fill_main_axis(true)
        .with_flex_child(image_view, FlexParams::new(1.0, None));

    let container = Container::new(layout)
        .background(druid::Color::rgb8(255, 255, 255))
        .controller(ImageViewController);

    Box::new(container)
}

// TODO: this will eventually be an alternative view for the folder view
// pub fn filmstrip_view_builder() -> Box<dyn Widget<AppState>> {
//     let button_width = 50.0;
//     let font_color = Color::rgb8(0, 0, 0);
//     let bg_color = Color::rgb8(0xff, 0xff, 0xff);
//     let hover_color = Color::rgb8(0xcc, 0xcc, 0xcc);
//     let active_color = Color::rgb8(0x90, 0x90, 0x90);

//     let left_button = crate::widget::Button::new(
//         "❮",
//         font_color.clone(),
//         bg_color.clone(),
//         hover_color.clone(),
//         active_color.clone(),
//     )
//     .on_click(|_ctx, data: &mut AppState, _env| {
//         if data.images.is_empty() || data.current_image_idx == 0 {
//             return;
//         }

//         data.current_image_idx -= 1;
//     })
//     .fix_width(button_width)
//     .expand_height();

//     let right_button = crate::widget::Button::new(
//         "❯",
//         font_color,
//         bg_color,
//         hover_color,
//         active_color,
//     )
//     .on_click(|_ctx, data: &mut AppState, _env| {
//         if data.images.is_empty()
//             || data.current_image_idx == data.images.len() - 1
//         {
//             return;
//         }
//         data.current_image_idx += 1;
//     })
//     .fix_width(button_width)
//     .expand_height();

//     let image = Image::new(ImageBuf::empty())
//         .interpolation_mode(InterpolationMode::Bilinear)
//         .fill_mode(FillStrat::Contain);
//     let image =
//         DisplayImage::new(image).padding(Insets::new(0.0, 5.0, 0.0, 5.0));

//     let image_view = Flex::row()
//         .must_fill_main_axis(true)
//         .with_child(left_button)
//         .with_flex_child(image, FlexParams::new(1.0, None))
//         .with_child(right_button)
//         .cross_axis_alignment(CrossAxisAlignment::Center)
//         .main_axis_alignment(MainAxisAlignment::SpaceBetween);

//     let film_strip_list = List::new(|| {
//         Image::new(ImageBuf::empty())
//             .interpolation_mode(InterpolationMode::NearestNeighbor)
//             .controller(ThumbnailController {})
//             .fix_size(150.0, 150.0)
//             .padding(15.0)
//             .background(Painter::new(
//                 |ctx, (current_image, data): &(usize, Thumbnail), _env| {
//                     let is_hot = ctx.is_hot();
//                     let is_active = ctx.is_active();
//                     let is_selected = *current_image == data.index;

//                     let background_color = if is_selected {
//                         Color::rgb8(0x9e, 0x9e, 0x9e)
//                     } else if is_active {
//                         Color::rgb8(0x87, 0x87, 0x87)
//                     } else if is_hot {
//                         Color::rgb8(0xc4, 0xc4, 0xc4)
//                     } else {
//                         Color::rgb8(0xee, 0xee, 0xee)
//                     };

//                     let rect = ctx.size().to_rect();
//                     ctx.stroke(rect, &background_color, 0.0);
//                     ctx.fill(rect, &background_color);
//                 },
//             ))
//             .on_click(
//                 |event: &mut EventCtx,
//                  (_current_image, data): &mut (usize, Thumbnail),
//                  _env| {
//                     let select_image =
//                         Selector::new("select_thumbnail").with(data.index);
//                     event.submit_command(select_image);
//                 },
//             )
//     })
//     .horizontal();

//     let film_strip_view = Scroll::new(film_strip_list)
//         .horizontal()
//         .background(Color::rgb8(0xee, 0xee, 0xee))
//         .expand_width();

//     let layout = Flex::column()
//         .must_fill_main_axis(true)
//         .with_flex_child(image_view, FlexParams::new(1.0, None))
//         .with_child(film_strip_view);

//     Box::new(
//         Container::new(layout)
//             .background(druid::Color::rgb8(255, 255, 255))
//             .controller(AppStateController {}),
//     )
// }
//         .background(Color::rgb8(0xee, 0xee, 0xee))
//         .expand_width();

//     let layout = Flex::column()
//         .must_fill_main_axis(true)
//         .with_flex_child(image_view, FlexParams::new(1.0, None))
//         .with_child(film_strip_view);

//     Box::new(
//         Container::new(layout)
//             .background(druid::Color::rgb8(255, 255, 255))
//             .controller(AppStateController {}),
//     )
// }

fn create_thumbnail(index: usize, image: RgbImage) -> Thumbnail {
    let (width, height) = image.dimensions();
    let (new_width, new_height) = {
        let max_height = 150.0;
        let scale = max_height / height as f64;
        let scaled_width = width as f64 * scale;
        let scaled_height = height as f64 * scale;
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
    Thumbnail { index, image }
}

pub fn create_thumbnail_from_path(
    path: &Path,
    idx: usize,
) -> Result<Thumbnail, ImageError> {
    let image = Reader::open(path)?;
    let image = image.decode()?.to_rgb8();
    Ok(create_thumbnail(idx, image))
}

struct ImageViewController;

impl Controller<FolderGalleryState, Container<FolderGalleryState>>
    for ImageViewController
{
    fn event(
        &mut self,
        child: &mut Container<FolderGalleryState>,
        ctx: &mut druid::EventCtx,
        event: &Event,
        data: &mut FolderGalleryState,
        env: &Env,
    ) {
        match event {
            Event::Command(select_image)
                if select_image.is(SELECT_IMAGE_SELECTOR) =>
            {
                let index = select_image.get_unchecked(SELECT_IMAGE_SELECTOR);
                data.selected_image = *index;
            }
            _ => (),
        }
        child.event(ctx, event, data, env);
    }
}
