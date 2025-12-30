use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::PathBuf,
    sync::{Arc, RwLock},
};

pub(crate) use dirtree::DirTree;
pub(crate) use file::File;

mod dirtree;
mod file;

#[derive(Default, Debug)]
pub(crate) struct Library {
    arena: HashMap<usize, DirTree>,
    files: HashMap<PathBuf, File>,
}

impl Library {
    pub(crate) async fn walker(path: PathBuf) -> Result<Arc<RwLock<Self>>, crate::IoError> {
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
                    continue;
                }

                if !path.is_file() {
                    continue;
                }

                let file_path = path.clone();
                let Ok(Ok(file)) = tokio::task::spawn_blocking(move || File::new(file_path)).await
                else {
                    continue;
                };

                me.files.insert(path.clone(), file);
                children.push(path);
            }

            std::mem::swap(
                &mut me.arena.get_mut(&current).unwrap().children,
                &mut children,
            );
        }

        Ok(Arc::new(RwLock::new(me)))
    }
}
