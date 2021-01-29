use std::{
    fs::read_dir,
    path::{Path, PathBuf},
    sync::{
        mpsc::{sync_channel, Receiver, SyncSender},
        Arc,
    },
    thread,
};

use druid::{
    commands::OPEN_FILE,
    im::{vector, HashSet, Vector},
    piet::ImageFormat,
    widget::{Container, Controller, Image, ScopeTransfer},
    Data, Env, Event, ExtEventSink, ImageBuf, Lens, LifeCycle, LifeCycleCtx,
    Target, UpdateCtx, Widget, WidgetId,
};
use druid_gridview::GridIter;
use druid_navigator::navigator::{View, ViewController};
use image::{imageops::thumbnail, io::Reader, ImageError, RgbImage};
use log::error;
use walkdir::WalkDir;

use crate::{
    app_commands::{
        CREATED_FIRST_IMAGE_THUMBNAIL, CREATED_THUMBNAIL,
        FINISHED_READING_ALL_PATHS, FINISHED_READING_FOLDER_IMAGE,
        FINISHED_READING_IMAGE, POP_FOLDER_VIEW, POP_VIEW,
        PUSH_VIEW_WITH_SELECTED_IMAGE, SELECTED_FOLDER, SELECT_IMAGE_SELECTOR,
    },
    view::check_folder_has_images,
};

#[derive(Clone, Data, Lens, Debug)]
pub struct AppState {
    // pub images: Arc<Vec<PathBuf>>,
    pub images: HashSet<Arc<PathBuf>>,
    pub current_image_idx: usize,
    pub thumbnails: Vector<Thumbnail>,
    pub views: Vector<AppView>,
    pub all_images: Vector<ImageFolder>,
    pub selected_folder: Option<usize>,
    // pub test_text: Vector<String>,
}

// impl ListIter<(ImageFolder, usize)> for AppState {
//     fn for_each(&self, mut cb: impl FnMut(&(ImageFolder, usize), usize)) {
//         for (i, image_folder) in self.all_images.iter().enumerate() {
//             cb(&(image_folder.clone(), i), i)
//         }
//     }

//     fn for_each_mut(
//         &mut self,
//         mut cb: impl FnMut(&mut (ImageFolder, usize), usize),
//     ) {
//         for (i, image_folder) in self.all_images.iter_mut().enumerate() {
//             cb(&mut (image_folder.clone(), i), i)
//         }
//     }

//     fn data_len(&self) -> usize {
//         self.all_images.len()
//     }
// }
impl GridIter<(ImageFolder, usize)> for AppState {
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

    fn child_data(&self) -> Option<(ImageFolder, usize)> {
        match self.all_images.iter().next() {
            Some(folder) => Some((folder.clone(), 0)),
            None => Some((
                ImageFolder {
                    name: Arc::new(PathBuf::from("".to_owned())),
                    paths: Vector::new(),
                    selected: None,
                    thumbnails: Vector::new(),
                },
                0,
            )),
        }
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
                if selector.is(FINISHED_READING_FOLDER_IMAGE) =>
            {
                let image_folder =
                    selector.get_unchecked(FINISHED_READING_FOLDER_IMAGE);
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
            Event::Command(cmd) if cmd.is(FINISHED_READING_ALL_PATHS) => {
                let current_folders =
                    cmd.get_unchecked(FINISHED_READING_ALL_PATHS);
                data.images = current_folders.clone();
                let handle = ctx.get_external_handle();
                let folders = data.all_images.clone();
                thread::spawn(move || {
                    for (folder_idx, folder) in folders.iter().enumerate() {
                        let thumbnail =
                            create_first_image_thumbnail(folder).unwrap();
                        handle
                            .submit_command(
                                CREATED_FIRST_IMAGE_THUMBNAIL,
                                (thumbnail, folder_idx),
                                Target::Auto,
                            )
                            .unwrap();
                    }
                });
            }
            Event::Command(cmd) if cmd.is(CREATED_FIRST_IMAGE_THUMBNAIL) => {
                let (thumbnail, folder_idx) =
                    cmd.get_unchecked(CREATED_FIRST_IMAGE_THUMBNAIL);
                data.all_images[*folder_idx].thumbnails[0] = thumbnail.clone();
            }
            Event::Command(cmd) if cmd.is(OPEN_FILE) => {
                let file_info = cmd.get_unchecked(OPEN_FILE);
                // dbg!(file_info.path());
                let handle = ctx.get_external_handle();
                let folders = data.images.clone();
                flatten_and_add_paths(
                    file_info.path().to_path_buf(),
                    folders,
                    handle,
                );
            }
            // Event::Command(selector) if selector.is(PUSH_VIEW) => {
            //     let view = selector.get_unchecked(PUSH_VIEW);
            //     data.add_view(view.clone());
            // }
            _ => {}
        }
        child.event(ctx, event, data, env)
    }
}
fn flatten_and_add_paths(
    path: PathBuf,
    mut current_folders: HashSet<Arc<PathBuf>>,
    handle: ExtEventSink,
) {
    thread::spawn(move || {
        // if current_folders.is_empty() {
        //     return;
        // }
        dbg!(&path);
        let entries = WalkDir::new(path).into_iter().filter_entry(|entry| {
            // only walks directories, not files, and only keeps directories
            // that don't fail to read and are not empty
            if entry.path().is_dir() {
                match read_dir(entry.path()) {
                    Ok(mut dir) => dir.next().is_some(),
                    Err(_) => false,
                }
            } else {
                false
            }
        });
        for (_i, entry) in entries.enumerate() {
            let entry = entry.unwrap();
            let current_folder = entry.path().to_path_buf();
            // checks if this directory has already been added previously
            // mostly dealing with if you add a directory that was the child of another directory
            // you've added
            if current_folders.contains(&current_folder) {
                continue;
            }
            let (thumbnails, paths) = check_folder_has_images(&entry);
            dbg!(&current_folder);
            dbg!(thumbnails.len());
            if !thumbnails.is_empty() {
                current_folders.insert(Arc::new(current_folder.clone()));
                let image_folder = ImageFolder {
                    paths,
                    thumbnails,
                    name: Arc::new(current_folder),
                    selected: None,
                };
                handle
                    .submit_command(
                        FINISHED_READING_FOLDER_IMAGE,
                        image_folder,
                        Target::Auto,
                    )
                    .unwrap();
            }
        }
        handle
            .submit_command(
                FINISHED_READING_ALL_PATHS,
                current_folders,
                Target::Auto,
            )
            .unwrap();
    });
}
fn create_first_image_thumbnail(
    folder: &ImageFolder,
) -> Result<Thumbnail, ImageError> {
    let image_path = folder.paths[0].clone();
    let image = Reader::open(image_path.as_ref())?
        .with_guessed_format()?
        .decode()?
        .to_rgb8();
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

    Ok(Thumbnail { index: 0, image })
}
pub struct ImageViewController;

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
                data.selected_image = *index;
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
        ctx: &mut LifeCycleCtx,
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
        ctx: &mut UpdateCtx,
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

