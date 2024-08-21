use axum::{
    extract::Path,
    response::Html,
    routing::{get},
    Router,
};

mod color;

#[tokio::main]
async fn main() {
    tracing_subscriber::fmt::init();

    let app = Router::new()
        .route("/", get(home))
        .route("/*path", get(color));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn home() -> &'static str {
    "Hello, World!"
}

async fn color(Path(path): Path<String>) -> Html<String> {
    let color = color::Color::from_repr(&path);
    if color.is_err() {
        return Html(format!("<div>err {}</div>", &path));
    }
    let (r, g, b) = color.unwrap().color_value.into_components();
    Html(format!(
        "<div style=\"width: 200px; height: 200px; background-color: rgb({} {} {}); border: 1px black solid\"></div>",
        r,
        g,
        b
    ))
}

// BAKERT how to test this? is that a goal in rust?
