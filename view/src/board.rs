use engine::{Engine, Random, AnyEngine};
use leptos::*;
use movegen::{board::{Board, Status}, mv::Move, state::State, Color, File, Rank, Square};
use web_time::UNIX_EPOCH;

#[component]
pub fn BoardComponent() -> impl IntoView {
    
    let (engine, set_engine) = create_signal(Some((AnyEngine::Random, false)));
    let (white_pov, set_white_pov) = create_signal(true);
    let (autoplay, set_autoplay) = create_signal(true);
    let (board, set_board) = create_signal(Board::default());
    
    let board = move || board.get();
    let random_engine = move || Engine::<Random>::new(
        board(),
        web_time::SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("systemtime is fine")
            .subsec_nanos() as u64,
    );
    
    let (selected_sq, set_selected_sq) = create_signal(None::<(Square, Vec<Move>)>);
    let (error, set_error) = create_signal(None);
    
    view! {
        <p>{move || format!("Best move: {}", match engine.get() {
            Some((AnyEngine::Random, _)) =>  {
                let mut e = random_engine();
                e.search(100);
                if let Some(best_move) = e.best_move {
                    logging::log!("BEST {}", best_move.to_string());
                    if autoplay.get() {
                        set_board.update(|b|b.make_move(&best_move))
                    }
                    best_move.to_string()
                } else {
                    "No best move found".to_owned()
                }
            },
            None => "No engine selected".to_owned()
        })}</p>
        <p>{move||format!("Status: {}", match board().status() {
            Status::Ongoing(_) => "Ongoing".to_owned(),
            s => format!("{:?}", s)
    })}</p>
        <input type="checkbox" checked = move ||white_pov on:change=move|_|set_white_pov.update(|p|*p=!*p)/>
        <div class="board">
            {
                move|| board().all_sqs(white_pov.get()).iter().map(|rank|{
                    view ! {
                        <div class="rank">
                            {
                                rank.iter().map(|(sq, piece)|{
                                    let sq = *sq;
                                    let svg_file_name = || if let Some((color, piece)) = piece {
                                        Some(format!("/static/{}-{}.svg", match color {
                                            Color::White => "white",
                                            Color::Black => "black"
                                        }, piece.to_char()))
                                    } else {
                                        None
                                    };
                                    let has_shadow = move || if let Some((s, moves)) = selected_sq.get() {
                                        if sq != s && moves.iter().find(|mv|mv.to==sq).is_none() {
                                            true
                                        } else {
                                            false
                                        }
                                    } else {
                                        false
                                    };
                                    view! {
                                        <div style:filter=move||if has_shadow() {
                                            Some("brightness(50%)")
                                        } else {
                                            None
                                        } style:background-color=move||{
                                            if let 0 = (sq.rank() as usize + sq.file() as usize) % 2 {
                                                "Coral"
                                            } else {
                                                "AntiqueWhite"
                                            }
                                        }
                                            class="square"
                                            on:click=move |_|if let Some((_, moves)) = selected_sq.get() {
                                                if let Some(mv) = moves.iter().find(|mv|mv.to==sq) {
                                                    set_board.update(|b|{
                                                        b.make_move(mv);
                                                    });
                                                    set_selected_sq.update(|s|*s=None);

                                                } else {
                                                    set_selected_sq.update(|s|*s=None);
                                                }

                                            } else if selected_sq.get().is_none() {
                                                set_selected_sq.update(move|s|*s=Some((sq, board().get_sq_moves(sq))));
                                            }
                                            >
                                            {
                                                if let Some(svg_file_name) = svg_file_name() {
                                                    Some(view ! {
                                                        <img src={svg_file_name}/>
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
        <input on:blur=move |ev|{
            let fen = event_target_value(&ev);
            match Board::from_fen(&fen) {
                Ok(new_board) => set_board.update(|b|*b=new_board),
                Err(err) => set_error.update(|e|*e=Some(format!("{:?}", err)))
            }
        }/>

        <input type="submit" prop:value="Reset" on:click=move |_|{
            set_board.update(|b|*b=Board::default());
            set_error.update(|e|*e=None);
        }/>
        {
            move || if let Some(e) = error.get() {
                Some(view ! {
                    <p style="color: red;">{e}</p>
                })
            } else {
                None
            }
        }
    }
}
