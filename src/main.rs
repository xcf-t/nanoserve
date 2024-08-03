mod upload;
mod download;

use std::ffi::OsStr;
use std::net::{Ipv4Addr, SocketAddrV4, ToSocketAddrs};
use std::path::Path;
use std::sync::Arc;
use async_std::fs::File;
use async_std::path::PathBuf;
use async_std::sync::Mutex;
use tide::{Body, Redirect, Request, Response, StatusCode};
use tide::prelude::*;
use clap::Parser;
use colored::Colorize;
use tide::http::Mime;
use tide::listener::ToListener;

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The path to serve and store data
    path: PathBuf,

    /// Activate upload functionality
    #[arg(short, long, default_value_t = false)]
    upload: bool,

    /// Activate download funcionality
    #[arg(short, long, default_value_t = false)]
    download: bool,

    /// Port to listen on
    #[arg(short, long, default_value_t = 8080)]
    port: u16,

    /// Whether to listen on all interfaces. True by default
    #[arg(short, long, default_value_t = true)]
    listen: bool,


    /// The maximal file size that can be uploaded
    #[arg(short, long, default_value_t = u64::MAX)]
    max_file_size: u64,
}

#[derive(Debug, Deserialize)]
struct FileInit {
    name: String,
    chunk: u64,
    size: u64,
}

struct FileCache {
    file: Arc<Mutex<File>>,
    chunk: u64,
    size: u64,
}

#[derive(Clone)]
struct State {
    file_path: PathBuf,
    upload: bool,
    download: bool,
}

#[async_std::main]
async fn main() -> tide::Result<()> {
    let mut args: Args = Args::parse();

    if !args.upload && !args.download {
        args.download = true;
    }

    let mut app = tide::with_state(State {
        file_path: args.path.clone(),
        upload: args.upload,
        download: args.download,
    });

    app.at("/style.css").get(res_style);
    app.at("/common.js").get(res_common);
    app.at("/upload.js").get(res_upload);
    app.at("/download.js").get(res_download);
    app.at("/bootstrap.min.css").get(res_bootstrap);
    app.at("/bootstrap.bundle.min.js").get(res_bootstrap_js);

    app.at("/").get(main_redirect);

    if args.upload {
        app.at("/upload.html").get(page_upload);
        app.at("/file").post(upload::create_file);
        app.at("/file/:id").post(upload::finalize_file);
        app.at("/file/:id").put(upload::write_file);
    } else {
        app.at("/upload.html").get(main_redirect);
    }

    if args.download {
        app.at("/download.html").get(page_download);
        app.at("/files/*").get(serve_files);
        app.at("/files").post(download::list_files);
    } else {
        app.at("/download.html").get(main_redirect);
    }

    app.at("/status").get(status);

    let ip: Ipv4Addr;
    if args.listen {
        ip = Ipv4Addr::from([0,0,0,0]);
    } else {
        ip = Ipv4Addr::from([127,0,0,1]);
    }

    let mut listener = SocketAddrV4::new(ip, args.port).to_socket_addrs()?.collect::<Vec<_>>().to_listener()?;
    listener.bind(app).await?;

    println!("{}{}{}", "┌".blue(), "─".repeat(50).blue(), "┐".blue());
    println!("{0}{1:50}{0}", "│".blue(), "");
    println!("{0}{1:50}{0}", "│".blue(), "    Started.".blue());
    let mut modes = vec![];
    if args.upload { modes.push("Upload"); }
    if args.download { modes.push("Download"); }
    let mode = format!("{}{}", "    Mode: ".bold(), modes.join(" / "));
    println!("{0}{1:58}{0}", "│".blue(), mode);
    let target_path = args.path.as_path().canonicalize().await?;
    let target_trunc = if target_path.to_str().unwrap().len() > 36 { ".." } else { "" };
    let target = format!("{}{}{}", "    Target: ".bold(), &target_path.to_str().unwrap()[0..34], target_trunc);
    println!("{0}{1:58}{0}", "│".blue(), target);
    println!("{0}{1:50}{0}", "│".blue(), "");
    for info in listener.info().iter() {
        let data = format!("     - Listening: {}", info.connection());
        println!("{0}{1:50}{0}", "│".blue(), data);
    }
    println!("{0}{1:50}{0}", "│".blue(), "");
    println!("{}{}{}", "└".blue(), "─".repeat(50).blue(), "┘".blue());

    listener.accept().await?;

    Ok(())
}

async fn main_redirect(req: Request<State>) -> tide::Result {
    let target: &str = if req.state().upload && !req.state().download {
        "/upload.html"
    } else {
        "/download.html"
    };

    Ok(Redirect::new(target).into())
}

async fn status(req: Request<State>) -> tide::Result {
    Ok(json!({
        "upload": req.state().upload,
        "download": req.state().download,
    }).into())
}

async fn serve_files(req: Request<State>) -> tide::Result {
    let path = req.url().path();
    let path = urlencoding::decode(path)?;
    let path = match path.strip_prefix("/files/") {
        None => return Ok(StatusCode::BadRequest.into()),
        Some(s) => s,
    };

    let mut file_path = req.state().file_path.clone();
    for p in Path::new(path) {
        if p == OsStr::new(".") {
            continue;
        } else if p == OsStr::new("..") {
            file_path.pop();
        } else {
            file_path.push(&p);
        }
    }

    let file_path = PathBuf::from(file_path);
    if !file_path.starts_with(&req.state().file_path) {
        Ok(Response::new(StatusCode::Forbidden))
    } else {
        match Body::from_file(&file_path).await {
            Ok(body) => Ok(Response::builder(StatusCode::Ok).body(body).build()),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                Ok(Response::new(StatusCode::NotFound))
            }
            Err(e) => Err(e.into()),
        }
    }
}

macro_rules! static_endpoint {
    ($func_name:ident, $file:expr, $mime:expr) => {
        async fn $func_name(_req: Request<State>) -> tide::Result {
            Ok(Response::builder(200)
                .body(include_str!($file))
                .content_type(Mime::from($mime))
                .build())
        }
    };
}

static_endpoint!(page_upload, "resources/upload.html", "text/html");
static_endpoint!(page_download, "resources/download.html", "text/html");

static_endpoint!(res_common, "resources/common.js", "text/javascript");
static_endpoint!(res_upload, "resources/upload.js", "text/javascript");
static_endpoint!(res_download, "resources/download.js", "text/javascript");

static_endpoint!(res_style, "resources/style.css", "text/css");

static_endpoint!(res_bootstrap, "resources/bootstrap.min.css", "text/css");
static_endpoint!(res_bootstrap_js, "resources/bootstrap.bundle.min.js", "text/javascript");
