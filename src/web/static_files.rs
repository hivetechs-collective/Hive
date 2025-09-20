use super::*;
use axum::http::header;

pub async fn serve_index() -> Html<&'static str> {
    Html(include_str!("../../../web/index.html"))
}

pub async fn serve_static(Path(path): Path<String>) -> Result<Response, StatusCode> {
    match path.as_str() {
        "style.css" => {
            let css = include_str!("../../../web/style.css");
            Ok(Response::builder()
                .header(header::CONTENT_TYPE, "text/css")
                .body(css.into())
                .unwrap())
        }
        "script.js" => {
            let js = include_str!("../../../web/script.js");
            Ok(Response::builder()
                .header(header::CONTENT_TYPE, "application/javascript")
                .body(js.into())
                .unwrap())
        }
        "prism.css" => {
            let css = include_str!("../../../web/prism.css");
            Ok(Response::builder()
                .header(header::CONTENT_TYPE, "text/css")
                .body(css.into())
                .unwrap())
        }
        "prism.js" => {
            let js = include_str!("../../../web/prism.js");
            Ok(Response::builder()
                .header(header::CONTENT_TYPE, "application/javascript")
                .body(js.into())
                .unwrap())
        }
        _ => Err(StatusCode::NOT_FOUND),
    }
}