#[derive(Debug, Clone, Data, Lens)]
pub struct ImageFolder {
    pub name: Arc<PathBuf>,
    pub paths: Vector<Arc<PathBuf>>,
    pub selected: Option<usize>,
    pub thumbnails: Vector<Thumbnail>,
}

#[derive(Debug, Clone, Data, Lens)]
pub struct FolderGalleryState {
    pub name: Arc<PathBuf>,
    pub images: Vector<Thumbnail>,
    pub selected_folder: Option<usize>,
    pub selected_image: usize,
    pub views: Vector<FolderView>,
    pub paths: Vector<Arc<PathBuf>>,
}
impl FolderGalleryState {
    pub fn new(state: AppState) -> Self {
        if let Some(idx) = state.selected_folder {
            Self {
                name: state.all_images[idx].name.clone(),
                images: state.all_images[idx].thumbnails.clone(),
                selected_folder: Some(idx),
                selected_image: 0,
                views: vector![FolderView::Folder],
                paths: state.all_images[idx].paths.clone(),
            }
        } else {
            Self {
                name: Arc::new(PathBuf::from("".to_string())),
                images: Vector::new(),
                selected_folder: None,
                selected_image: 0,
                views: vector![FolderView::Folder],
                paths: Vector::new(),
            }
        }
    }
}
impl ViewController<FolderView> for FolderGalleryState {
    fn add_view(&mut self, view: FolderView) {
        self.views.push_back(view);
    }

    fn pop_view(&mut self) {
        self.views.pop_back();
    }

    fn current_view(&self) -> &FolderView {
        let last = self.views.len() - 1;
        &self.views[last]
    }

    fn len(&self) -> usize {
        self.views.len()
    }

