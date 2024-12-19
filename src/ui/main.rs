use core::time::Duration;
use iglo::{
    chess::{
        board::{ChessBoardState, ChessPiece, PieceColor},
        chess_move::Move,
        move_generator::generate_legal_moves,
        square::Square,
    },
    engine::board_eval::{EvaluationFunction, PieceCountEvaluation, PieceSquareTableEvaluation},
};
use sdl2::{
    audio::{AudioCVT, AudioCallback, AudioDevice, AudioSpecDesired, AudioSpecWAV},
    event::Event,
    image::{self, InitFlag, LoadSurface, LoadTexture},
    keyboard::Keycode,
    mouse::MouseButton,
    pixels::Color,
    rect::{Point, Rect},
    render::{BlendMode, Canvas, Texture, TextureCreator},
    surface::Surface,
    ttf::Font,
    video::{Window, WindowContext},
    AudioSubsystem,
};
use std::env;

const SQUARE_SIZE: i32 = 100;
const MIN_MARGIN: i32 = 20;
const WIDTH_STATS_RIGHT: u32 = 240;

const WINDOW_WIDTH: u32 = SQUARE_SIZE as u32 * 8 + MIN_MARGIN as u32 * 2 + WIDTH_STATS_RIGHT;
const WINDOW_HEIGHT: u32 = SQUARE_SIZE as u32 * 8 + MIN_MARGIN as u32 * 2;

const COLOR_BLACK_FIELD: Color = Color::RGBA(119, 149, 86, 255);
const COLOR_WHITE_FIELD: Color = Color::RGBA(235, 236, 208, 255);
const COLOR_BACKGROUND: Color = Color::RGBA(18, 18, 18, 255);
const COLOR_MOVEMENT_INDICATOR: Color = Color::RGBA(17, 102, 0, 153);
const COLOR_PROMOTION_PROMPT_COLOR: Color = Color::RGBA(230, 230, 230, 200);
const COLOR_CHECK_BACKGROUND: Color = Color::RGBA(230, 0, 0, 200);

const PIECE_SPRITE_SIZE: u32 = 320;
const DESIGNATOR_MARGIN: i32 = 5;

const PROMOTION_PROMPT_HEIGHT: i32 = 250;
const PROMOTION_PIECE_SIZE: u32 = 120;

const CAPTURE_INDICATOR_THICKNESS: u32 = 5;
const CAPTURE_INDICATOR_MARGIN: i32 = 3;
const CAPTURE_INDICATOR_SIDE_LEN: u32 = SQUARE_SIZE as u32 / 5;

#[derive(Default)]
pub struct EvaluationEngine;
impl EvaluationFunction for EvaluationEngine {
    fn eval(&mut self, board_state: &ChessBoardState) -> i32 {
        PieceCountEvaluation.eval(&board_state) + PieceSquareTableEvaluation.eval(&board_state)
    }
}

struct AssetPack<'a> {
    sprite_texture: Texture<'a>,
    font: Font<'a, 'a>,
    capture_sound: AudioDevice<Sound>,
    move_sound: AudioDevice<Sound>,
}

#[derive(Default, Debug)]
struct GameUIState {
    flipped: bool,
    moves_for_selected_piece: Vec<Move>,
    last_clicked_square: Option<u16>,
    dragging_piece_pos: Option<(i32, i32)>,
    promotion_prompt: Option<(PieceColor, Vec<Move>)>,
    white_in_check: bool,
    black_in_check: bool,
}

struct Sound {
    data: Vec<u8>,
    volume: f32,
    pos: usize,
}

impl AudioCallback for Sound {
    type Channel = u8;

    fn callback(&mut self, out: &mut [u8]) {
        for dst in out.iter_mut() {
            // With channel type u8 the "silence" value is 128 (middle of the 0-2^8 range) so we need
            // to both fill in the silence and scale the wav data accordingly. Filling the silence
            // once the wav is finished is trivial, applying the volume is more tricky. We need to:
            // * Change the range of the values from [0, 255] to [-128, 127] so we can multiply
            // * Apply the volume by multiplying, this gives us range [-128*volume, 127*volume]
            // * Move the resulting range to a range centered around the value 128, the final range
            //   is [128 - 128*volume, 128 + 127*volume] – scaled and correctly positioned
            //
            // Using value 0 instead of 128 would result in clicking. Scaling by simply multiplying
            // would not give correct results.
            let pre_scale = *self.data.get(self.pos).unwrap_or(&128);
            let scaled_signed_float = (pre_scale as f32 - 128.0) * self.volume;
            let scaled = (scaled_signed_float + 128.0) as u8;
            *dst = scaled;
            self.pos += 1;
        }
    }
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
    y: i32,
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
    let evaluation = EvaluationEngine::default().eval(board_state);

