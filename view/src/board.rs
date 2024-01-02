use leptos::*;
use movegen::{board::Board, Rank, File, Square, Color};

#[component]
pub fn BoardComponent() -> impl IntoView {
    let (board, set_board) = create_signal(Board::default());
    view! {
        <div class="board">
            {
                Rank::ALL.iter().rev().map(|rank|{
                    view ! {
                        <div class="rank">
                            {
                                File::ALL.iter().map(|file|{
                                    let sq = Square::from_rank_file(*rank, *file);
                                    
                                    view! {
                                        <div style:background-color=||{
                                            if let 0 = (*rank as usize + *file as usize) % 2 {
                                                "Coral" 
                                            } else {
                                                "AntiqueWhite"
                                            }
                                        }
                                            class="square">
                                            {
                                                if let Some((color, piece)) = board.get().get_sq(sq) {
                                                    let mut c = piece.to_char();
                                                    if color == Color::White {
                                                        c.make_ascii_uppercase();
                                                    }
                                                    Some(view ! {
                                                        <h1>
                                                            {c}
                                                        </h1>
                                                    })
                                                } else {
                                                    None
                                                }
                                            }
                                            <span class="square-label">{format!("{:?}", sq)}</span>
                                        </div>
                                    }
                                }).collect_view()
                            }
                        </div>
                    }
                }).collect_view()
            }
        </div>
    }
}