use std::{path::PathBuf, rc::Rc, sync::Arc};

use druid::{
    im::Vector,
    widget::{Container, Controller, Image, ListIter, ScopeTransfer},
    Data, Env, Event, FileInfo, ImageBuf, Lens, LifeCycle, Selector, Widget,
};
use druid_gridview::GridIter;
use druid_navigator::navigator::{View, ViewController};

use crate::{
    view::{
        FINISHED_READING_IMAGE_FOLDER, GALLERY_SELECTED_IMAGE, POP_VIEW,
        PUSH_VIEW, SELECTED_FOLDER,
    },
    widget::{
        read_images, CREATED_THUMBNAIL, FINISHED_READING_FOLDER, OPEN_SELECTOR,
        SELECT_IMAGE_SELECTOR,
    },
};

#[derive(Clone, Data, Lens, Debug)]
pub struct AppState {
    pub images: Arc<Vec<PathBuf>>,
    pub current_image_idx: usize,
    pub thumbnails: Vector<Thumbnail>,
    pub views: Vector<AppView>,
    pub all_images: Vector<ImageFolder>,
    pub selected_folder: Option<usize>,
    // pub test_text: Vector<String>,
}

impl ListIter<(ImageFolder, usize)> for AppState {
    fn for_each(&self, mut cb: impl FnMut(&(ImageFolder, usize), usize)) {
        for (i, image_folder) in self.all_images.iter().enumerate() {
            cb(&(image_folder.clone(), i), i)
        }
    }

    fn for_each_mut(
        &mut self,
        mut cb: impl FnMut(&mut (ImageFolder, usize), usize),
    ) {
        for (i, image_folder) in self.all_images.iter_mut().enumerate() {
            cb(&mut (image_folder.clone(), i), i)
        }
    }

    fn data_len(&self) -> usize {
        self.all_images.len()
    }
}

// impl GridIter<()