    fn is_empty(&self) -> bool {
        self.views.is_empty()
    }
}
#[derive(Debug, Data, Clone, PartialEq, Eq, Hash)]
pub enum FolderView {
    Folder,
    SingleImage,
}
impl View for FolderView {}

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
            Event::Command(selector)
                if selector.is(PUSH_VIEW_WITH_SELECTED_IMAGE) =>
            {
                let (view, idx) =
                    selector.get_unchecked(PUSH_VIEW_WITH_SELECTED_IMAGE);
                data.add_view(view.clone());
                data.selected_image = *idx;
            }
            // Event::Command(cmd) if cmd.is(FINISHED_READING_IMAGE_FOLDER) => {
            //     let thumbnails =
            //         cmd.get_unchecked(FINISHED_READING_IMAGE_FOLDER);
            //     data.images = thumbnails.clone();
            // }
            Event::Command(cmd) if cmd.is(CREATED_THUMBNAIL) => {
                let thumbnail = cmd.get_unchecked(CREATED_THUMBNAIL);
                // TODO: this isn't workign correctly because this can be out of bounds
                data.images[thumbnail.index] = thumbnail.clone();
            }
            Event::Command(selector) if selector.is(POP_FOLDER_VIEW) => {
                // let view = selector.get_unchecked(POP_FOLDER_VIEW);
                data.pop_view();
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut Container<FolderGalleryState>,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &FolderGalleryState,
        env: &Env,
    ) {
        child.lifecycle(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut Container<FolderGalleryState>,
        ctx: &mut UpdateCtx,
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
        match inner.selected_folder {
            Some(idx) => {
                if let Some(current_idx) = state.selected_folder {
                    if idx != current_idx {
                        let folder = &inner.all_images[idx];
                        dbg!("Change Folder", &folder.name);
                        state.selected_folder = Some(idx);
                        state.name = folder.name.clone();
                        state.images = folder.thumbnails.clone();
                        state.paths = folder.paths.clone();
                    }
                } else {
                    let folder = &inner.all_images[idx];
                    dbg!("None", &folder.name);
                    state.selected_folder = Some(idx);
                    state.name = folder.name.clone();
                    state.images = folder.thumbnails.clone();
                    state.paths = folder.paths.clone();
                }
            }
            None => {
                dbg!("Nothing should be read or maybe it should");
            }
        }
        // if let Some(idx) = inner.selected_folder {
        //     let folder = &inner.all_images[idx];
        //     state.selected_folder = Some(idx);
        //     state.name = folder.name.clone();
        //     state.images = folder.thumbnails.clone();
        //     state.paths = folder.paths.clone();
        // } else {
        //     state.name = "".to_string();
        //     state.images = Vector::new();
        // }
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
pub struct DisplayImageController {
    sender: SyncSender<RgbImage>,
    receiver: Receiver<RgbImage>,
}
impl DisplayImageController {
    pub fn new() -> Self {
        let (sender, receiver) = sync_channel(3);

        DisplayImageController { sender, receiver }
    }

    fn read_image(
        &self,
        sink: ExtEventSink,
        path: PathBuf,
        widget_id: WidgetId,
    ) {
        let sender = self.sender.clone();
        std::thread::spawn(move || {
            let image = image::io::Reader::open(path)
                .unwrap()
                // .with_guessed_format()
                // .unwrap()
                .decode()
                .unwrap()
                .into_rgb8();
            sender.send(image).unwrap();
            sink.submit_command(FINISHED_READING_IMAGE, (), widget_id)
                .unwrap();
        });
    }
}
impl Controller<FolderGalleryState, Image> for DisplayImageController {
    fn event(
        &mut self,
        child: &mut Image,
        ctx: &mut druid::EventCtx,
        event: &Event,
        data: &mut FolderGalleryState,
        env: &Env,
    ) {
        match event {
            Event::Command(image_selector)
                if image_selector.is(FINISHED_READING_IMAGE) =>
            {
                let image = self.receiver.recv().unwrap();
                let (width, height) = image.dimensions();
                let image = ImageBuf::from_raw(
                    image.into_raw(),
                    ImageFormat::Rgb,
                    width as usize,
                    height as usize,
                );
                child.set_image_data(image);
                ctx.request_layout();
                ctx.request_paint();
            }
            _ => (),
        }
        child.event(ctx, event, data, env)
    }

    fn update(
        &mut self,
        child: &mut Image,
        ctx: &mut UpdateCtx,
        old_data: &FolderGalleryState,
        data: &FolderGalleryState,
        env: &Env,
    ) {
        if data.paths.is_empty() {
            return;
        }
        if data.selected_image != old_data.selected_image
        // || data.paths != old_data.paths
        {
            let path = data.paths[data.selected_image].as_ref().clone();
            let sink = ctx.get_external_handle();
            // only need to send this payload back to itself
            // after it finishes reading the image on a separate thread
            // only DisplayImageController needs to see this payload
            self.read_image(sink, path, ctx.widget_id());
            ctx.request_layout();
            ctx.request_paint();
        }
        child.update(ctx, old_data, data, env)
    }

    fn lifecycle(
        &mut self,
        child: &mut Image,
        ctx: &mut LifeCycleCtx,
        event: &LifeCycle,
        data: &FolderGalleryState,
        env: &Env,
    ) {
        // TODO: doing this is currently not really safe, though
        // not problematic. Druid warns because this might send an event
        // back here, to read the image, before it gets laid out
        if let LifeCycle::WidgetAdded = event {
            let path = data.paths[data.selected_image].as_ref().clone();
            let sink = ctx.get_external_handle();
            // only need to send this payload back to itself
            // after it finishes reading the image on a separate thread
            // only DisplayImage needs to see this payload
            self.read_image(sink, path, ctx.widget_id());
        }
        child.lifecycle(ctx, event, data, env)
    }
}
