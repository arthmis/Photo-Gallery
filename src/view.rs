use druid::{
    piet::InterpolationMode,
    widget::{
        Container, CrossAxisAlignment, FillStrat, Flex, FlexParams, Image,
        Label, List, MainAxisAlignment, Painter,
    },
    Color, EventCtx, ImageBuf, Insets, RenderContext, Selector, Widget,
    WidgetExt,
};

use crate::{
    data::{AppStateController, Thumbnail, ThumbnailController},
    widget::DisplayImage,
    AppState, Scroll,
};

pub fn main_view() -> Box<dyn Widget<AppState>> {
    let layout = Flex::column();
    let container = Container::new(layout);
    Box::new(container)
}

pub fn ui_builder() -> Box<dyn Widget<AppState>> {
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
