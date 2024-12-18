use actix_files::Files;
use actix_multipart::Multipart;
use actix_web::{web, App, HttpResponse, HttpServer, Responder, middleware, Error};
use chrono::Utc;
use futures_util::stream::StreamExt;
use html_escape::encode_safe;
use log::{error, info};
use mime_guess::mime;
use rusqlite::{params, Connection};
use serde::{Deserialize, Serialize};
use std::fs as stdfs;
use std::io::Write;
use std::sync::{Arc, Mutex};
use uuid::Uuid;

#[derive(Serialize, Deserialize, Clone, Debug)]
enum MediaType {
    Image,
    Video,
}

impl MediaType {
    fn to_str(&self) -> &str {
        match self {
            MediaType::Image => "Image",
            MediaType::Video => "Video",
        }
    }

    fn from_str(s: &str) -> Option<MediaType> {
        match s {
            "Image" => Some(MediaType::Image),
            "Video" => Some(MediaType::Video),
            _ => None,
        }
    }
}

#[derive(Serialize, Deserialize, Clone)]
struct Post {
    id: i32,
    name: String,
    subject: String,
    body: String,
    timestamp: i64,
    media_url: Option<String>,
    media_type: Option<MediaType>,
}

const IMAGE_UPLOAD_DIR: &str = "./uploads/images/";
const VIDEO_UPLOAD_DIR: &str = "./uploads/videos/";
const DB_FILE: &str = "posts.db";

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    env_logger::init();

    // Ensure directories
    for dir in &[IMAGE_UPLOAD_DIR, VIDEO_UPLOAD_DIR] {
        if !std::path::Path::new(dir).exists() {
            stdfs::create_dir_all(dir)?;
            info!("Created directory: {}", dir);
        }
    }

    // Initialize SQLite database
    let conn = Connection::open(DB_FILE).expect("Failed to open DB");
    db_init(&conn).expect("Failed to initialize DB schema");

    let conn = Arc::new(Mutex::new(conn));

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(conn.clone()))
            .wrap(middleware::Logger::default())
            .service(Files::new("/static", "./static"))
            .service(Files::new("/uploads/images", IMAGE_UPLOAD_DIR))
            .service(Files::new("/uploads/videos", VIDEO_UPLOAD_DIR))
            .route("/", web::get().to(homepage))
            .route("/post", web::post().to(create_post))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}

fn db_init(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute(
        "CREATE TABLE IF NOT EXISTS posts (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            subject TEXT NOT NULL,
            body TEXT NOT NULL,
            timestamp INTEGER NOT NULL,
            media_url TEXT,
            media_type TEXT
        )",
        [],
    )?;
    Ok(())
}

fn load_posts_from_db(conn: &Connection) -> rusqlite::Result<Vec<Post>> {
    let mut stmt = conn.prepare("SELECT id, name, subject, body, timestamp, media_url, media_type FROM posts ORDER BY timestamp DESC")?;
    let rows = stmt.query_map([], |row| {
        let media_type_str: Option<String> = row.get(6)?;
        let media_type = media_type_str.as_deref().and_then(MediaType::from_str);

        Ok(Post {
            id: row.get(0)?,
            name: row.get(1)?,
            subject: row.get(2)?,
            body: row.get(3)?,
            timestamp: row.get(4)?,
            media_url: row.get(5)?,
            media_type,
        })
    })?;

    let mut posts = Vec::new();
    for row in rows {
        posts.push(row?);
    }
    Ok(posts)
}

fn insert_post(conn: &Connection, post: &Post) -> rusqlite::Result<()> {
    let media_type_str = post.media_type.as_ref().map(|m| m.to_str());
    conn.execute(
        "INSERT INTO posts (name, subject, body, timestamp, media_url, media_type)
         VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
        params![
            post.name,
            post.subject,
            post.body,
            post.timestamp,
            post.media_url,
            media_type_str
        ],
    )?;
    Ok(())
}

fn escape_html(input: &str) -> String {
    encode_safe(input).to_string()
}

fn render_error_page(title: &str, message: &str) -> String {
    format!(
        r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <title>{}</title>
    <link rel="stylesheet" href="/static/css/style.css">
</head>
<body>
    <h1>{}</h1>
    <p>{}</p>
    <a href="/">Back to Home</a>
</body>
</html>"#,
        escape_html(title),
        escape_html(title),
        escape_html(message)
    )
}

