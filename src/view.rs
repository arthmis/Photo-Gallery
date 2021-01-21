use std::{
    fs::{self},
    thread,
};

use druid::{
    im::Vector,
    piet::{ImageFormat, InterpolationMode},
    widget::{
        Container, CrossAxisAlignment, FillStrat, Flex, FlexParams, Image,
        Label, List, MainAxisAlignment, Painter,
    },
    Color, EventCtx, ImageBuf, Insets, RenderContext, Selector, Target, Widget,
    WidgetExt,
};
use fs::read_dir;
use image::{imageops::thumbnail, io::Reader, RgbImage};
use log::error;
use walkdir::{DirEntry, WalkDir};

use crate::Scroll;
use crate::{
    data::{
        AppStateController, GalleryThumbnailController, ImageFolder,
        MainViewController, Thumbnail, ThumbnailController,
    },
    widget::DisplayImage,
    AppState,
};
// use druid::widget::Scroll;

fn image_gridview_builder() -> impl Widget<ImageFolder> {
    // this will display the folder name
    let folder_name = Label::dynamic(|data: &String, _env| data.clone())
        .with_text_color(Color::BLACK);

    // this will display the image thumbnails
    let thumbnails = List::new(|| {
        Image::new(ImageBuf::empty())
            .interpolation_mode(InterpolationMode::NearestNeighbor)
            .controller(GalleryThumbnailController {})
            .fix_size(150., 150.)
            .padding(15.)
    })
    .horizontal()
    .with_spacing(5.);

    let thumbnails = Scroll::new(thumbnails).horizontal();
    // let thumbnails =
    //     Flex::row().with_child(thumbnails).must_fill_main_axis(true);

    Flex::column()
        .with_child(folder_name.lens(ImageFolder::name))
        .with_child(thumbnails.lens(ImageFolder::thumbnails))
        .cross_axis_alignment(CrossAxisAlignment::Start)
}

pub fn main_view() -> Box<dyn Widget<AppState>> {
    let gallery_list = List::new(image_gridview_builder);
    // let gallery_list = Scroll::new(gallery_list).vertical();
    let layout = {
        let layout = Flex::column().with_child(gallery_list);
        // let layout = Flex::row()
        //     .with_child(layout)
        //     // .must_fill_main_axis(true)
        //     .background(Color::WHITE)
        //     .fix_width(1000.);
        layout
    }
    .lens(AppState::all_images);

    let container = Container::new(layout)
        .background(Color::WHITE)
        .controller(MainViewController)
        .on_added(|_self, ctx, data, _env| {
            let handle = ctx.get_external_handle();
            let folder = data.images.clone();
            thread::spawn(move || {
                let walk = WalkDir::new(&folder[0]).into_iter().filter_entry(
                    |entry| {
                        // only walks directories, not files, and only keeps directories
                        // that don't fail to read
                        if entry.path().is_dir() {
                            match read_dir(entry.path()) {
                                Ok(mut dir) => dir.next().is_some(),
                                Err(_) => false,
                            }
                        } else {
                            false
                        }
                    },
                );
                for entry in walk {
                    let entry = entry.unwrap();
                    let thumbnails = read_directory(&entry);
                    if !thumbnails.is_empty() {
                        let image_folder = ImageFolder {
                            name: entry.path().to_string_lossy().to_string(),
                            thumbnails,
                        };
                        handle
                            .submit_command(
                                FINISHED_READING_IMAGE_FOLDER,
                                image_folder,
                                Target::Auto,
                            )
                            .unwrap();
                    }
                }
            });
        });
    // Box::new(container)
    Box::new(Scroll::new(container).vertical())
}
pub const FINISHED_READING_IMAGE_FOLDER: Selector<ImageFolder> =
    Selector::new("finished_reading_image_folder");
fn read_directory(entry: &DirEntry) -> Vector<Thumbnail> {
    let mut images = Vector::new();
    let entries = fs::read_dir(entry.path()).unwrap();
    for file in entries {
        let file = file.unwrap();
        if file.path().is_file() {
            let image = match Reader::open(file.path()) {
                Ok(image) => match image.with_guessed_format() {
                    Ok(image) => image,
                    Err(err) => {
                        error!("Error getting image format: {}", err);
                        continue;
                    }
                },
                Err(err) => {
                    error!("Error opening file: {}", err);
                    continue;
                }
            };
            let image = match image.decode() {
                Ok(image) => image,
                Err(_) => {
                    continue;
                }
            }
            .to_rgb8();
            images.push_back(create_thumbnail(images.len(), image));
        }
    }
    images
}
fn create_thumbnail(index: usize, image: RgbImage) -> Thumbnail {
    let (width, height) = image.dimensions();
    // dbg!(width, height);
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

pub fn image_view_builder() -> Box<dyn Widget<AppState>> {
    let button_width = 50.0;
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
        if data.images.is_empty() || data.current_image_idx == 0 {
            return;
        }

        data.current_image_idx -= 1;
    })
    .fix_width(button_width)
    .expand_height();

    let right_button = crate::widget::Button::new(
        "❯",
        font_color,
        bg_color,
        hover_color,
        active_color,
    )
    .on_click(|_ctx, data: &mut AppState, _env| {
        if data.images.is_empty()
            || data.current_image_idx == data.images.len() - 1
        {
            return;
        }
        data.current_image_idx += 1;
    })
    .fix_width(button_width)
    .expand_height();

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

    let film_strip_list = List::new(|| {
        Image::new(ImageBuf::empty())
            .interpolation_mode(InterpolationMode::NearestNeighbor)
            .controller(ThumbnailController {})
            .fix_size(150.0, 150.0)
            .padding(15.0)
            .background(Painter::new(
                |ctx, (current_image, data): &(usize, Thumbnail), _env| {
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
                 (_current_image, data): &mut (usize, Thumbnail),
                 _env| {
                    let select_image =
                        Selector::new("select_thumbnail").with(data.index);
                    event.submit_command(select_image);
                },
            )
    })
    .horizontal();

    let film_strip_view = Scroll::new(film_strip_list)
        .horizontal()
        .background(Color::rgb8(0xee, 0xee, 0xee))
        .expand_width();

    let layout = Flex::column()
        .must_fill_main_axis(true)
        .with_flex_child(image_view, FlexParams::new(1.0, None))
        .with_child(film_strip_view);

    Box::new(
        Container::new(layout)
            .background(druid::Color::rgb8(255, 255, 255))
            .controller(AppStateController {}),
    )
}
