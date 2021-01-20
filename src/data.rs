use std::{path::PathBuf, sync::Arc};

use druid::{
    im::Vector,
    widget::{Container, Controller, Image, ListIter},
    Data, Env, Event, FileInfo, ImageBuf, Lens, LifeCycle, Selector, Widget,
};
use druid_navigator::navigator::{View, ViewController};

use crate::widget::{
    read_images, CREATED_THUMBNAIL, FINISHED_READING_FOLDER, OPEN_SELECTOR,
    SELECT_IMAGE_SELECTOR,
};

#[derive(Clone, Data, Lens, Debug)]
pub struct AppState {
    pub images: Arc<Vec<PathBuf>>,
    pub current_image_idx: usize,
    pub thumbnails: Vector<Thumbnail>,
    pub views: Vector<AppView>,
}

#[derive(Debug, Clone, Data, PartialEq, Hash, Eq)]
pub enum AppView {
    MainView,
    ImageView,
}

impl View for AppView {}
impl ViewController<AppView> for AppState {
    fn add_view(&mut self, view: AppView) {
        self.views.push_back(view);
    }

    fn pop_view(&mut self) {
        self.views.pop_back();
    }

    fn current_view(&self) -> &AppView {
        self.views.last().unwrap()
    }

    fn len(&self) -> usize {
        self.views.len()
    }

    fn is_empty(&self) -> bool {
        self.views.is_empty()
    }
}

pub struct AppStateController;

impl Controller<AppState, Container<AppState>> for AppStateController {
    fn event(
        &mut self,
        child: &mut Container<AppState>,
        ctx: &mut druid::EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::Command(open) if open.is(OPEN_SELECTOR) => {
                // I don't know if this is right
                // if I don't return here, the application crashes everytime
                // I close it because of unwrap() and can't find selector
                // is the command being sent periodically?
                let payload: &FileInfo = open.get_unchecked(Selector::new(
                    "druid-builtin.open-file-path",
                ));

                let path = payload.path();
                let sink = ctx.get_external_handle();
                read_images(sink, path.to_owned());
            }
            Event::Command(select_image)
                if select_image.is(SELECT_IMAGE_SELECTOR) =>
            {
                let index = select_image.get_unchecked(SELECT_IMAGE_SELECTOR);
                data.current_image_idx = *index;
            }
            Event::Command(paths) if paths.is(FINISHED_READING_FOLDER) => {
                let (paths, thumbnails) =
                    paths.get_unchecked(FINISHED_READING_FOLDER).clone();
                data.images = paths;
                data.current_image_idx = 0;
                data.thumbnails = thumbnails;
            }
            Event::Command(selector) if selector.is(CREATED_THUMBNAIL) => {
                let thumbnail = selector.get_unchecked(CREATED_THUMBNAIL);
                data.thumbnails[thumbnail.index] = thumbnail.clone();
            }
            _ => (),
        }
        child.event(ctx, event, data, env);
    }
}

impl ListIter<(usize, Thumbnail)> for AppState {
    fn for_each(&self, mut cb: impl FnMut(&(usize, Thumbnail), usize)) {
        for (i, item) in self.thumbnails.iter().enumerate() {
            cb(&(self.current_image_idx, item.to_owned()), i);
        }
    }

    fn for_each_mut(
        &mut self,
        mut cb: impl FnMut(&mut (usize, Thumbnail), usize),
    ) {
        for (i, item) in self.thumbnails.iter().enumerate() {
            let owned_item = item.to_owned();
            cb(&mut (self.current_image_idx, owned_item.clone()), i);
        }
    }

    fn data_len(&self) -> usize {
        self.thumbnails.len()
    }
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
        ctx: &mut druid::LifeCycleCtx,
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
        ctx: &mut druid::UpdateCtx,
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
