use axum::{
    extract::{Path, Query},
    response::{Html, IntoResponse},
    routing::get,
    Router,
};
use movegen::{board::Board, Color, Square};

use serde::Deserialize;
use tower_http::services::ServeDir;

#[derive(Deserialize)]
struct BoardPageParams {
    selected: Option<String>,
}

async fn board_page(
    Path(fen): Path<String>,
    Query(params): Query<BoardPageParams>,
) -> impl IntoResponse {
    let fen = fen.replace('_', " ");
    let mut board = match Board::from_fen(&fen) {
        Ok(b) => b,
        Err(e) => return format!("{:?}", e).into_response(),
    };

    let selected = if let Some(sq) = params.selected {
        match Square::from_str(&sq) {
            Ok(sq) => Some((sq, board.get_sq_moves(sq))),
            Err(e) => return format!("{:?}", e).into_response(),
        }
    } else {
        None
    };

    let board_html = board
        .all_sqs(true)
        .iter()
        .fold(String::new(), |mut acc, rank| {
            acc += &format!(
                "
        <div class=\"rank\">
        {}
        </div>",
                rank.iter().fold(String::new(), |mut acc, (sq, piece)| {
                    acc += &format!(
                        "
                <a href=\"{}\" class=\"square\" style=\"background-color: {};{}\">
                    {}
                    <span class=\"square-label\">{}</span>
                </a>
            ",
                        if let Some((_, moves)) = &selected {
                            if let Some(mv) = moves.iter().find(|mv| mv.to == *sq) {
                                let mut new_board = board;
                                new_board.make_move(mv);
                                format!("/{}", new_board.fen(true))
                            } else {
                                format!("/{}", board.fen(true))
                            }
                        } else {
                            format!("/{}?selected={}", board.fen(true), sq.to_string())
                        },
                        if let 0 = (sq.rank() as usize + sq.file() as usize) % 2 {
                            "Coral"
                        } else {
                            "AntiqueWhite"
                        },
                        if let Some((selected, moves)) = &selected {
                            if sq == selected || moves.iter().find(|mv|mv.to==*sq).is_some() {
                                ""
                            } else {
                                "filter: brightness(50%);"
                            }
                        } else {
                            ""
                        },
                        if let Some((color, piece)) = piece {
                            let svg_file_name = format!(
                                "/static/{}-{}.svg",
                                match color {
                                    Color::White => "white",
                                    Color::Black => "black",
                                },
                                piece.to_char()
                            );
                            format!("<img src={svg_file_name}/>")
                        } else {
                            "".to_string()
                        },
                        sq.to_string()
                    );
                    acc
                })
            );
            acc
        });

    Html(format!(
        "
        <html>
            <head>
                <style>
                    .html {{
                        padding: 0;
                    }}
                    .board {{
                        display: flex;
                        flex-flow: column;
                    }}
                    .rank {{
                        display: flex;
                    }}
                    .square {{
                        position: relative;
                        display: flex;
                        justify-content: center;
                        align-items: center;
                        width: 64px;
                        height: 64px;
                    }}
                    .square-label {{
                        position: absolute;
                        left: 0.25rem;
                        bottom: 0.25rem;
                        font-size: 0.5rem;
                    }}
                </style>
            </head>
            <body>
                    <div class=\"board\">
                        {board_html}
                    </div>
            </body>
        </html>
    "
    ))
    .into_response()
}

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/*fen", get(board_page))
        .nest_service("/static", ServeDir::new("./static"));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