    let enpassant_text = if let Some(target) = board_state.en_passant_target {
        Square::designator_str_from_index(target as u16)
    } else {
        "None".to_string()
    };

    let text_blocks = [
        format!("Turn: {}", board_state.side.as_display_str()),
        format!("Evaluation: {}", evaluation),
        format!("Castling: {}", board_state.castling_rights.to_string()),
        format!("En Passant: {}", enpassant_text),
        format!("Fullmoves: {}", board_state.full_moves),
        format!("Halfmoves: {}", board_state.half_moves),
        format!(
            "King Attackers: {}",
            board_state.board.king_attackers(board_state.side)[6].0
        ),
        format!(
            "Legal Move Count: {}",
            generate_legal_moves::<false>(board_state, board_state.side).len()
        ),
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

fn draw_piece_at_location(
    canvas: &mut Canvas<Window>,
    asset_pack: &AssetPack,
    piece: ChessPiece,
    color: PieceColor,
    rct: Rect,
) -> Result<(), String> {
    canvas.copy(
        &asset_pack.sprite_texture,
        get_sprite_rect(&piece, &color),
        rct,
    )
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
                    let dst_rct = get_square_by_index(i, ui_state);

                    // Do not draw dragged piece
                    if ui_state.dragging_piece_pos.is_some()
                        && ui_state.last_clicked_square == Some(i as u16)
                    {
                        continue;
                    }

                    if piece == ChessPiece::King
                        && ((*piece_color == PieceColor::White && ui_state.white_in_check)
                            || (*piece_color == PieceColor::Black && ui_state.black_in_check))
                    {
                        canvas.set_blend_mode(BlendMode::Blend);
                        canvas.set_draw_color(COLOR_CHECK_BACKGROUND);
                        canvas.fill_rect(dst_rct)?;
                    }

                    draw_piece_at_location(canvas, asset_pack, piece, *piece_color, dst_rct)?;
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

    if piece_move.is_capture() || piece_move.is_en_passant() {
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

fn draw_moves_indicator(canvas: &mut Canvas<Window>, ui_state: &GameUIState) -> Result<(), String> {
    for piece_move in &ui_state.moves_for_selected_piece {
        draw_single_move_indicator(canvas, *piece_move, ui_state)?;
    }
    Ok(())
}

fn draw_dragged_piece(
    canvas: &mut Canvas<Window>,
    asset_pack: &AssetPack,
    board_state: &ChessBoardState,
    ui_state: &GameUIState,
) -> Result<(), String> {
    if ui_state.dragging_piece_pos.is_none() || ui_state.last_clicked_square.is_none() {
        return Ok(());
    }
    let last_clicked_pos = ui_state.last_clicked_square.unwrap();
    let cursor_pos = ui_state.dragging_piece_pos.unwrap();

    if let Some((piece, piece_col)) = board_state
        .board
        .get_piece_at_pos(last_clicked_pos as usize)
    {
        let dst_rect = Rect::new(
            cursor_pos.0 - SQUARE_SIZE / 2,
            cursor_pos.1 - SQUARE_SIZE / 2,
            SQUARE_SIZE as u32,
            SQUARE_SIZE as u32,
        );
        draw_piece_at_location(canvas, asset_pack, piece, piece_col, dst_rect)?;
    }

    Ok(())
}

fn promotion_prompt_rects() -> [(Rect, ChessPiece); 4] {
    let x = MIN_MARGIN + 4 * SQUARE_SIZE - 2 * (PROMOTION_PIECE_SIZE as i32 + MIN_MARGIN);
    let y = MIN_MARGIN + (SQUARE_SIZE * 4) - (PROMOTION_PIECE_SIZE as i32 / 2);
    [
        ChessPiece::Knight,
        ChessPiece::Bishop,
        ChessPiece::Rook,
        ChessPiece::Queen,
    ]
    .iter()
    .enumerate()
    .map(|(i, &piece)| {
        (
            Rect::new(
                x + i as i32 * (PROMOTION_PIECE_SIZE as i32 + MIN_MARGIN),
                y,
                PROMOTION_PIECE_SIZE,
                PROMOTION_PIECE_SIZE,
            ),
            piece,
        )
    })
    .collect::<Vec<(Rect, ChessPiece)>>()
    .try_into()
    .unwrap()
}

fn draw_promotion_prompt(
    canvas: &mut Canvas<Window>,
    asset_pack: &AssetPack,
    board_state: &ChessBoardState,
    ui_state: &GameUIState,
) -> Result<(), String> {
    if ui_state.promotion_prompt.is_none() {
        return Ok(());
    }

    canvas.set_draw_color(COLOR_PROMOTION_PROMPT_COLOR);
    canvas.set_blend_mode(BlendMode::Blend);
    let rct = Rect::new(
        MIN_MARGIN,
        MIN_MARGIN + (SQUARE_SIZE * 4) - (PROMOTION_PROMPT_HEIGHT / 2),
        8 * SQUARE_SIZE as u32,
        PROMOTION_PROMPT_HEIGHT as u32,
    );
    canvas.fill_rect(rct)?;

    for (rct, piece) in promotion_prompt_rects() {
        draw_piece_at_location(canvas, asset_pack, piece, board_state.side, rct)?;
    }

    Ok(())
}

fn get_square_from_cursor_pos(x: i32, y: i32, ui_state: &GameUIState) -> Option<u16> {
    let x = (x - MIN_MARGIN) / SQUARE_SIZE;
    if x < 0 || x > 7 {
        return None;
    }

    let mut y = (y - MIN_MARGIN) / SQUARE_SIZE;
    if y < 0 || y > 7 {
        return None;
    }
    if ui_state.flipped {
        y = 7 - y;
    }

    Some(x as u16 + y as u16 * 8)
}

fn execute_move_with_src_and_dst(
    board_state: &mut ChessBoardState,
    ui_state: &mut GameUIState,
    asset_pack: &mut AssetPack,
    src: u16,
    dst: u16,
) {
    let moves: Vec<Move> = ui_state
        .moves_for_selected_piece
        .iter()
        .filter(|mv| mv.get_src() == src && mv.get_dst() == dst)
        .map(|&x| x)
        .collect();

    if moves.is_empty() {
    } else if moves.len() == 1 {
        let move_to_play = moves[0];
        println!("{:?}", &move_to_play);
        *board_state = board_state.exec_move(move_to_play);

        if move_to_play.is_capture() {
            play_sound(&mut asset_pack.capture_sound);
        } else {
            play_sound(&mut asset_pack.move_sound);
        }
        println!("{:x}", board_state.zhash.0);
    } else {
        ui_state.promotion_prompt = Some((board_state.side, moves))
    }

    ui_state.black_in_check = !board_state.board.king_attackers(PieceColor::Black)[6].is_empty();
    ui_state.white_in_check = !board_state.board.king_attackers(PieceColor::White)[6].is_empty();

    ui_state.last_clicked_square = None;
    ui_state.moves_for_selected_piece.clear();
}

fn generate_possible_moves_for_piece(board_state: &ChessBoardState, pos: u16) -> Vec<Move> {
    generate_legal_moves::<false>(board_state, board_state.side)
        .iter()
        .filter(|mv| mv.get_src() == pos)
        .map(|&x| x)
        .collect()
}

fn play_sound(audio_device: &mut AudioDevice<Sound>) {
    {
        let mut lock = audio_device.lock();
        (*lock).pos = 0;
    }
    audio_device.resume()
}

fn create_audio_device_sound(path: &str, audio_subsystem: &AudioSubsystem) -> AudioDevice<Sound> {
    let audio_spec = AudioSpecDesired {
        freq: Some(48_000),
        channels: Some(1), // mono
        samples: None,     // default
    };
    let sound_wav = AudioSpecWAV::load_wav(path).expect("Could not load capture WAV file");

    audio_subsystem
        .open_playback(None, &audio_spec, |spec| {
            let cvt = AudioCVT::new(
                sound_wav.format,
                sound_wav.channels,
                sound_wav.freq,
                spec.format,
                spec.channels,
                spec.freq,
            )
            .expect("Could not convert WAV file");

            let data = cvt.convert(sound_wav.buffer().to_vec());

            // initialize the audio callback
            Sound {
                data: data,
                volume: 1.0,
                pos: 0,
            }
        })
        .expect("Audio device not openable to playback")
}

fn main() {
    let args: Vec<String> = env::args().collect();

    let mut board_state = if args.len() < 2 {
        ChessBoardState::from_fen("rnbqkbnr/pppppppp/8/8/8/8/PPPPPPPP/RNBQKBNR w QKqk - 0 0")
    } else {
        ChessBoardState::from_fen(&args[1])
    }
    .expect("Error parsing FEN");

    let sdl_context = sdl2::init().expect("Error creating context");
    let video_subsystem = sdl_context.video().expect("Error creating video subsystem");

    let mut window = video_subsystem
        .window("Iglo UI", WINDOW_WIDTH, WINDOW_HEIGHT)
        .position_centered()
        .opengl()
        .build()
        .map_err(|e| e.to_string())
        .expect("Error building Window");

    window.set_icon(Surface::from_file("iglo.png").unwrap());

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

    let audio_subsystem = sdl_context.audio().expect("Error creating audio context");

    let mut asset_pack = AssetPack {
        sprite_texture,
        font,
        capture_sound: create_audio_device_sound("capture.wav", &audio_subsystem),
        move_sound: create_audio_device_sound("move.wav", &audio_subsystem),
    };

    let mut game_ui_state = GameUIState::default();

    let mut redraw_board = |board_state: &ChessBoardState,
                            game_ui_state: &GameUIState,
                            asset_pack: &AssetPack|
     -> Result<(), String> {
        draw_grid(&mut canvas, asset_pack, &texture_creator, game_ui_state)?;
        draw_chess_board(&mut canvas, &board_state, asset_pack, game_ui_state)?;
        draw_moves_indicator(&mut canvas, game_ui_state)?;
        draw_dragged_piece(&mut canvas, asset_pack, board_state, game_ui_state)?;
        draw_promotion_prompt(&mut canvas, asset_pack, board_state, game_ui_state)?;
        draw_stats_bar(&mut canvas, &board_state, asset_pack, &texture_creator)?;
        canvas.present();
        Ok(())
    };

    redraw_board(&board_state, &game_ui_state, &asset_pack).expect("Error redrawing board");
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
                    redraw_board(&board_state, &game_ui_state, &asset_pack).expect("Error redrawing board");
                }
                Event::MouseButtonDown { x, y, .. } => {
                    if game_ui_state.promotion_prompt.is_none() {
                        let clicked_square = get_square_from_cursor_pos(x, y, &game_ui_state);
                        match (game_ui_state.last_clicked_square, clicked_square) {
                            (Some(src), Some(dst)) => execute_move_with_src_and_dst(
                                &mut board_state,
                                &mut game_ui_state,
                                &mut asset_pack,
                                src,
                                dst,
                            ),
                            (Some(_), None) => {
                                game_ui_state.last_clicked_square = None;
                                game_ui_state.moves_for_selected_piece.clear();
                                game_ui_state.dragging_piece_pos = None;
                            }
                            (None, Some(dst)) => {
                                game_ui_state.last_clicked_square = clicked_square;
                                game_ui_state.moves_for_selected_piece =
                                    generate_possible_moves_for_piece(&board_state, dst);
                            }
                            (None, None) => {}
                        };
                    } else {
                        let promotion_candidates = promotion_prompt_rects();
                        let promotion_target = promotion_candidates
                            .iter()
                            .filter(|(r, _p)| r.contains_point(Point::new(x, y)))
                            .map(|(_, p)| p)
                            .nth(0);

                        if promotion_target.is_none() {
                            continue;
                        }
                        let move_to_exec = game_ui_state
                            .promotion_prompt
                            .unwrap()
                            .1
                            .iter()
                            .filter(|mv| mv.promotion_target() == *promotion_target.unwrap())
                            .map(|y| *y)
                            .nth(0)
                            .unwrap();
                        board_state = board_state.exec_move(move_to_exec);
                        game_ui_state.promotion_prompt = None;
                    }

                    redraw_board(&board_state, &game_ui_state, &asset_pack).expect("Error redrawing board");
                }
                Event::MouseMotion {
                    x, y, mousestate, ..
                } => {
                    if mousestate.left()
                        && game_ui_state.last_clicked_square.is_some()
                        && game_ui_state.promotion_prompt.is_none()
                    {
                        game_ui_state.dragging_piece_pos = Some((x, y));
                        redraw_board(&board_state, &game_ui_state, &asset_pack).expect("Error redrawing board");
                    }
                }
                Event::MouseButtonUp {
                    x, y, mouse_btn, ..
                } => {
                    if mouse_btn == MouseButton::Left
                        && game_ui_state.dragging_piece_pos.is_some()
                        && game_ui_state.last_clicked_square.is_some()
                        && game_ui_state.promotion_prompt.is_none()
                    {
                        let mv_src = game_ui_state.last_clicked_square.unwrap();
                        if let Some(dst_square) = get_square_from_cursor_pos(x, y, &game_ui_state) {
                            execute_move_with_src_and_dst(
                                &mut board_state,
                                &mut game_ui_state,
                                &mut asset_pack,
                                mv_src,
                                dst_square,
                            );
                        }
                        game_ui_state.dragging_piece_pos = None;
                        game_ui_state.last_clicked_square = None;
                        game_ui_state.moves_for_selected_piece.clear();
                        redraw_board(&board_state, &game_ui_state, &asset_pack).expect("Error redrawing board");
                    }
                }

                _ => {}
            }
        }

        ::std::thread::sleep(Duration::new(0, 1_000_000_000u32 / 240));
        // The rest of the game loop goes here...
    }
}
