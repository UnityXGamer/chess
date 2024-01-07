use leptos::*;
mod board;
use board::BoardComponent;

fn main() {
    mount_to_body(|| {
        view! {
            <BoardComponent/>
        }
    })
}
