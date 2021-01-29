use std::{
    fs::{self, read_dir},
    path::PathBuf,
    sync::Arc,
    thread,
};

use druid::{
    commands::OPEN_FILE,
    im::{HashSet, Vector},
    piet::ImageFormat,
    widget::{Container, Controller},
    Data, Env, Event, ExtEventSink, ImageBuf, Target, Widget,
};
use druid_gridview::GridIter;
use druid_navigator::navigator::{View, ViewController};
use image::{imageops::thumbnail, io::Reader, ImageError};
use log::error;
use walkdir::{DirEntry, WalkDir};

use crate::{
    app_commands::{
        CREATED_FIRST_IMAGE_THUMBNAIL, FINISHED_READING_ALL_PATHS,
        FINISHED_READING_FOLDER_IMAGE, POP_VIEW, SELECTED_FOLDER,
    },
    app_data::{AppState, ImageFolder, Thumbnail},
};

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
                let handle = ctx.get_external_handle();
                let folders = data.images.clone();
                flatten_and_add_paths(
                    file_info.path().to_path_buf(),
                    folders,
                    handle,
                );
            }
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

fn check_folder_has_images(
    entry: &DirEntry,
) -> (Vector<Thumbnail>, Vector<Arc<PathBuf>>) {
    let mut images = Vector::new();
    let mut paths = Vector::new();
    let entries = fs::read_dir(entry.path()).unwrap();
    for file in entries {
        let file = file.unwrap();
        if file.path().is_file() {
            match Reader::open(file.path()) {
                Ok(image) => match image.format() {
                    Some(image::ImageFormat::Png)
                    | Some(image::ImageFormat::Jpeg) => {}
                    Some(_) | None => continue,
                },
                Err(err) => {
                    error!("Error opening file: {}", err);
                    continue;
                }
            };
            images.push_back(Thumbnail {
                index: images.len(),
                image: ImageBuf::empty(),
            });
            paths.push_back(Arc::new(file.path().to_path_buf()));
        }
    }
    (images, paths)
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
