use chessica::engine::{
    board::{ChessBoardState, ChessPiece, PieceColor},
    board_eval::{EvaluationEngine, EvaluationFunction},
    chess_move::Move,
    move_generator::generate_pseudo_legal_moves,
    square::Square,
};
use core::time::Duration;
use sdl2::{
    event::Event,
    image::{self, InitFlag, LoadTexture},
    keyboard::Keycode,
    pixels::Color,
    rect::{Point, Rect},
    render::{BlendMode, Canvas, Texture, TextureCreator},
    ttf::Font,
    video::{Window, WindowContext},
};
use std::{env, fmt::format};

const SQUARE_SIZE: i32 = 100;
const MIN_MARGIN: i32 = 20;
const WIDTH_STATS_RIGHT: u32 = 240;

const WINDOW_WIDTH: u32 = SQUARE_SIZE as u32 * 8 + MIN_MARGIN as u32 * 2 + WIDTH_STATS_RIGHT;
const WINDOW_HEIGHT: u32 = SQUARE_SIZE as u32 * 8 + MIN_MARGIN as u32 * 2;

const COLOR_BLACK_FIELD: Color = Color::RGBA(119, 149, 86, 255);
const COLOR_WHITE_FIELD: Color = Color::RGBA(235, 236, 208, 255);
const COLOR_BACKGROUND: Color = Color::RGBA(18, 18, 18, 255);
const COLOR_MOVEMENT_INDICATOR: Color = Color::RGBA(17, 102, 0, 153);

const PIECE_SPRITE_SIZE: u32 = 320;
const DESIGNATOR_MARGIN: i32 = 5;

const CAPTURE_INDICATOR_THICKNESS: u32 = 5;
const CAPTURE_INDICATOR_MARGIN: i32 = 3;
const CAPTURE_INDICATOR_SIDE_LEN: u32 = SQUARE_SIZE as u32 / 5;

struct AssetPack<'a> {
    sprite_texture: Texture<'a>,
    font: Font<'a, 'a>,
}

#[derive(Default, Debug)]
struct GameUIState {
    flipped: bool,
    last_clicked_square: Option<u16>
}

fn get_square_by_pos(x: i32, mut y: i32, ui_state: &GameUIState) -> Rect {
    let x_orig = MIN_MARGIN;
    let y_orig = MIN_MARGIN;

    if ui_state.flipped {
        y = 7 - y;
    }

    Rect::new(
        x_orig + x * SQUARE_SIZE,
        y_orig + y * SQUARE_SIZE,
        SQUARE_SIZE as u32,
        SQUARE_SIZE as u32,
    )
}

fn get_square_by_index(index: usize, ui_state: &GameUIState) -> Rect {
    let x = (index % 8) as i32;
    let y = (index / 8) as i32;

    get_square_by_pos(x, y, ui_state)
}

fn get_designator_rect(
    x: i32,
    mut y: i32,
    ui_state: &GameUIState,
    is_vertical: bool,
    designator_size: (u32, u32),
) -> Rect {
    let mut rect = get_square_by_pos(x, y, ui_state);

    if is_vertical {
        rect.set_x(rect.x() + DESIGNATOR_MARGIN);
        rect.set_y(rect.y() + DESIGNATOR_MARGIN);
    } else {
        rect.set_x(
            (rect.x() + rect.width() as i32) - (designator_size.0 as i32 + DESIGNATOR_MARGIN),
        );
        rect.set_y(
            (rect.y() + rect.height() as i32) - (designator_size.1 as i32 + DESIGNATOR_MARGIN),
        );
    }

    rect.set_width(designator_size.0);
    rect.set_height(designator_size.1);

    rect
}

fn draw_stats_bar(
    canvas: &mut Canvas<Window>,
    board_state: &ChessBoardState,
    asset_pack: &AssetPack,
    texture_creator: &TextureCreator<WindowContext>,
) -> Result<(), String> {
    let evaluation = EvaluationEngine::eval(board_state);

    let text_blocks = [
        format!("Evaluation: {}", evaluation),
        format!("Castling: {}", board_state.castling_rights.to_string()),
    ];

    let mut y_offset = 0;

    for text_block in &text_blocks {
        let surface = asset_pack
            .font
            .render(&text_block)
            .blended(COLOR_WHITE_FIELD)
            .map_err(|e| e.to_string())?;
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .map_err(|e| e.to_string())?;

        let stats_rect = Rect::new(
            MIN_MARGIN * 2 + SQUARE_SIZE * 8,
            MIN_MARGIN + y_offset,
            surface.width(),
            surface.height(),
        );

        canvas.copy(&texture, surface.rect(), stats_rect)?;
        y_offset += surface.height() as i32 + 5;
    }
    Ok(())
}

