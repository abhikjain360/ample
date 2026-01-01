use std::{
    collections::{HashMap, HashSet, VecDeque},
    fmt,
    path::PathBuf,
    sync::RwLock,
    time::Duration,
};

use lofty::{
    file::{AudioFile, TaggedFile, TaggedFileExt},
    probe::Probe,
    tag::Accessor,
};
use serde::Serialize;

use crate::settings::SettingsState;

#[derive(Default, Debug)]
pub struct Library {
    arena: HashMap<usize, DirTree>,
    pub(crate) files: Vec<File>,
}

impl Library {
    pub async fn walker(path: PathBuf) -> Result<Self, std::io::Error> {
        let mut me = Self::default();

        let root = DirTree::new(path);
        me.arena.insert(0, root);

        let mut to_explore = VecDeque::from([0]);
        let mut visited = HashSet::new();
        let mut children = vec![];

        while let Some(current) = to_explore.pop_front() {
            let path = &me.arena[&current].path;
            if visited.contains(path) {
                continue;
            }
            visited.insert(path.clone());

            let mut read_dir = tokio::fs::read_dir(path).await?;

            while let Some(dir_entry) = read_dir.next_entry().await? {
                let path = dir_entry.path();
                let id = me.arena.len();

                if path.is_dir() {
                    me.arena.insert(id, DirTree::new(path));
                    to_explore.push_back(id);
                    continue;
                }

                if !path.is_file() {
                    continue;
                }

                let file_path = path.clone();
                let Ok(Ok(file)) = tokio::task::spawn_blocking(move || File::new(file_path))
                    .await
                    .inspect_err(|e| {
                        log::error!("metadata read job ended unsuccessfully for file {path:?}: {e}")
                    })
                else {
                    continue;
                };

                me.files.push(file);
                children.push(path);
            }

            std::mem::swap(
                &mut me.arena.get_mut(&current).unwrap().children,
                &mut children,
            );
        }

        me.files.sort_by(|a, b| a.path.cmp(&b.path));

        Ok(me)
    }
}

#[derive(Debug)]
pub(crate) struct DirTree {
    pub(crate) path: PathBuf,
    pub(crate) children: Vec<PathBuf>,
}

impl DirTree {
    pub(crate) fn new(path: PathBuf) -> Self {
        DirTree {
            path,
            children: vec![],
        }
    }
}

pub(crate) struct File {
    pub(crate) path: PathBuf,
    pub(crate) metadata: TaggedFile,
}

impl fmt::Debug for File {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("File").field("path", &self.path).finish()
    }
}

#[derive(Debug, thiserror::Error)]
pub enum FileInitError {
    #[error("failed to read metadata: {0}")]
    Lofty(#[from] lofty::error::LoftyError),
}

impl File {
    pub fn new(path: PathBuf) -> Result<Self, FileInitError> {
        let metadata = Probe::open(&path)?.read()?;
        Ok(Self { path, metadata })
    }
}

pub type LibraryState<'a> = tauri::State<'a, RwLock<Option<Library>>>;

#[tauri::command]
pub async fn library_open(
    path: String,
    library: LibraryState<'_>,
    settings: SettingsState<'_>,
) -> crate::Result<()> {
    let path: PathBuf = path.into();
    let new_library = Library::walker(path.clone()).await?;
    library.write().unwrap().replace(new_library);

    let mut settings = settings.write().unwrap();
    settings.libraries.retain(|p| {
        p.to_string_lossy().to_ascii_lowercase() != path.to_string_lossy().to_ascii_lowercase()
    });
    settings.libraries.push_front(path);
    if let Err(e) = settings.save() {
        log::error!("failed to save settings: {}", e);
    };

    Ok(())
}

#[derive(Debug, Serialize)]
pub struct SongData {
    id: usize,
    title: String,
    artist: Option<String>,
    duration: (u64, u64),
}

impl SongData {
    pub fn new(id: usize, title: String, duration: Duration) -> Self {
        Self {
            id,
            title,
            artist: None,
            duration: (duration.as_secs() / 60, duration.as_secs() % 60),
        }
    }
}

#[tauri::command]
pub fn library_list_songs(library: LibraryState<'_>) -> Vec<SongData> {
    let library = library.read().unwrap();
    let Some(library) = library.as_ref() else {
        return vec![];
    };
    library
        .files
        .iter()
        .enumerate()
        .map(|(id, file)| {
            let mut data = SongData::new(
                id,
                file.path
                    .file_name()
                    .unwrap_or_else(|| file.path.as_os_str())
                    .to_string_lossy()
                    .to_string(),
                file.metadata.properties().duration(),
            );

            let Some(tag) = file.metadata.primary_tag() else {
                return data;
            };

            if let Some(title) = tag.title() {
                data.title = title.to_string();
            }

            if let Some(artist) = tag.artist() {
                data.artist = Some(artist.to_string());
            }

            data
        })
        .collect()
}
