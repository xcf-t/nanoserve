use std::time::SystemTime;
use async_std::path::PathBuf;
use async_std::stream::StreamExt;
use serde::{Deserialize, Serialize};
use tide::{Request, StatusCode};
use clean_path::Clean;
use crate::State;

#[derive(Serialize)]
struct FileInfo {
    name: String,
    size: u64,
    dir: bool,
    sym: bool,
    modified: Option<u64>,
}

async fn get_entries(path: &PathBuf) -> tide::Result<Vec<FileInfo>> {
    let mut readdir = path.read_dir().await?;

    let mut result = Vec::new();

    while let Some(entry) = readdir.next().await {
        let entry = match entry {
            Ok(entry) => entry,
            Err(_) => continue,
        };

        let meta = entry.metadata().await?;
        let modified = meta.modified().ok().map(|time| {
            time.duration_since(SystemTime::UNIX_EPOCH).map(|t| t.as_secs()).ok()
        }).flatten();

        result.push(FileInfo {
            name: entry.file_name().to_string_lossy().to_string(),
            size: meta.len(),
            dir: meta.file_type().is_dir(),
            modified,
            sym: meta.file_type().is_symlink(),
        });
    }

    Ok(result)
}

#[derive(Deserialize)]
struct PathQuery {
    path: Option<String>
}

pub async fn list_files(req: Request<State>) -> tide::Result {
    let query = req.query::<PathQuery>()?;
    let mut path = req.state().file_path.clone();
    if let Some(p) = query.path {
        let buf = std::path::PathBuf::from(p);
        if !buf.is_relative() {
            return Ok(StatusCode::BadRequest.into())
        }

        let p = path.join(Clean::clean(&buf));

        path = PathBuf::from(p);
    }

    let entries = get_entries(&path).await?;

    Ok(serde_json::value::to_value(entries)?.into())
}