use macroquad::prelude::*;

const WINS: [[usize; 3]; 8] = [
    [0, 1, 2],
    [3, 4, 5],
    [6, 7, 8],
    [0, 3, 6],
    [1, 4, 7],
    [2, 5, 8],
    [0, 4, 8],
    [2, 4, 6],
];

#[derive(Clone, Copy, PartialEq, Eq)]
enum Player {
    X,
    O,
}

impl Player {
    fn other(self) -> Self {
        match self {
            Self::X => Self::O,
            Self::O => Self::X,
        }
    }

    fn label(self) -> &'static str {
        match self {
            Self::X => "X",
            Self::O => "O",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum GameMode {
    Local,
    Computer,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BoardMode {
    Classic,
    Super,
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum BoardResult {
    Won(Player),
    Draw,
}

#[derive(Clone, Copy)]
struct MoveAnim {
    board: usize,
    cell: usize,
    born: f64,
}

#[derive(Clone, Copy)]
struct Button {
    rect: Rect,
    text: &'static str,
}

#[derive(Clone)]
struct App {
    game_mode: GameMode,
    board_mode: BoardMode,
    current: Player,
    classic: [Option<Player>; 9],
    super_cells: [[Option<Player>; 9]; 9],
    super_results: [Option<BoardResult>; 9],
    active_super_board: Option<usize>,
    winner: Option<BoardResult>,
    animations: Vec<MoveAnim>,
    ai_wait_until: f64,
    message: String,
}

impl Default for App {
    fn default() -> Self {
        Self {
            game_mode: GameMode::Local,
            board_mode: BoardMode::Classic,
            current: Player::X,
            classic: [None; 9],
            super_cells: [[None; 9]; 9],
            super_results: [None; 9],
            active_super_board: None,
            winner: None,
            animations: Vec::new(),
            ai_wait_until: 0.0,
            message: "X starts".to_string(),
        }
    }
}

impl App {
    fn reset(&mut self) {
        let game_mode = self.game_mode;
        let board_mode = self.board_mode;
        *self = Self {
            game_mode,
            board_mode,
            ..Self::default()
        };
    }

    fn set_game_mode(&mut self, mode: GameMode) {
        if self.game_mode != mode {
            self.game_mode = mode;
            self.reset();
        }
    }

    fn set_board_mode(&mut self, mode: BoardMode) {
        if self.board_mode != mode {
            self.board_mode = mode;
            self.reset();
        }
    }

    fn status(&self) -> String {
        if let Some(result) = self.winner {
            return match result {
                BoardResult::Won(player) => format!("{} wins", player.label()),
                BoardResult::Draw => "Draw game".to_string(),
            };
        }

        match self.board_mode {
            BoardMode::Classic => format!("{} to move", self.current.label()),
            BoardMode::Super => {
                if let Some(board) = self.active_super_board {
                    format!("{} to move in board {}", self.current.label(), board + 1)
                } else {
                    format!("{} to move anywhere", self.current.label())
                }
            }
        }
    }

    fn play_classic(&mut self, cell: usize) {
        if self.winner.is_some() || self.classic[cell].is_some() {
            return;
        }

        self.classic[cell] = Some(self.current);
        self.animations.push(MoveAnim {
            board: 0,
            cell,
            born: get_time(),
        });
        self.winner = board_result(&self.classic);
        if self.winner.is_none() {
            self.current = self.current.other();
        }
        self.message = self.status();
        self.queue_ai();
    }

    fn play_super(&mut self, board: usize, cell: usize) {
        if self.winner.is_some()
            || self.super_cells[board][cell].is_some()
            || self.super_results[board].is_some()
            || !self.is_super_board_active(board)
        {
            return;
        }

        self.super_cells[board][cell] = Some(self.current);
        self.animations.push(MoveAnim {
            board,
            cell,
            born: get_time(),
        });
        self.super_results[board] = board_result(&self.super_cells[board]);

        let target_open = self.super_results[cell].is_none()
            && self.super_cells[cell].iter().any(Option::is_none);
        self.active_super_board = target_open.then_some(cell);

        self.winner = super_result(&self.super_results);
        if self.winner.is_none() {
            self.current = self.current.other();
        }
        self.message = self.status();
        self.queue_ai();
    }

    fn is_super_board_active(&self, board: usize) -> bool {
        self.active_super_board
            .map_or(true, |active| active == board)
    }

    fn queue_ai(&mut self) {
        if self.game_mode == GameMode::Computer
            && self.current == Player::O
            && self.winner.is_none()
        {
            self.ai_wait_until = get_time() + 0.35;
        }
    }

    fn maybe_ai_move(&mut self) {
        if self.game_mode != GameMode::Computer
            || self.current != Player::O
            || self.winner.is_some()
            || get_time() < self.ai_wait_until
        {
            return;
        }

        match self.board_mode {
            BoardMode::Classic => {
                if let Some(cell) = best_classic_move(self.classic, Player::O) {
                    self.play_classic(cell);
                }
            }
            BoardMode::Super => {
                if let Some((board, cell)) = best_super_move(self) {
                    self.play_super(board, cell);
                }
            }
        }
    }

    fn handle_click(&mut self) {
        if self.game_mode == GameMode::Computer && self.current == Player::O {
            return;
        }

        let (mx, my) = mouse_position();
        if my < 118.0 {
            return;
        }

        match self.board_mode {
            BoardMode::Classic => {
                if let Some(cell) = hit_classic_cell(mx, my) {
                    self.play_classic(cell);
                }
            }
            BoardMode::Super => {
                if let Some((board, cell)) = hit_super_cell(mx, my) {
                    self.play_super(board, cell);
                }
            }
        }
    }
}

fn board_result(board: &[Option<Player>; 9]) -> Option<BoardResult> {
    for line in WINS {
        if let Some(player) = board[line[0]] {
            if board[line[1]] == Some(player) && board[line[2]] == Some(player) {
                return Some(BoardResult::Won(player));
            }
        }
    }
    board
        .iter()
        .all(Option::is_some)
        .then_some(BoardResult::Draw)
}

fn super_result(results: &[Option<BoardResult>; 9]) -> Option<BoardResult> {
    let mut meta = [None; 9];
    for (i, result) in results.iter().enumerate() {
        meta[i] = match result {
            Some(BoardResult::Won(player)) => Some(*player),
            _ => None,
        };
    }

    for line in WINS {
        if let Some(player) = meta[line[0]] {
            if meta[line[1]] == Some(player) && meta[line[2]] == Some(player) {
                return Some(BoardResult::Won(player));
            }
        }
    }

    results
        .iter()
        .all(Option::is_some)
        .then_some(BoardResult::Draw)
}

fn best_classic_move(board: [Option<Player>; 9], player: Player) -> Option<usize> {
    let mut best = None;
    let mut best_score = i32::MIN;
    for cell in empty_cells(&board) {
        let mut next = board;
        next[cell] = Some(player);
        let score = minimax(next, player.other(), player, -100, 100);
        if score > best_score {
            best_score = score;
            best = Some(cell);
        }
    }
    best
}

fn minimax(
    board: [Option<Player>; 9],
    turn: Player,
    ai: Player,
    mut alpha: i32,
    mut beta: i32,
) -> i32 {
    if let Some(result) = board_result(&board) {
        return match result {
            BoardResult::Won(player) if player == ai => 10,
            BoardResult::Won(_) => -10,
            BoardResult::Draw => 0,
        };
    }

    if turn == ai {
        let mut score = i32::MIN;
        for cell in empty_cells(&board) {
            let mut next = board;
            next[cell] = Some(turn);
            score = score.max(minimax(next, turn.other(), ai, alpha, beta) - 1);
            alpha = alpha.max(score);
            if beta <= alpha {
                break;
            }
        }
        score
    } else {
        let mut score = i32::MAX;
        for cell in empty_cells(&board) {
            let mut next = board;
            next[cell] = Some(turn);
            score = score.min(minimax(next, turn.other(), ai, alpha, beta) + 1);
            beta = beta.min(score);
            if beta <= alpha {
                break;
            }
        }
        score
    }
}

fn empty_cells(board: &[Option<Player>; 9]) -> Vec<usize> {
    let order = [4, 0, 2, 6, 8, 1, 3, 5, 7];
    order.into_iter().filter(|&i| board[i].is_none()).collect()
}

fn best_super_move(app: &App) -> Option<(usize, usize)> {
    let moves = legal_super_moves(app);
    let mut best = None;
    let mut best_score = i32::MIN;
    for (board, cell) in moves {
        let mut next = app.clone();
        apply_super_sim(&mut next, board, cell, Player::O);
        let score = evaluate_super(&next, Player::O) + tactical_bonus(app, board, cell, Player::O);
        if score > best_score {
            best_score = score;
            best = Some((board, cell));
        }
    }
    best
}

fn legal_super_moves(app: &App) -> Vec<(usize, usize)> {
    let mut moves = Vec::new();
    for board in 0..9 {
        if app.super_results[board].is_some() || !app.is_super_board_active(board) {
            continue;
        }
        for cell in empty_cells(&app.super_cells[board]) {
            moves.push((board, cell));
        }
    }
    moves
}

fn apply_super_sim(app: &mut App, board: usize, cell: usize, player: Player) {
    app.super_cells[board][cell] = Some(player);
    app.super_results[board] = board_result(&app.super_cells[board]);
    let target_open =
        app.super_results[cell].is_none() && app.super_cells[cell].iter().any(Option::is_none);
    app.active_super_board = target_open.then_some(cell);
}

fn tactical_bonus(app: &App, board: usize, cell: usize, player: Player) -> i32 {
    let mut score = 0;
    let mut small = app.super_cells[board];
    small[cell] = Some(player);
    if matches!(board_result(&small), Some(BoardResult::Won(p)) if p == player) {
        score += 80;
    }
    let target = cell;
    if app.super_results[target].is_some() || app.super_cells[target].iter().all(Option::is_some) {
        score -= 12;
    } else {
        score += 6 - board_value(target).abs() / 2;
    }
    score + board_value(cell)
}

fn evaluate_super(app: &App, ai: Player) -> i32 {
    if let Some(result) = super_result(&app.super_results) {
        return match result {
            BoardResult::Won(player) if player == ai => 10_000,
            BoardResult::Won(_) => -10_000,
            BoardResult::Draw => 0,
        };
    }

    let mut score = 0;
    for board in 0..9 {
        match app.super_results[board] {
            Some(BoardResult::Won(player)) if player == ai => score += 180 + board_value(board),
            Some(BoardResult::Won(_)) => score -= 180 + board_value(board),
            Some(BoardResult::Draw) => {}
            None => {
                score += evaluate_small_board(&app.super_cells[board], ai) + board_value(board) / 2
            }
        }
    }

    score += evaluate_meta_lines(&app.super_results, ai);
    if let Some(active) = app.active_super_board {
        score -= evaluate_small_board(&app.super_cells[active], ai.other()) / 3;
    }
    score
}

fn evaluate_small_board(board: &[Option<Player>; 9], ai: Player) -> i32 {
    let mut score = 0;
    for line in WINS {
        let mut mine = 0;
        let mut theirs = 0;
        for cell in line {
            match board[cell] {
                Some(player) if player == ai => mine += 1,
                Some(_) => theirs += 1,
                None => {}
            }
        }
        score += line_score(mine, theirs);
    }
    score
}

fn evaluate_meta_lines(results: &[Option<BoardResult>; 9], ai: Player) -> i32 {
    let mut score = 0;
    for line in WINS {
        let mut mine = 0;
        let mut theirs = 0;
        for board in line {
            match results[board] {
                Some(BoardResult::Won(player)) if player == ai => mine += 1,
                Some(BoardResult::Won(_)) => theirs += 1,
                _ => {}
            }
        }
        score += line_score(mine, theirs) * 25;
    }
    score
}

fn line_score(mine: i32, theirs: i32) -> i32 {
    match (mine, theirs) {
        (3, 0) => 100,
        (2, 0) => 18,
        (1, 0) => 3,
        (0, 3) => -100,
        (0, 2) => -22,
        (0, 1) => -3,
        _ => 0,
    }
}

fn board_value(index: usize) -> i32 {
    match index {
        4 => 12,
        0 | 2 | 6 | 8 => 7,
        _ => 3,
    }
}

fn window_conf() -> Conf {
    Conf {
        window_title: "Tic Tac Toe Reactor".to_string(),
        window_width: 980,
        window_height: 760,
        high_dpi: true,
        sample_count: 4,
        ..Default::default()
    }
}

#[macroquad::main(window_conf)]
async fn main() {
    let mut app = App::default();

    loop {
        clear_background(Color::from_rgba(12, 16, 24, 255));
        draw_background();

        let buttons = draw_header(&app);
        if is_mouse_button_pressed(MouseButton::Left) {
            let (mx, my) = mouse_position();
            if clicked(buttons[0], mx, my) {
                app.set_game_mode(GameMode::Local);
            } else if clicked(buttons[1], mx, my) {
                app.set_game_mode(GameMode::Computer);
            } else if clicked(buttons[2], mx, my) {
                app.set_board_mode(BoardMode::Classic);
            } else if clicked(buttons[3], mx, my) {
                app.set_board_mode(BoardMode::Super);
            } else if clicked(buttons[4], mx, my) {
                app.reset();
            } else {
                app.handle_click();
            }
        }

        app.maybe_ai_move();

        match app.board_mode {
            BoardMode::Classic => draw_classic(&app),
            BoardMode::Super => draw_super(&app),
        }

        draw_footer(&app);
        next_frame().await;
    }
}

fn clicked(button: Button, x: f32, y: f32) -> bool {
    button.rect.contains(vec2(x, y))
}

fn draw_header(app: &App) -> [Button; 5] {
    let title = "Tic Tac Toe Reactor";
    draw_text_ex(
        title,
        34.0,
        48.0,
        TextParams {
            font_size: 36,
            color: WHITE,
            ..Default::default()
        },
    );
    draw_text_ex(
        &app.message,
        36.0,
        82.0,
        TextParams {
            font_size: 22,
            color: Color::from_rgba(163, 181, 207, 255),
            ..Default::default()
        },
    );

    let buttons = [
        Button {
            rect: Rect::new(520.0, 24.0, 86.0, 34.0),
            text: "Local",
        },
        Button {
            rect: Rect::new(614.0, 24.0, 72.0, 34.0),
            text: "AI",
        },
        Button {
            rect: Rect::new(704.0, 24.0, 86.0, 34.0),
            text: "3x3",
        },
        Button {
            rect: Rect::new(798.0, 24.0, 92.0, 34.0),
            text: "Super",
        },
        Button {
            rect: Rect::new(904.0, 24.0, 52.0, 34.0),
            text: "New",
        },
    ];

    for button in buttons {
        let selected = match button.text {
            "Local" => app.game_mode == GameMode::Local,
            "AI" => app.game_mode == GameMode::Computer,
            "3x3" => app.board_mode == BoardMode::Classic,
            "Super" => app.board_mode == BoardMode::Super,
            _ => false,
        };
        draw_button(button, selected);
    }

    buttons
}

fn draw_button(button: Button, selected: bool) {
    let (mx, my) = mouse_position();
    let hover = button.rect.contains(vec2(mx, my));
    let color = if selected {
        Color::from_rgba(33, 176, 137, 255)
    } else if hover {
        Color::from_rgba(49, 63, 84, 255)
    } else {
        Color::from_rgba(28, 36, 51, 255)
    };
    draw_rectangle_rounded(button.rect, 0.22, 8, color);
    draw_rectangle_rounded_lines(
        button.rect,
        0.22,
        8,
        1.5,
        Color::from_rgba(112, 132, 164, 160),
    );
    let dims = measure_text(button.text, None, 18, 1.0);
    draw_text_ex(
        button.text,
        button.rect.x + (button.rect.w - dims.width) / 2.0,
        button.rect.y + 22.5,
        TextParams {
            font_size: 18,
            color: WHITE,
            ..Default::default()
        },
    );
}

fn draw_background() {
    let t = get_time() as f32;
    for i in 0..18 {
        let x = ((i as f32 * 73.0 + t * 18.0).sin() * 0.5 + 0.5) * screen_width();
        let y = ((i as f32 * 41.0 + t * 9.0).cos() * 0.5 + 0.5) * screen_height();
        draw_circle(
            x,
            y,
            1.5 + (i % 4) as f32,
            Color::from_rgba(62, 201, 173, 45),
        );
    }
}

fn draw_classic(app: &App) {
    let rect = classic_rect();
    draw_panel(rect);
    draw_grid(rect, 4.0, Color::from_rgba(137, 158, 194, 255));
    for cell in 0..9 {
        if let Some(player) = app.classic[cell] {
            draw_mark_in_cell(rect, cell, player, anim_progress(app, 0, cell), 1.0);
        }
    }
    if let Some(BoardResult::Won(player)) = app.winner {
        draw_win_line(rect, app.classic, player);
    }
}

fn draw_super(app: &App) {
    let rect = super_rect();
    draw_panel(rect);
    let gap = 10.0;
    let board_size = (rect.w - gap * 2.0) / 3.0;

    for board in 0..9 {
        let br = nested_rect(rect, board, gap, board_size);
        let active = app.is_super_board_active(board)
            && app.super_results[board].is_none()
            && app.winner.is_none();
        if active {
            let pulse = (get_time() as f32 * 3.0).sin() * 0.5 + 0.5;
            draw_rectangle_rounded(
                inflate_rect(br, 5.0 + pulse * 2.0, 5.0 + pulse * 2.0),
                0.06,
                8,
                Color::from_rgba(33, 176, 137, 46),
            );
        }
        draw_rectangle_rounded(br, 0.05, 8, Color::from_rgba(18, 25, 36, 218));
        draw_grid(br, 1.7, Color::from_rgba(92, 111, 145, 255));

        for cell in 0..9 {
            if let Some(player) = app.super_cells[board][cell] {
                draw_mark_in_cell(br, cell, player, anim_progress(app, board, cell), 0.42);
            }
        }

        if let Some(result) = app.super_results[board] {
            match result {
                BoardResult::Won(player) => draw_big_overlay(br, player),
                BoardResult::Draw => draw_draw_overlay(br),
            }
        }
    }

    draw_grid(rect, 5.0, Color::from_rgba(207, 219, 238, 255));
}

fn draw_footer(app: &App) {
    let text = match (app.board_mode, app.game_mode) {
        (BoardMode::Classic, GameMode::Computer) => {
            "AI uses minimax with alpha-beta pruning. You are X."
        }
        (BoardMode::Classic, GameMode::Local) => "Local 3x3: take turns on the same machine.",
        (BoardMode::Super, GameMode::Computer) => {
            "Super mode sends the next move to the mini-board matching the cell just played."
        }
        (BoardMode::Super, GameMode::Local) => {
            "Super local: if the target mini-board is closed, the next player may play anywhere."
        }
    };
    draw_text_ex(
        text,
        34.0,
        screen_height() - 24.0,
        TextParams {
            font_size: 18,
            color: Color::from_rgba(153, 171, 199, 255),
            ..Default::default()
        },
    );
}

fn draw_panel(rect: Rect) {
    draw_rectangle_rounded(
        inflate_rect(rect, 16.0, 16.0),
        0.035,
        10,
        Color::from_rgba(9, 12, 18, 180),
    );
    draw_rectangle_rounded(rect, 0.025, 10, Color::from_rgba(21, 29, 42, 235));
    draw_rectangle_rounded_lines(rect, 0.025, 10, 2.0, Color::from_rgba(95, 122, 160, 150));
}

fn draw_grid(rect: Rect, thickness: f32, color: Color) {
    let cell = rect.w / 3.0;
    for i in 1..3 {
        let p = rect.x + cell * i as f32;
        draw_line(p, rect.y, p, rect.y + rect.h, thickness, color);
        let p = rect.y + cell * i as f32;
        draw_line(rect.x, p, rect.x + rect.w, p, thickness, color);
    }
}

fn inflate_rect(rect: Rect, x: f32, y: f32) -> Rect {
    Rect::new(rect.x - x, rect.y - y, rect.w + x * 2.0, rect.h + y * 2.0)
}

fn draw_rectangle_rounded(rect: Rect, roundness: f32, _segments: u8, color: Color) {
    let radius = (rect.w.min(rect.h) * roundness)
        .max(0.0)
        .min(rect.w.min(rect.h) / 2.0);
    draw_rectangle(
        rect.x + radius,
        rect.y,
        rect.w - radius * 2.0,
        rect.h,
        color,
    );
    draw_rectangle(
        rect.x,
        rect.y + radius,
        rect.w,
        rect.h - radius * 2.0,
        color,
    );
    draw_circle(rect.x + radius, rect.y + radius, radius, color);
    draw_circle(rect.x + rect.w - radius, rect.y + radius, radius, color);
    draw_circle(rect.x + radius, rect.y + rect.h - radius, radius, color);
    draw_circle(
        rect.x + rect.w - radius,
        rect.y + rect.h - radius,
        radius,
        color,
    );
}

fn draw_rectangle_rounded_lines(
    rect: Rect,
    roundness: f32,
    _segments: u8,
    thickness: f32,
    color: Color,
) {
    let radius = (rect.w.min(rect.h) * roundness)
        .max(0.0)
        .min(rect.w.min(rect.h) / 2.0);
    draw_line(
        rect.x + radius,
        rect.y,
        rect.x + rect.w - radius,
        rect.y,
        thickness,
        color,
    );
    draw_line(
        rect.x + radius,
        rect.y + rect.h,
        rect.x + rect.w - radius,
        rect.y + rect.h,
        thickness,
        color,
    );
    draw_line(
        rect.x,
        rect.y + radius,
        rect.x,
        rect.y + rect.h - radius,
        thickness,
        color,
    );
    draw_line(
        rect.x + rect.w,
        rect.y + radius,
        rect.x + rect.w,
        rect.y + rect.h - radius,
        thickness,
        color,
    );
    draw_circle_lines(rect.x + radius, rect.y + radius, radius, thickness, color);
    draw_circle_lines(
        rect.x + rect.w - radius,
        rect.y + radius,
        radius,
        thickness,
        color,
    );
    draw_circle_lines(
        rect.x + radius,
        rect.y + rect.h - radius,
        radius,
        thickness,
        color,
    );
    draw_circle_lines(
        rect.x + rect.w - radius,
        rect.y + rect.h - radius,
        radius,
        thickness,
        color,
    );
}

fn draw_mark_in_cell(rect: Rect, cell: usize, player: Player, progress: f32, scale: f32) {
    let cell_size = rect.w / 3.0;
    let cx = rect.x + (cell % 3) as f32 * cell_size + cell_size / 2.0;
    let cy = rect.y + (cell / 3) as f32 * cell_size + cell_size / 2.0;
    let radius = cell_size * 0.28 * scale;
    let p = ease_out_back(progress.clamp(0.0, 1.0));
    match player {
        Player::X => {
            let len = radius * p;
            let color = Color::from_rgba(89, 214, 255, 255);
            draw_line(
                cx - len,
                cy - len,
                cx + len,
                cy + len,
                6.0 * scale.max(0.6),
                color,
            );
            draw_line(
                cx + len,
                cy - len,
                cx - len,
                cy + len,
                6.0 * scale.max(0.6),
                color,
            );
        }
        Player::O => {
            draw_circle_lines(
                cx,
                cy,
                radius * p,
                6.0 * scale.max(0.6),
                Color::from_rgba(255, 193, 92, 255),
            );
        }
    }
}

fn draw_big_overlay(rect: Rect, player: Player) {
    let color = match player {
        Player::X => Color::from_rgba(89, 214, 255, 54),
        Player::O => Color::from_rgba(255, 193, 92, 54),
    };
    draw_rectangle_rounded(rect, 0.05, 8, color);
    let dims = measure_text(player.label(), None, 86, 1.0);
    draw_text_ex(
        player.label(),
        rect.x + (rect.w - dims.width) / 2.0,
        rect.y + rect.h / 2.0 + dims.height / 2.0,
        TextParams {
            font_size: 86,
            color: WHITE,
            ..Default::default()
        },
    );
}

fn draw_draw_overlay(rect: Rect) {
    draw_rectangle_rounded(rect, 0.05, 8, Color::from_rgba(160, 174, 194, 38));
    let dims = measure_text("DRAW", None, 26, 1.0);
    draw_text_ex(
        "DRAW",
        rect.x + (rect.w - dims.width) / 2.0,
        rect.y + rect.h / 2.0 + dims.height / 2.0,
        TextParams {
            font_size: 26,
            color: Color::from_rgba(218, 226, 238, 255),
            ..Default::default()
        },
    );
}

fn draw_win_line(rect: Rect, board: [Option<Player>; 9], player: Player) {
    for line in WINS {
        if board[line[0]] == Some(player)
            && board[line[1]] == Some(player)
            && board[line[2]] == Some(player)
        {
            let a = cell_center(rect, line[0]);
            let b = cell_center(rect, line[2]);
            let p = ((get_time() as f32 * 2.8).sin() * 0.5 + 0.5) * 2.0;
            draw_line(
                a.x,
                a.y,
                b.x,
                b.y,
                12.0 + p,
                Color::from_rgba(33, 176, 137, 235),
            );
        }
    }
}

fn anim_progress(app: &App, board: usize, cell: usize) -> f32 {
    app.animations
        .iter()
        .rev()
        .find(|anim| anim.board == board && anim.cell == cell)
        .map(|anim| {
            let age = (get_time() - anim.born) as f32;
            (age / 0.22).clamp(0.0, 1.0)
        })
        .unwrap_or(1.0)
}

fn ease_out_back(t: f32) -> f32 {
    let c1 = 1.70158;
    let c3 = c1 + 1.0;
    1.0 + c3 * (t - 1.0).powi(3) + c1 * (t - 1.0).powi(2)
}

fn classic_rect() -> Rect {
    let size = screen_height().min(screen_width()) * 0.66;
    Rect::new((screen_width() - size) / 2.0, 138.0, size, size)
}

fn super_rect() -> Rect {
    let size = (screen_height() - 174.0).min(screen_width() - 80.0);
    Rect::new((screen_width() - size) / 2.0, 128.0, size, size)
}

fn nested_rect(rect: Rect, index: usize, gap: f32, size: f32) -> Rect {
    Rect::new(
        rect.x + (index % 3) as f32 * (size + gap),
        rect.y + (index / 3) as f32 * (size + gap),
        size,
        size,
    )
}

fn cell_center(rect: Rect, cell: usize) -> Vec2 {
    let cell_size = rect.w / 3.0;
    vec2(
        rect.x + (cell % 3) as f32 * cell_size + cell_size / 2.0,
        rect.y + (cell / 3) as f32 * cell_size + cell_size / 2.0,
    )
}

fn hit_classic_cell(x: f32, y: f32) -> Option<usize> {
    hit_cell(classic_rect(), x, y)
}

fn hit_super_cell(x: f32, y: f32) -> Option<(usize, usize)> {
    let rect = super_rect();
    if !rect.contains(vec2(x, y)) {
        return None;
    }
    let gap = 10.0;
    let board_size = (rect.w - gap * 2.0) / 3.0;
    for board in 0..9 {
        let br = nested_rect(rect, board, gap, board_size);
        if let Some(cell) = hit_cell(br, x, y) {
            return Some((board, cell));
        }
    }
    None
}

fn hit_cell(rect: Rect, x: f32, y: f32) -> Option<usize> {
    if !rect.contains(vec2(x, y)) {
        return None;
    }
    let cell = rect.w / 3.0;
    let col = ((x - rect.x) / cell).floor() as usize;
    let row = ((y - rect.y) / cell).floor() as usize;
    (row < 3 && col < 3).then_some(row * 3 + col)
}
