use std::{path::PathBuf, sync::Arc};

use druid::{
    im::{HashSet, Vector},
    widget::{Controller, Image},
    Data, Env, Event, ImageBuf, Lens, LifeCycle, LifeCycleCtx, UpdateCtx,
    Widget,
};

use crate::main_view::AppView;

#[derive(Clone, Data, Lens, Debug)]
pub struct AppState {
    pub folder_paths: HashSet<Arc<PathBuf>>,
    pub current_image_idx: usize,
    pub views: Vector<AppView>,
    pub all_images: Vector<ImageFolder>,
    pub selected_folder: Option<usize>,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct ImageFolder {
    pub name: Arc<PathBuf>,
    pub folder_thumbnail: Thumbnail,
    pub paths: Vector<Arc<PathBuf>>,
    pub selected: Option<usize>,
    pub thumbnails: Vector<Thumbnail>,
}

#[derive(Clone, Lens, Debug)]
pub struct Thumbnail {
    pub index: usize,
    pub image: ImageBuf,
}

impl Data for Thumbnail {
    fn same(&self, other: &Self) -> bool {
        self.index == other.index
            && self
                .image
                .raw_pixels_shared()
                .same(&other.image.raw_pixels_shared())
    }
}

pub struct GalleryThumbnailController;

impl Controller<Thumbnail, Image> for GalleryThumbnailController {
    fn lifecycle(
        &mut self,
        child: &mut Image,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &Thumbnail,
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            child.set_image_data(data.image.clone());
            ctx.request_layout();
            ctx.request_paint();
        }
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut Image,
        ctx: &mut UpdateCtx,
        old_data: &Thumbnail,
        data: &Thumbnail,
        env: &Env,
    ) {
        let old_image = old_data;
        let current_image = data;
        if !current_image.same(old_image) {
            child.set_image_data(current_image.image.clone());
            ctx.request_layout();
            ctx.request_paint();
        }
        child.update(ctx, old_data, data, env)
    }
}

pub struct ThumbnailController;

impl Controller<(usize, Thumbnail), Image> for ThumbnailController {
    fn event(
        &mut self,
        child: &mut Image,
        ctx: &mut druid::EventCtx,
        event: &Event,
        data: &mut (usize, Thumbnail),
        env: &Env,
    ) {
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut Image,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &(usize, Thumbnail),
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            child.set_image_data(data.1.image.clone());
            ctx.request_layout();
            ctx.request_paint();
        }
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut Image,
        ctx: &mut UpdateCtx,
        old_data: &(usize, Thumbnail),
        data: &(usize, Thumbnail),
        env: &Env,
    ) {
        let (_, old_image) = old_data;
        let (_, current_image) = data;
        if !current_image.same(old_image) {
            child.set_image_data(current_image.image.clone());
            ctx.request_layout();
            ctx.request_paint();
        }
        child.update(ctx, old_data, data, env)
    }
}
