use std::io::SeekFrom;
use std::ops::DerefMut;
use std::sync::Arc;
use async_std::fs::File;
use async_std::io::prelude::SeekExt;
use async_std::io::ReadExt;
use async_std::sync::Mutex;
use dashmap::DashMap;
use getrandom::getrandom;
use lazy_static::lazy_static;
use tide::{Request, StatusCode};
use crate::{FileCache, FileInit, State};

lazy_static! {
    static ref FILES: DashMap<String, FileCache> = DashMap::new();
}

pub async fn create_file(mut req: Request<State>) -> tide::Result {
    let FileInit { name, chunk, size } = req.body_json().await?;

    let name = sanitise_file_name::sanitise(name.as_str());
    let file = File::create(req.state().file_path.join(name)).await?;
    file.set_len(size).await?;

    let mut id_buf = [0u8; 8];
    getrandom(&mut id_buf)?;
    let id = format!("{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}{:02X}", id_buf[0], id_buf[1], id_buf[2], id_buf[3], id_buf[4], id_buf[5], id_buf[6], id_buf[7]);

    FILES.insert(id.clone(), FileCache {
        chunk,
        file: Arc::new(Mutex::new(file)),
        size,
    });

    Ok(id.into())
}

pub async fn finalize_file(req: Request<State>) -> tide::Result {
    let id = req.param("id")?;
    if !FILES.contains_key(id) {
        return Ok(StatusCode::NotFound.into());
    }

    FILES.remove(id).unwrap();

    Ok("Ok".into())
}

pub async fn write_file(req: Request<State>) -> tide::Result {
    let id = req.param("id")?;
    let cache = match FILES.get_mut(id) {
        None => return Ok(StatusCode::NotFound.into()),
        Some(cache) => cache
    };

    /*if let Some(range) = req.header("Range") {
        let range = range.as_str();

        let data = &range[6..].split("-").collect::<Vec<_>>();
        if data.len() != 2 {
            return Ok(StatusCode::BadRequest.into());
        }

        let start = u64::from_str(data[0])?;
        let end = u64::from_str(data[1])?;

        if start > end || end > cache.size || (end - start) > cache.chunk {
            return Ok(StatusCode::BadRequest.into());
        }

        let mut file = cache.file.lock().await;
        let target = file.deref_mut();
        target.seek(SeekFrom::Start(start)).await?;
        async_std::io::copy(&mut req.take(end - start), target).await?;
        drop(file);
    } else {*/
    let mut file = cache.file.lock().await;
    let target = file.deref_mut();
    let pos = target.seek(SeekFrom::Current(0)).await?;
    if pos >= cache.size {
        return Ok(StatusCode::BadRequest.into())
    }
    async_std::io::copy(&mut req.take(cache.chunk), target).await?;
    drop(file);
    //}

    Ok(StatusCode::NoContent.into())
}