#[derive(Debug, Clone, Data, PartialEq, Hash, Eq)]
pub enum AppView {
    MainView,
    ImageView,
    FolderView,
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

pub struct MainViewController;

impl Controller<AppState, Container<AppState>> for MainViewController {
    fn event(
        &mut self,
        child: &mut Container<AppState>,
        ctx: &mut druid::EventCtx,
        event: &Event,
        data: &mut AppState,
        env: &Env,
    ) {
        match event {
            Event::Command(selector)
                if selector.is(FINISHED_READING_IMAGE_FOLDER) =>
            {
                let image_folder =
                    selector.get_unchecked(FINISHED_READING_IMAGE_FOLDER);
                data.all_images.push_back(image_folder.clone());
                ctx.request_layout();
                ctx.request_paint();
            }
            Event::Command(selector) if selector.is(SELECTED_FOLDER) => {
                let selected = selector.get_unchecked(SELECTED_FOLDER);
                data.selected_folder = Some(*selected);
                data.add_view(AppView::FolderView);
            }
            Event::Command(selector) if selector.is(POP_VIEW) => {
                dbg!(data.views.len());
                data.pop_view();
                dbg!(data.views.len());
            }
            Event::Command(selector) if selector.is(PUSH_VIEW) => {
                let view = selector.get_unchecked(PUSH_VIEW);
                data.add_view(view.clone());
            }
            _ => {}
        }
        child.event(ctx, event, data, env)
    }
}
pub struct ImageViewController;

impl Controller<ImageViewState, Container<ImageViewState>>
    for ImageViewController
{
    fn event(
        &mut self,
        child: &mut Container<ImageViewState>,
        ctx: &mut druid::EventCtx,
        event: &Event,
        data: &mut ImageViewState,
        env: &Env,
    ) {
        match event {
            // Event::Command(open) if open.is(OPEN_SELECTOR) => {
            //     // I don't know if this is right
            //     // if I don't return here, the application crashes everytime
            //     // I close it because of unwrap() and can't find selector
            //     // is the command being sent periodically?
            //     let payload: &FileInfo = open.get_unchecked(Selector::new(
            //         "druid-builtin.open-file-path",
            //     ));

            //     let path = payload.path();
            //     let sink = ctx.get_external_handle();
            //     read_images(sink, path.to_owned());
            // }
            Event::Command(select_image)
                if select_image.is(SELECT_IMAGE_SELECTOR) =>
            {
                let index = select_image.get_unchecked(SELECT_IMAGE_SELECTOR);
                data.selected = *index;
            }
            // Event::Command(paths) if paths.is(FINISHED_READING_FOLDER) => {
            //     let (paths, thumbnails) =
            //         paths.get_unchecked(FINISHED_READING_FOLDER).clone();
            //     data.images = paths;
            //     data.current_image_idx = 0;
            //     data.thumbnails = thumbnails;
            // }
            // Event::Command(selector) if selector.is(CREATED_THUMBNAIL) => {
            //     let thumbnail = selector.get_unchecked(CREATED_THUMBNAIL);
            //     data.thumbnails[thumbnail.index] = thumbnail.clone();
            // }
            _ => (),
        }
        child.event(ctx, event, data, env);
    }
}
// pub struct AppStateController;

// impl Controller<AppState, Container<AppState>> for AppStateController {
//     fn event(
//         &mut self,
//         child: &mut Container<AppState>,
//         ctx: &mut druid::EventCtx,
//         event: &Event,
//         data: &mut AppState,
//         env: &Env,
//     ) {
//         match event {
//             Event::Command(open) if open.is(OPEN_SELECTOR) => {
//                 // I don't know if this is right
//                 // if I don't return here, the application crashes everytime
//                 // I close it because of unwrap() and can't find selector
//                 // is the command being sent periodically?
//                 let payload: &FileInfo = open.get_unchecked(Selector::new(
//                     "druid-builtin.open-file-path",
//                 ));

//                 let path = payload.path();
//                 let sink = ctx.get_external_handle();
//                 read_images(sink, path.to_owned());
//             }
//             Event::Command(select_image)
//                 if select_image.is(SELECT_IMAGE_SELECTOR) =>
//             {
//                 let index = select_image.get_unchecked(SELECT_IMAGE_SELECTOR);
//                 data.current_image_idx = *index;
//             }
//             Event::Command(paths) if paths.is(FINISHED_READING_FOLDER) => {
//                 let (paths, thumbnails) =
//                     paths.get_unchecked(FINISHED_READING_FOLDER).clone();
//                 data.images = paths;
//                 data.current_image_idx = 0;
//                 data.thumbnails = thumbnails;
//             }
//             Event::Command(selector) if selector.is(CREATED_THUMBNAIL) => {
//                 let thumbnail = selector.get_unchecked(CREATED_THUMBNAIL);
//                 data.thumbnails[thumbnail.index] = thumbnail.clone();
//             }
//             _ => (),
//         }
//         child.event(ctx, event, data, env);
//     }
// }

// impl ListIter<(usize, Thumbnail)> for AppState {
//     fn for_each(&self, mut cb: impl FnMut(&(usize, Thumbnail), usize)) {
//         for (i, item) in self.thumbnails.iter().enumerate() {
//             cb(&(self.current_image_idx, item.to_owned()), i);
//         }
//     }

//     fn for_each_mut(
//         &mut self,
//         mut cb: impl FnMut(&mut (usize, Thumbnail), usize),
//     ) {
//         for (i, item) in self.thumbnails.iter().enumerate() {
//             let owned_item = item.to_owned();
//             cb(&mut (self.current_image_idx, owned_item.clone()), i);
//         }
//     }

//     fn data_len(&self) -> usize {
//         self.thumbnails.len()
//     }
// }

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
        ctx: &mut druid::LifeCycleCtx,
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
        ctx: &mut druid::UpdateCtx,
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
pub struct FolderThumbnailController;
impl Controller<(Thumbnail, usize), Image> for FolderThumbnailController {
    fn event(
        &mut self,
        child: &mut Image,
        ctx: &mut druid::EventCtx,
        event: &Event,
        data: &mut (Thumbnail, usize),
        env: &Env,
    ) {
        child.event(ctx, event, data, env)
    }
    fn lifecycle(
        &mut self,
        child: &mut Image,
        ctx: &mut druid::LifeCycleCtx,
        event: &LifeCycle,
        data: &(Thumbnail, usize),
        env: &Env,
    ) {
        if let LifeCycle::WidgetAdded = event {
            child.set_image_data(data.0.image.clone());
            ctx.request_layout();
            ctx.request_paint();
        }
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut Image,
        ctx: &mut druid::UpdateCtx,
        old_data: &(Thumbnail, usize),
        data: &(Thumbnail, usize),
        env: &Env,
    ) {
        if !data.same(old_data) {
            child.set_image_data(data.0.image.clone());
            ctx.request_layout();
            ctx.request_paint();
        }
        child.update(ctx, old_data, data, env)
    }
    // fn lifecycle(
    //     &mut self,
    //     child: &mut Image,
    //     ctx: &mut druid::LifeCycleCtx,
    //     event: &LifeCycle,
    //     data: &Thumbnail,
    //     env: &Env,
    // ) {
    //     if let LifeCycle::WidgetAdded = event {
    //         child.set_image_data(data.image.clone());
    //         ctx.request_layout();
    //         ctx.request_paint();
    //     }
    //     child.lifecycle(ctx, event, data, env)
    // }

    // fn update(
    //     &mut self,
    //     child: &mut Image,
    //     ctx: &mut druid::UpdateCtx,
    //     old_data: &Thumbnail,
    //     data: &Thumbnail,
    //     env: &Env,
    // ) {
    //     let old_image = old_data;
    //     let current_image = data;
    //     if !current_image.same(old_image) {
    //         child.set_image_data(current_image.image.clone());
    //         ctx.request_layout();
    //         ctx.request_paint();
    //     }
    //     child.update(ctx, old_data, data, env)
    // }
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

#[derive(Debug, Clone, Data, Lens)]
pub struct ImageFolder {
    pub name: String,
    pub paths: Vector<Arc<PathBuf>>,
    pub selected: Option<usize>,
    pub thumbnails: Vector<Thumbnail>,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct ImageViewState {
    pub name: String,
    pub paths: Vector<Arc<PathBuf>>,
    pub selected: usize,
    // pub thumbnails: Vector<Thumbnail>,
}
impl ImageViewState {
    pub fn new(state: AppState) -> Self {
        if let Some(idx) = state.selected_folder {
            Self {
                name: state.all_images[idx].name.clone(),
                paths: state.all_images[idx].paths.clone(),
                selected: 0,
                // thumbnails: state.all_images[idx].thumbnails,
            }
        } else {
            Self {
                name: "".to_string(),
                paths: Vector::new(),
                selected: 0,
                // thumbnails: Vector::new(),
            }
        }
    }
}

pub struct ImageViewTransfer;
impl ScopeTransfer for ImageViewTransfer {
    type In = AppState;