async fn homepage(conn_data: web::Data<Arc<Mutex<Connection>>>) -> impl Responder {
    let posts = {
        let conn = conn_data.lock().unwrap();
        match load_posts_from_db(&conn) {
            Ok(p) => p,
            Err(e) => {
                error!("Failed to load posts: {}", e);
                return HttpResponse::InternalServerError()
                    .content_type("text/html")
                    .body(render_error_page("Internal Server Error", "Failed to load posts"));
            }
        }
    };

    let threads_html = if posts.is_empty() {
        "<p>No posts yet.</p>".to_string()
    } else {
        posts.iter().map(render_post).collect::<Vec<_>>().join("\n")
    };

    let html = format!(
r#"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<title>/a/ - Random</title>
<meta name="viewport" content="width=device-width, initial-scale=1, user-scalable=yes">
<link rel="stylesheet" title="default" href="/static/css/style.css" type="text/css" media="screen">
<link rel="stylesheet" title="style1" href="/static/css/1.css" type="text/css" media="screen" disabled="disabled">
<link rel="stylesheet" title="style2" href="/static/css/2.css" type="text/css" media="screen" disabled="disabled">
<link rel="stylesheet" title="style3" href="/static/css/3.css" type="text/css" media="screen" disabled="disabled">
<link rel="stylesheet" title="style4" href="/static/css/4.css" type="text/css" media="screen" disabled="disabled">
<link rel="stylesheet" title="style5" href="/static/css/5.css" type="text/css" media="screen" disabled="disabled">
<link rel="stylesheet" title="style6" href="/static/css/6.css" type="text/css" media="screen" disabled="disabled">
<link rel="stylesheet" title="style7" href="/static/css/7.css" type="text/css" media="screen" disabled="disabled">
<link rel="stylesheet" href="/static/css/font-awesome/css/font-awesome.min.css">

<script type="text/javascript">
    const active_page = "index";
    const board_name = "a";

    function setActiveStyleSheet(title) {{
        const links = document.getElementsByTagName("link");
        for (let i = 0; i < links.length; i++) {{
            const a = links[i];
            if(a.getAttribute("rel") && a.getAttribute("rel").indexOf("stylesheet") !== -1 && a.getAttribute("title")) {{
                a.disabled = true;
                if(a.getAttribute("title") === title) a.disabled = false;
            }}
        }}
        localStorage.setItem('selectedStyle', title);
    }}

    window.addEventListener('load', () => {{
        const savedStyle = localStorage.getItem('selectedStyle');
        if(savedStyle) {{
            setActiveStyleSheet(savedStyle);
        }}
    }});
</script>

<script type="text/javascript" src="/static/js/jquery.min.js"></script>
<script type="text/javascript" src="/static/js/main.js"></script>
<script type="text/javascript" src="/static/js/inline-expanding.js"></script>
<script type="text/javascript" src="/static/js/hide-form.js"></script>
</head>
<body class="visitor is-not-moderator active-index" data-stylesheet="default">
<header><h1>/a/ - Random</h1><div class="subtitle"></div></header>
<form name="post" enctype="multipart/form-data" action="/post" method="post">
<input type="hidden" name="csrf_token" value="TODO_CSRF_TOKEN">
<table>
    <tr><th>Name</th><td><input type="text" name="name" size="25" maxlength="35" required></td></tr>
    <tr><th>Subject</th><td><input type="text" name="subject" size="25" maxlength="100" required>
        <input type="submit" name="post" value="New Topic" style="margin-left:2px;"></td></tr>
    <tr><th>Comment</th><td><textarea name="body" id="body" rows="5" cols="35" required></textarea></td></tr>
    <tr id="upload"><th>File</th><td><input type="file" name="file" accept=".jpg,.jpeg,.png,.gif,.webp,.mp4"></td></tr>
</table>
</form>
<hr />
{threads}
<div class="pagination"><strong>1</strong> </div><footer>
    <!-- Style selector -->
    <div id="style-selector">
        <label for="style_select">Style:</label>
        <select id="style_select" onchange="setActiveStyleSheet(this.value)">
            <option value="default">default</option>
            <option value="style1">style1</option>
            <option value="style2">style2</option>
            <option value="style3">style3</option>
            <option value="style4">style4</option>
            <option value="style5">style5</option>
            <option value="style6">style6</option>
            <option value="style7">style7</option>
        </select>
    </div>

    <p class="unimportant">
        All trademarks, copyrights,
        comments, and images on this page are owned by and are
        the responsibility of their respective parties.
    </p>

    <div style="text-align:center; margin-top:10px;">
        <a href="https://example.com/">COM</a> | 
        <a href="https://example.net/">NET</a> |
        <a href="https://example.org/">ORG</a>
    </div>
</footer>

<div id="home-button">
    <a href="../">Home</a>
</div>

<script type="text/javascript">ready();</script>
</body>
</html>"#,
threads = threads_html
    );

    HttpResponse::Ok().content_type("text/html").body(html)
}

fn render_post(post: &Post) -> String {
    let files_html = if let Some(url) = &post.media_url {
        match post.media_type {
            Some(MediaType::Image) => format!(
                r#"<div class="files">
    <div class="file">
        <p class="fileinfo">File: <a href="{}">{}</a></p>
        <a href="{}" target="_blank"><img class="post-image" src="{}" alt="" /></a>
    </div>
</div>"#,
                escape_html(url),
                escape_html(url),
                escape_html(url),
                escape_html(url)
            ),
            Some(MediaType::Video) => format!(
                r#"<div class="files">
    <div class="file">
        <p class="fileinfo">File: <a href="{}">{}</a></p>
        <video class="post-video" controls>
            <source src="{}" type="video/mp4">
            Your browser does not support the video tag.
        </video>
    </div>
</div>"#,
                escape_html(url),
                escape_html(url),
                escape_html(url)
            ),
            None => "".to_string(),
        }
    } else {
        "".to_string()
    };

    format!(
        r#"<div class="thread" id="thread_{id}" data-board="a">
{files}
<div class="post op" id="op_{id}">
<p class="intro"><span class="subject">{subject}</span> <span class="name">{name}</span>
    &nbsp;<a href="threads/thread_{id}.html">Reply</a>
</p>
<div class="body">{body}</div>
</div>
<br class="clear"/>
<hr/>
</div>"#,
        id = post.id,
        files = files_html,
        subject = escape_html(&post.subject),
        name = escape_html(&post.name),
        body = escape_html(&post.body)
    )
}

