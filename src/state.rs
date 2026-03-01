use std::path::PathBuf;
use std::sync::Arc;

use crate::salita_client::SalitaClient;

#[derive(Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

struct AppStateInner {
    data_dir: PathBuf,
    salita: SalitaClient,
}

impl AppState {
    pub fn new(data_dir: PathBuf, salita_url: &str) -> Self {
        Self {
            inner: Arc::new(AppStateInner {
                data_dir,
                salita: SalitaClient::new(salita_url),
            }),
        }
    }

    pub fn images_dir(&self) -> PathBuf {
        self.inner.data_dir.join("images")
    }

    pub fn image_dir(&self, id: &str) -> PathBuf {
        self.images_dir().join(id)
    }

    pub fn cache_dir(&self) -> PathBuf {
        self.inner.data_dir.join("cache")
    }

    pub fn salita(&self) -> &SalitaClient {
        &self.inner.salita
    }
}
