use std::{path::PathBuf, sync::Arc};

use druid::{
    im::{HashSet, Vector},
    Selector,
};

use crate::data::{AppState, FolderView, ImageFolder, Thumbnail};

pub const SELECTED_FOLDER: Selector<usize> =
    Selector::new("app.selected-folder");
pub const FINISHED_READING_ALL_PATHS: Selector<HashSet<Arc<PathBuf>>> =
    Selector::new("app.finished-reading-all-paths");

pub const FINISHED_READING_FOLDER_IMAGE: Selector<ImageFolder> =
    Selector::new("finished_reading_image_folder");
pub const POP_VIEW: Selector<()> = Selector::new("app.pop-view");
pub const POP_FOLDER_VIEW: Selector<()> = Selector::new("app.pop-folder-view");
pub const PUSH_VIEW_WITH_SELECTED_IMAGE: Selector<(FolderView, usize)> =
    Selector::new("app.push-view-with-selected-image");
pub const FINISHED_READING_IMAGE_FOLDER: Selector<Vector<Thumbnail>> =
    Selector::new("app.finished-reading-image-folder");

// pub const GALLERY_SELECTED_IMAGE: Selector<usize> =
// Selector::new("app.gallery-view.selected-image");

pub const OPEN_SELECTOR: Selector<AppState> =
    Selector::new("druid-builtin.open-file-path");
pub const SELECT_IMAGE_SELECTOR: Selector<usize> =
    Selector::new("select_thumbnail");
pub const FINISHED_READING_FOLDER: Selector<(
    Arc<Vec<PathBuf>>,
    Vector<Thumbnail>,
)> = Selector::new("finish_reading_folder");
pub const FINISHED_READING_IMAGE: Selector<()> =
    Selector::new("finished_reading_image");
pub const CREATED_THUMBNAIL: Selector<Thumbnail> =
    Selector::new("created_thumbnail");
pub const CREATED_FIRST_IMAGE_THUMBNAIL: Selector<(Thumbnail, usize)> =
    Selector::new("app.created-first-image-thumbnail");
