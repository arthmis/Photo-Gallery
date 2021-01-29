use std::{path::PathBuf, sync::Arc};

use druid::{im::HashSet, Selector};

use crate::{
    app_data::{ImageFolder, Thumbnail},
    folder_view::FolderView,
};

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

pub const SELECT_IMAGE_SELECTOR: Selector<usize> =
    Selector::new("select_thumbnail");

pub const FINISHED_READING_IMAGE: Selector<()> =
    Selector::new("finished_reading_image");

pub const CREATED_THUMBNAIL: Selector<Thumbnail> =
    Selector::new("created_thumbnail");

pub const CREATED_FIRST_IMAGE_THUMBNAIL: Selector<(Thumbnail, usize)> =
    Selector::new("app.created-first-image-thumbnail");