async fn create_post(
    conn_data: web::Data<Arc<Mutex<Connection>>>,
    mut payload: Multipart,
) -> Result<HttpResponse, Error> {
    let mut name = String::new();
    let mut subject = String::new();
    let mut body = String::new();
    let mut media_url: Option<String> = None;
    let mut media_type: Option<MediaType> = None;

    while let Some(item) = payload.next().await {
        let mut field = item?;
        let content_disposition = field.content_disposition();
        let field_name = if let Some(n) = content_disposition.get_name() {
            n
        } else {
            continue;
        };

        match field_name {
            "name" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    name.push_str(&String::from_utf8_lossy(&data));
                }
            }
            "subject" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    subject.push_str(&String::from_utf8_lossy(&data));
                }
            }
            "body" => {
                while let Some(chunk) = field.next().await {
                    let data = chunk?;
                    body.push_str(&String::from_utf8_lossy(&data));
                }
            }
            "file" => {
                if let Some(filename) = content_disposition.get_filename() {
                    if filename.trim().is_empty() {
                        continue;
                    }
                    let mime_type = mime_guess::from_path(&filename).first_or_octet_stream();
                    match mime_type.type_() {
                        mime::IMAGE => {
                            if !matches!(mime_type.subtype().as_ref(), "jpeg" | "jpg" | "png" | "gif" | "webp") {
                                return Ok(HttpResponse::BadRequest().body("Unsupported image format"));
                            }

                            let unique_id = Uuid::new_v4().to_string();
                            let extension = mime_type.subtype().as_str();
                            let sanitized_filename = format!("{}.{}", unique_id, extension);
                            let filepath = format!("{}{}", IMAGE_UPLOAD_DIR, sanitized_filename);
                            let filepath_clone = filepath.clone();

                            let mut f = web::block(move || stdfs::File::create(&filepath_clone)).await??;
                            while let Some(chunk) = field.next().await {
                                let data = chunk?;
                                f = web::block(move || f.write_all(&data).map(|_| f)).await??;
                            }

                            if image::open(&filepath).is_err() {
                                stdfs::remove_file(&filepath)?;
                                return Ok(HttpResponse::BadRequest().body("Invalid image file"));
                            }

                            media_url = Some(format!("/uploads/images/{}", sanitized_filename));
                            media_type = Some(MediaType::Image);
                        }
                        mime::VIDEO => {
                            if mime_type.subtype().as_ref() != "mp4" {
                                return Ok(HttpResponse::BadRequest().body("Unsupported video format"));
                            }

                            let unique_id = Uuid::new_v4().to_string();
                            let extension = mime_type.subtype().as_str();
                            let sanitized_filename = format!("{}.{}", unique_id, extension);
                            let filepath = format!("{}{}", VIDEO_UPLOAD_DIR, sanitized_filename);
                            let filepath_clone = filepath.clone();

                            let mut f = web::block(move || stdfs::File::create(&filepath_clone)).await??;
                            while let Some(chunk) = field.next().await {
                                let data = chunk?;
                                f = web::block(move || f.write_all(&data).map(|_| f)).await??;
                            }

                            media_url = Some(format!("/uploads/videos/{}", sanitized_filename));
                            media_type = Some(MediaType::Video);
                        }
                        _ => {
                            return Ok(HttpResponse::BadRequest().body("Unsupported media type"));
                        }
                    }
                }
            }
            _ => {}
        }
    }

    if name.trim().is_empty() || subject.trim().is_empty() || body.trim().is_empty() {
        return Ok(HttpResponse::BadRequest()
            .content_type("text/html")
            .body(render_error_page("Bad Request", "Name, Subject, and Comment cannot be empty")));
    }

    let post = Post {
        id: 0, // Will be assigned by autoincrement
        name: name.trim().to_string(),
        subject: subject.trim().to_string(),
        body: body.trim().to_string(),
        timestamp: Utc::now().timestamp(),
        media_url,
        media_type,
    };

    {
        let conn = conn_data.lock().unwrap();
        if let Err(e) = insert_post(&conn, &post) {
            error!("Failed to save post: {}", e);
            return Ok(HttpResponse::InternalServerError()
                .content_type("text/html")
                .body(render_error_page("Internal Server Error", "Failed to save post")));
        }
    }

    Ok(HttpResponse::SeeOther()
        .append_header(("Location", "/"))
        .finish())
}