fn draw_grid(
    canvas: &mut Canvas<Window>,
    asset_pack: &AssetPack,
    texture_creator: &TextureCreator<WindowContext>,
    ui_state: &GameUIState,
) -> Result<(), String> {
    canvas.set_draw_color(COLOR_BACKGROUND);
    canvas.clear();

    let draw_designator = |canvas: &mut Canvas<Window>,
                           x: i32,
                           y: i32,
                           text: &str,
                           is_vertical,
                           designator_color: Color|
     -> Result<(), String> {
        let surface = asset_pack
            .font
            .render(text)
            .blended(designator_color)
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        canvas.copy(
            &texture,
            surface.rect(),
            get_designator_rect(x, y, ui_state, is_vertical, surface.size()),
        )?;
        Ok(())
    };

    for x in 0..8 {
        for y in 0..8 {
            let (field_color, designator_color) = if (x + y) % 2 == 0 {
                (COLOR_WHITE_FIELD, COLOR_BLACK_FIELD)
            } else {
                (COLOR_BLACK_FIELD, COLOR_WHITE_FIELD)
            };

            canvas.set_draw_color(field_color);
            canvas.fill_rect(get_square_by_pos(x, y, ui_state))?;

            if x == 0 {
                draw_designator(
                    canvas,
                    x,
                    y,
                    &(7 - y + 1).to_string(),
                    true,
                    designator_color,
                )?;
            }

            if y == 7 {
                let char_designator = char::from_u32(x as u32 + 'a' as u32).unwrap().to_string();
                draw_designator(canvas, x, y, &char_designator, false, designator_color)?;
            }
        }
    }
    Ok(())
}

fn get_sprite_rect(piece: &ChessPiece, color: &PieceColor) -> Rect {
    let y = if *color == PieceColor::White {
        0
    } else {
        PIECE_SPRITE_SIZE
    };

    let x = PIECE_SPRITE_SIZE
        * match piece {
            ChessPiece::King => 0,
            ChessPiece::Queen => 1,
            ChessPiece::Bishop => 2,
            ChessPiece::Knight => 3,
            ChessPiece::Rook => 4,
            ChessPiece::Pawn => 5,
        };

    Rect::new(x as i32, y as i32, PIECE_SPRITE_SIZE, PIECE_SPRITE_SIZE)
}

fn draw_chess_board(
    canvas: &mut Canvas<Window>,
    board_state: &ChessBoardState,
    asset_pack: &AssetPack,
    ui_state: &GameUIState,
) -> Result<(), String> {
    let piece_boards = [
        (&board_state.board.white_pieces, PieceColor::White),
        (&board_state.board.black_pieces, PieceColor::Black),
    ];

    for (bitboard_array, piece_color) in &piece_boards {
        for (piece_index, bitboard) in bitboard_array.iter().enumerate() {
            let piece = ChessPiece::from(piece_index);
            for i in 0..64 {
                if bitboard.get_bit(i) {
                    canvas.copy(
                        &asset_pack.sprite_texture,
                        get_sprite_rect(&piece, piece_color),
                        get_square_by_index(i, ui_state),
                    )?;
                }
            }
        }
    }
    Ok(())
}

fn draw_single_move_indicator(
    canvas: &mut Canvas<Window>,
    piece_move: Move,
    ui_state: &GameUIState,
) -> Result<(), String> {
    canvas.set_draw_color(COLOR_MOVEMENT_INDICATOR);

    let mut rect = get_square_by_index(piece_move.get_dst() as usize, ui_state);

    let (x, y, w, h, s, m, l, t) = (
        rect.x,
        rect.y,
        rect.width(),
        rect.height(),
        SQUARE_SIZE,
        CAPTURE_INDICATOR_MARGIN,
        CAPTURE_INDICATOR_SIDE_LEN,
        CAPTURE_INDICATOR_THICKNESS,
    );

    if piece_move.is_capture() {
        canvas.set_blend_mode(BlendMode::None);
        let corner_rects = [
            Rect::new(x + m, y + m, l, t),
            Rect::new(x + m, y + m, t, l),
            Rect::new(x + s - m - l as i32, y + m, l, t),
            Rect::new(x + s - m - t as i32, y + m, t, l),
            Rect::new(x + m, y + s - m - l as i32, t, l),
            Rect::new(x + m, y + s - m - t as i32, l, t),
            Rect::new(x + s - m - t as i32, y + s - m - l as i32, t, l),
            Rect::new(x + s - m - l as i32, y + s - m - t as i32, l, t),
        ];

        for r in corner_rects {
            canvas.fill_rect(r)?;
        }
    } else {
        canvas.set_blend_mode(BlendMode::Blend);
        rect.set_x(x + (SQUARE_SIZE / 3));
        rect.set_y(y + (SQUARE_SIZE / 3));
        rect.set_width(w - (2 * SQUARE_SIZE / 3) as u32);
        rect.set_height(h - (2 * SQUARE_SIZE / 3) as u32);
        canvas.fill_rect(rect)?;
    }

    Ok(())
}

