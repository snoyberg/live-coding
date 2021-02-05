extern crate cfg_if;
extern crate wasm_bindgen;

mod utils;

use cfg_if::cfg_if;
use wasm_bindgen::prelude::*;
use snafu::{Snafu, ResultExt, OptionExt};
use std::collections::HashMap;
use url::Url;
use chrono::NaiveDate;

cfg_if! {
    // When the `wee_alloc` feature is enabled, use `wee_alloc` as the global
    // allocator.
    if #[cfg(feature = "wee_alloc")] {
        extern crate wee_alloc;
        #[global_allocator]
        static ALLOC: wee_alloc::WeeAlloc = wee_alloc::WeeAlloc::INIT;
    }
}

#[wasm_bindgen]
pub struct Response {
    status: u16,
    body: String,
    content_type: String,
}

#[wasm_bindgen]
impl Response {
    pub fn get_status(&self) -> u16 {
        self.status
    }

    pub fn get_body(&self) -> String {
        self.body.clone()
    }

    pub fn get_content_type(&self) -> String {
        self.content_type.clone()
    }
}

const HTMLTYPE: &str = "text/html; charset=utf-8";
const CSSTYPE: &str = "text/css; charset=utf-8";

type Result<T, E = Error> = std::result::Result<T, E>;

#[derive(Debug, Snafu)]
enum Error {
    #[snafu(display("Could not parse URL {}: {}", url, source))]
    UrlError { url: String, source: url::ParseError },
    #[snafu(display("Field not provided for prediction: {}", name))]
    FieldNotProvided { name: &'static str },
    #[snafu(display("Invalid date {}: {}", date, source))]
    InvalidDate { date: String, source: chrono::ParseError },
}

impl Error {
    fn status(&self) -> u16 {
        use Error::*;
        match self {
            UrlError {..} => 500,
            FieldNotProvided { .. } => 400,
            InvalidDate { .. } => 400,
        }
    }
}

#[wasm_bindgen]
pub async fn handle(url: String) -> Response {
    match handle_inner(url).await {
        Ok(res) => res,
        Err(e) => error_response(e)
    }
}

async fn handle_inner(url: String) -> Result<Response> {
    let url: Url = url.parse().with_context(|| UrlError { url: url.clone() })?;
    let path = url.path();
    let pieces: Vec<&str> = path.split('/').skip(1).collect();
    let res = match pieces.as_slice() {
        &[""] => homepage(),
        &["style.css"] => style_css(),
        &["predict"] => predict(&url)?,
        _ => not_found(path),
    };
    Ok(res)
}

fn error_response(err: Error) -> Response {
    Response {
        status: err.status(),
        content_type: HTMLTYPE.into(),
        body: format!(r#"<!DOCTYPE html>
<html>
    <head>
        <meta charset=utf-8>
        <title>Error occurred</title>
        <link rel='stylesheet' href='/style.css'>
    </head>
    <body>
        <h1>An error occurred</h1>
        <pre>{}</pre>
    </body>
</html>
"#,
    err
    )
    }
}

fn homepage() -> Response {
    Response {
        status: 200,
        content_type: HTMLTYPE.into(),
        body: r#"<!DOCTYPE html>
<html>
    <head>
        <meta charset=utf-8>
        <title>SnoyPredict!</title>
        <link rel='stylesheet' href='/style.css'>
    </head>
    <body>
        <h1>SnoyPredict!</h1>
        <p>More information, boring, blah blah blah</p>
        <form action='/predict'>
            <p>Prediction goes live on <input required name='date' type='date'></p>
            <textarea required name='prediction'>The sky will still be blue</textarea>
            <button>Predict!</button>
        </form>
    </body>
</html>
"#.into()
    }
}

fn not_found(path: &str) -> Response {
    Response {
        status: 200,
        content_type: HTMLTYPE.into(),
        body: format!(r#"<!DOCTYPE html>
<html>
    <head>
        <meta charset=utf-8>
        <title>Not found</title>
        <link rel='stylesheet' href='/style.css'>
    </head>
    <body>
        <h1>Not found</h1>
        <p>Could not find requested path {}</p>
        <p><a href="/">Return to homepage</a></p>
    </body>
</html>
"#, path)
    }
}

fn style_css() -> Response {
    Response {
        status: 200,
        content_type: CSSTYPE.into(),
        body: r#"
body {
    width: 760px;
    margin: 0 auto;
    background: #000;
    color: #fff;
}

h1 {
    color: red;
}

textarea {
    width: 70%;
    margin: 0 auto;
    height: 10em;
}

button {
    display: block;
}
"#.into()
    }
}

fn predict(url: &Url) -> Result<Response> {
    let params = url.query_pairs().collect::<HashMap<_, _>>();
    let get_field = |name| {
        params.get(name).context(FieldNotProvided { name })
    };
    let prediction = get_field("prediction")?;
    let date = get_field("date")?;
    let date: NaiveDate = date.parse().with_context(|| InvalidDate { date: date.to_owned() })?;

    Ok(Response {
        status: 200,
        content_type: HTMLTYPE.into(),
        body: format!(r#"<!DOCTYPE html>
<html>
    <head>
        <meta charset=utf-8>
        <title>Prediction made (I'm lying)</title>
        <link rel='stylesheet' href='/style.css'>
    </head>
    <body>
        <h1>Prediction made</h1>
        <p>Prediction: {prediction}</p>
        <p>Date: {date}</p>
        <p><a href="/">Return to homepage</a></p>
    </body>
</html>
"#,
prediction = prediction,
date = date,
)
    })
}
