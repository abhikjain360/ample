use std::{
    collections::{HashMap, HashSet, VecDeque},
    path::PathBuf,
    sync::Arc,
};

pub(crate) use dirtree::DirTree;
pub(crate) use file::File;

mod dirtree;
mod file;

#[derive(Default, Debug)]
pub(crate) struct Library {
    arena: HashMap<usize, DirTree>,
    pub(crate) files: Vec<File>,
}

impl Library {
    pub(crate) async fn walker(path: PathBuf) -> Result<Arc<Self>, crate::IoError> {
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
                        tracing::error!(
                            "metadata read job ended unsuccessfully for file {path:?}: {e}"
                        )
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

        Ok(Arc::new(me))
    }
}
