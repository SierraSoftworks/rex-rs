mod api;
mod app;
mod auth;
mod components;
mod views;

fn main() {
    yew::Renderer::<app::App>::new().render();
}
