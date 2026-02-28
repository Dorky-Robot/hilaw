use std::path::PathBuf;
use std::sync::Arc;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    data_dir: PathBuf,
}

impl AppState {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            inner: Arc::new(AppStateInner { data_dir }),
        }
    }

    pub fn images_dir(&self) -> PathBuf {
        self.inner.data_dir.join("images")
    }

    pub fn image_dir(&self, id: &str) -> PathBuf {
        self.images_dir().join(id)
    }
}