fn draw_moves_indicator(
    canvas: &mut Canvas<Window>,
    board_state: &ChessBoardState,
    pos: u16,
    ui_state: &GameUIState,
) -> Result<(), String> {
    let moves_of_piece: Vec<Move> = generate_pseudo_legal_moves(board_state, board_state.side)
        .iter()
        .filter(|mv| mv.get_src() == pos)
        .map(|&x| x)
        .collect();
    for piece_move in moves_of_piece {
        draw_single_move_indicator(canvas, piece_move, ui_state)?;
    }
    Ok(())
}

fn get_square_from_cursor_pos(x: i32, y: i32) -> Option<u16> {
    let x = (x - MIN_MARGIN) / SQUARE_SIZE;
    if x < 0 || x > 7 {
        return None;
    }

    let y = (y - MIN_MARGIN) / SQUARE_SIZE;
    if y < 0 || y > 7 {
        return None;
    }
    Some(x as u16 + y as u16 * 8)
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let board_state = if args.len() < 2 {
        ChessBoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QKqk - 0 0")
    } else {
        ChessBoardState::from_fen(&args[1])
    }
    .expect("Error parsing FEN");

    let sdl_context = sdl2::init().expect("Error creating context");
    let video_subsystem = sdl_context.video().expect("Error creating video subsystem");

    let window = video_subsystem
        .window("Chessica UI", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())
        .expect("Error building Window");

    let mut canvas = window
        .into_canvas()
        .build()
        .map_err(|e| e.to_string())
        .expect("Error creating canvas");

    // TODO: move this outside
    let ttf_context = sdl2::ttf::init()
        .map_err(|e| e.to_string())
        .expect("Error creating ttf context");
    let _image_context = image::init(InitFlag::PNG | InitFlag::JPG)
        .map_err(|e| e.to_string())
        .expect("Error creating image context");
    let texture_creator = canvas.texture_creator();
    let sprite_texture = texture_creator
        .load_texture("sprites.png")
        .expect("Error loading texture");

    let mut font = ttf_context
        .load_font("font.ttf", 18)
        .expect("Error loading ttf");
    font.set_style(sdl2::ttf::FontStyle::BOLD);
    let asset_pack = AssetPack {
        sprite_texture,
        font,
    };

    let mut game_ui_state = GameUIState::default();

    let mut redraw_board = |game_ui_state: &GameUIState| -> Result<(), String> {
        draw_grid(&mut canvas, &asset_pack, &texture_creator, game_ui_state)?;
        draw_chess_board(&mut canvas, &board_state, &asset_pack, game_ui_state)?;
        if let Some(pos) = game_ui_state.last_clicked_square {
            draw_moves_indicator(&mut canvas, &board_state, pos, game_ui_state)?;
        }
        draw_stats_bar(&mut canvas, &board_state, &asset_pack, &texture_creator);
        canvas.present();
        Ok(())
    };

    redraw_board(&game_ui_state);
    let mut event_pump = sdl_context.event_pump().unwrap();

    'running: loop {
        for event in event_pump.poll_iter() {
            match event {
                Event::Quit { .. }
                | Event::KeyDown {
                    keycode: Some(Keycode::Escape),
                    ..
                } => break 'running,

                Event::KeyDown {
                    keycode: Some(Keycode::F),
                    ..
                } => {
                    game_ui_state.flipped = !game_ui_state.flipped;
                    redraw_board(&game_ui_state);
                }
                Event::MouseButtonDown { x, y, .. } => {
                    game_ui_state.last_clicked_square = get_square_from_cursor_pos(x, y);
                    redraw_board(&game_ui_state);
                }

                _ => {}
            }
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }
}