    type State = ImageViewState;

    fn read_input(&self, state: &mut Self::State, inner: &Self::In) {
        if let Some(idx) = inner.selected_folder {
            let folder = &inner.all_images[idx];
            state.name = folder.name.clone();
            // state.selected = folder.selected.unwrap_or(0);
            // state.thumbnails = folder.thumbnails.clone();
            state.paths = folder.paths.clone();
        } else {
            state.name = "".to_string();
            // state.thumbnails = Vector::new();
            // state.selected = 0;
            state.paths = Vector::new();
        }
    }

    fn write_back_input(&self, state: &Self::State, inner: &mut Self::In) {
        // if let Some(idx) = state.selected_folder {
        //     inner.all_images[idx].name = state.name.clone();
        //     inner.all_images[idx].thumbnails = state.images.clone();
        // }
        // else {
        //     dbg!("This should do nothing because there is no state to write back.");
        // }
        // dbg!("This should do nothing because there is no state to write back.");
    }
}

#[derive(Debug, Clone, Data, Lens)]
pub struct FolderGalleryState {
    pub name: String,
    pub images: Vector<Thumbnail>,
    pub selected_folder: Option<usize>,
    pub selected_image: usize,
}
impl FolderGalleryState {
    pub fn new(state: AppState) -> Self {
        if let Some(idx) = state.selected_folder {
            Self {
                name: state.all_images[idx].name.clone(),
                images: state.all_images[idx].thumbnails.clone(),
                selected_folder: Some(idx),
                selected_image: 0,
            }
        } else {
            Self {
                name: "".to_string(),
                images: Vector::new(),
                selected_folder: None,
                selected_image: 0,
            }
        }
    }
}

pub struct FolderViewController;
impl Controller<FolderGalleryState, Container<FolderGalleryState>>
    for FolderViewController
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
            Event::Command(selector) if selector.is(GALLERY_SELECTED_IMAGE) => {
                let idx = selector.get_unchecked(GALLERY_SELECTED_IMAGE);
                data.selected_image = *idx;
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut Container<FolderGalleryState>,
        ctx: &mut druid::LifeCycleCtx,
        event: &LifeCycle,
        data: &FolderGalleryState,
        env: &Env,
    ) {
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut Container<FolderGalleryState>,
        ctx: &mut druid::UpdateCtx,
        old_data: &FolderGalleryState,
        data: &FolderGalleryState,
        env: &Env,
    ) {
        child.update(ctx, old_data, data, env)
    }
}

impl GridIter<(Thumbnail, usize)> for FolderGalleryState {
    fn for_each(&self, mut cb: impl FnMut(&(Thumbnail, usize), usize)) {
        for (i, thumbnail) in self.images.iter().enumerate() {
            cb(&(thumbnail.clone(), i), i);
        }
    }

    fn for_each_mut(
        &mut self,
        mut cb: impl FnMut(&mut (Thumbnail, usize), usize),
    ) {
        for (i, thumbnail) in self.images.iter_mut().enumerate() {
            cb(&mut (thumbnail.clone(), i), i);
        }
    }

    fn data_len(&self) -> usize {
        self.images.len()
    }

    fn child_data(&self) -> Option<(Thumbnail, usize)> {
        match self.images.iter().next() {
            Some(thumbnail) => Some((thumbnail.clone(), 0)),
            None => {
                let thumbnail = Thumbnail {
                    index: 0,
                    image: ImageBuf::empty(),
                };
                Some((thumbnail, 0))
            }
        }
    }
}

pub struct GalleryTransfer;

impl ScopeTransfer for GalleryTransfer {
    type In = AppState;

    type State = FolderGalleryState;

    fn read_input(&self, state: &mut Self::State, inner: &Self::In) {
        if let Some(idx) = inner.selected_folder {
            state.selected_folder = Some(idx);
            state.name = inner.all_images[idx].name.clone();
            state.images = inner.all_images[idx].thumbnails.clone();
        } else {
            state.name = "".to_string();
            state.images = Vector::new();
        }
    }

    fn write_back_input(&self, state: &Self::State, inner: &mut Self::In) {
        if let Some(idx) = state.selected_folder {
            inner.all_images[idx].name = state.name.clone();
            inner.all_images[idx].thumbnails = state.images.clone();
        } else {
            dbg!("This should do nothing because there is no state to write back.");
        }
    }
}
