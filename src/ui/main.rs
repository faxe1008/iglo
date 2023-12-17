use chessica::engine::{
    board::{ChessBoardState, ChessPiece, PieceColor},
    board_eval::{EvaluationEngine, EvaluationFunction},
};
use core::time::Duration;
use sdl2::{
    event::Event,
    image::{self, InitFlag, LoadTexture},
    keyboard::Keycode,
    pixels::Color,
    rect::Rect,
    render::{Canvas, Texture, TextureCreator},
    ttf::Font,
    video::{Window, WindowContext},
};
use std::env;

const SQUARE_SIZE: i32 = 100;
const MIN_MARGIN: i32 = 20;
const WIDTH_STATS_RIGHT: u32 = 240;

const WINDOW_WIDTH: u32 = SQUARE_SIZE as u32 * 8 + MIN_MARGIN as u32 * 2 + WIDTH_STATS_RIGHT;
const WINDOW_HEIGHT: u32 = SQUARE_SIZE as u32 * 8 + MIN_MARGIN as u32 * 2;

const COLOR_BLACK_FIELD: Color = Color::RGBA(46, 60, 32, 255);
const COLOR_WHITE_FIELD: Color = Color::RGBA(91, 92, 80, 255);
const COLOR_BACKGROUND: Color = Color::RGBA(18, 18, 18, 255);

const PIECE_SPRITE_SIZE: u32 = 320;
const DESIGNATOR_MARGIN: i32 = 5;

pub struct AssetPack<'a> {
    pub sprite_texture: Texture<'a>,
    pub font: Font<'a, 'a>,
}

fn get_square_by_pos(x: i32, mut y: i32, flipped: bool) -> Rect {
    let x_orig = MIN_MARGIN;
    let y_orig = MIN_MARGIN;

    if flipped {
        y = 7 - y;
    }

    Rect::new(
        x_orig + x * SQUARE_SIZE,
        y_orig + y * SQUARE_SIZE,
        SQUARE_SIZE as u32,
        SQUARE_SIZE as u32,
    )
}

fn get_square_by_index(index: usize, flipped: bool) -> Rect {
    let x = (index % 8) as i32;
    let y = (index / 8) as i32;

    get_square_by_pos(x, y, flipped)
}

fn get_designator_rect(
    x: i32,
    mut y: i32,
    flipped: bool,
    is_vertical: bool,
    designator_size: (u32, u32),
) -> Rect {
    let mut rect = get_square_by_pos(x, y, flipped);

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

fn draw_grid(
    canvas: &mut Canvas<Window>,
    asset_pack: &AssetPack,
    texture_creator: &TextureCreator<WindowContext>,
    flipped: bool,
) {
    canvas.set_draw_color(COLOR_BACKGROUND);
    canvas.clear();

    let draw_designator = |canvas: &mut Canvas<Window>,
                           x: i32,
                           y: i32,
                           text: &str,
                           is_vertical,
                           designator_color: Color| {
        let surface = asset_pack
            .font
            .render(text)
            .blended(designator_color)
            .unwrap();
        let texture = texture_creator
            .create_texture_from_surface(&surface)
            .unwrap();
        canvas
            .copy(
                &texture,
                surface.rect(),
                get_designator_rect(x, y, flipped, is_vertical, surface.size()),
            )
            .unwrap();
    };

    for x in 0..8 {
        for y in 0..8 {
            let (field_color, designator_color) = if (x + y) % 2 == 0 {
                (COLOR_WHITE_FIELD, COLOR_BLACK_FIELD)
            } else {
                (COLOR_BLACK_FIELD, COLOR_WHITE_FIELD)
            };

            canvas.set_draw_color(field_color);
            canvas.fill_rect(get_square_by_pos(x, y, flipped)).unwrap();

            if x == 0 {
                draw_designator(
                    canvas,
                    x,
                    y,
                    &(7 - y + 1).to_string(),
                    true,
                    designator_color,
                );
            }

            if y == 7 {
                let char_designator = char::from_u32(x as u32 + 'a' as u32).unwrap().to_string();
                draw_designator(canvas, x, y, &char_designator, false, designator_color);
            }
        }
    }
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
    flipped: bool,
) {
    let piece_boards = [
        (&board_state.board.white_pieces, PieceColor::White),
        (&board_state.board.black_pieces, PieceColor::Black),
    ];

    for (bitboard_array, piece_color) in &piece_boards {
        for (piece_index, bitboard) in bitboard_array.iter().enumerate() {
            let piece = ChessPiece::from(piece_index);
            for i in 0..64 {
                if bitboard.get_bit(i) {
                    canvas
                        .copy(
                            &asset_pack.sprite_texture,
                            get_sprite_rect(&piece, piece_color),
                            get_square_by_index(i, flipped),
                        )
                        .unwrap();
                }
            }
        }
    }
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
        .window("rust-sdl2 demo: Video", WINDOW_WIDTH, WINDOW_HEIGHT)
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

    let mut flipped = false;

    draw_grid(&mut canvas, &asset_pack, &texture_creator, flipped);
    draw_chess_board(&mut canvas, &board_state, &asset_pack, flipped);
    canvas.present();

    println!(
        "Evaluation function: {}",
        EvaluationEngine::eval(&board_state)
    );

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
                    flipped = !flipped;
                    draw_grid(&mut canvas, &asset_pack, &texture_creator, flipped);
                    draw_chess_board(&mut canvas, &board_state, &asset_pack, flipped);
                    canvas.present();
                }

                _ => {}
            }
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 30));
        // The rest of the game loop goes here...
    }
}
