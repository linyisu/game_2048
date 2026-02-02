use gpui::*;
use rand::prelude::*;
use std::{env, fs, time::Duration};

gpui::actions!(game, [Up, Down, Left, Right, Enter]);

fn get_font_color(value: u32) -> Rgba {
    if value <= 4 {
        rgb(0x776e65)
    } else {
        rgb(0xe7e7e7)
    }
}

fn get_font_size(value: u32) -> Pixels {
    if value == 0 {
        return px(0.0);
    }

    let digits = value.to_string().len() as f32;
    let size = (60.0 / (digits * 0.7)).min(36.0);

    px(size)
}

fn get_color(value: u32) -> Hsla {
    if value == 0 {
        return rgb(0xcdc1b4).into();
    }

    let power = (value as f32).log2();

    let hue = (30.0 + power * 20.0) % 360.0 / 360.0;
    let saturation = (0.5 + (power * 0.04)).min(0.9);
    let lightness = 0.45 + (0.35 * f32::powf(0.8, power - 1.0));

    hsla(hue, saturation, lightness, 1.0)
}

pub struct Game {
    score: u32,
    best_score: u32,
    datas: Vec<u32>,
    is_started: bool,
    is_game_over: bool,
    focus_handle: FocusHandle,
    spawn_count: u32,
    new_tiles: Vec<Option<usize>>,
}

impl Game {
    pub fn new(cx: &mut Context<Self>) -> Game {
        let mut config_path = env::current_dir().unwrap();
        config_path.push("config");
        let best_score = fs::read_to_string(&config_path)
            .ok()
            .and_then(|s| s.trim().parse().ok())
            .unwrap_or(0);
        Game {
            score: 0,
            best_score,
            is_started: false,
            is_game_over: false,
            datas: vec![0; 16],
            focus_handle: cx.focus_handle(),
            spawn_count: 0,
            new_tiles: Vec::new(),
        }
    }

    fn new_game(&mut self, _window: &mut Window, cx: &mut Context<Self>) {
        self.score = 0;
        self.is_started = true;
        self.new_tiles.clear();
        self.datas = vec![0; 16];
        self.is_game_over = false;
        self.spawn_tile(cx);
        self.spawn_tile(cx);
        cx.notify();
    }

    fn save_best_score(&self) {
        let mut config_path = env::current_dir().unwrap();
        config_path.push("config");
        if !config_path.exists() {
            fs::File::create(&config_path).ok();
        }
        fs::write(&config_path, self.best_score.to_string()).ok();
    }
}

impl Game {
    // about render
    fn render_box(&self, label: &'static str, value: u32) -> impl IntoElement {
        div()
            .bg(rgb(0xbbada0))
            .px_4()
            .py_1()
            .rounded_md()
            .flex()
            .flex_col()
            .items_center()
            .min_w(px(80.0))
            .child(div().text_xs().text_color(rgb(0xeee4da)).child(label))
            .child(
                div()
                    .text_lg()
                    .text_color(rgb(0xffffff))
                    .font_weight(FontWeight::BOLD)
                    .child(value.to_string()),
            )
    }

    fn render_grid(&self) -> impl IntoElement {
        div()
            .relative()
            .bg(rgb(0xbbada0))
            .p_3()
            .rounded_lg()
            .flex()
            .flex_col()
            .gap_3()
            .child(
                div().flex().flex_col().p(px(6.0)).gap(px(12.0)).children(
                    std::array::from_fn::<usize, 16, _>(|i| i)
                        .chunks(4)
                        .map(|_| {
                            div().flex().flex_row().gap(px(12.0)).children(
                                (0..4).map(|_| div().size(px(90.0)).bg(rgb(0xcdc1b4)).rounded_md()),
                            )
                        }),
                ),
            )
    }

    fn render_single_tile(&self, idx: usize, val: u32) -> impl IntoElement {
        let r = (idx / 4) as f32;
        let c = (idx % 4) as f32;

        let offset = 18.0;
        let step = 102.0;
        let base_top = offset + r * step;
        let base_left = offset + c * step;

        let tile_div = div()
            .absolute()
            .bg(get_color(val))
            .text_color(get_font_color(val))
            .font_weight(FontWeight::BOLD)
            .rounded_md()
            .flex()
            .justify_center()
            .items_center()
            .child(val.to_string());

        if self.new_tiles.contains(&Some(idx)) {
            tile_div
                .with_animation(
                    ("spawn", self.spawn_count),
                    Animation::new(Duration::from_millis(160)),
                    move |this, progress| {
                        let current_size = 90.0 * progress;
                        let compensation = (90.0 - current_size) / 2.0;

                        this.w(px(current_size))
                            .h(px(current_size))
                            .top(px(base_top + compensation))
                            .left(px(base_left + compensation))
                            .text_size(get_font_size(val) * progress)
                    },
                )
                .into_any_element()
        } else {
            tile_div
                .w(px(90.0))
                .h(px(90.0))
                .top(px(base_top))
                .left(px(base_left))
                .text_size(get_font_size(val))
                .into_any_element()
        }
    }

    fn render_tiles(&self) -> impl Iterator<Item = impl IntoElement> {
        self.datas
            .iter()
            .enumerate()
            .filter(|(_, val)| **val > 0)
            .map(|(idx, &val)| self.render_single_tile(idx, val))
    }
}
impl Game {
    // about core logic
    fn spawn_tile(&mut self, cx: &mut Context<Self>) {
        let mut rng = rand::rng();

        let mut nums: Vec<usize> = (0..16).filter(|&i| self.datas[i] == 0).collect();
        nums.shuffle(&mut rng);
        let idx = nums[0];
        self.datas[idx] = match rng.random_bool(0.9) {
            true => 2,
            false => 4,
        };
        self.spawn_count += 1;
        self.new_tiles.push(Some(idx));
        cx.notify();
    }

    fn transpose(&mut self) {
        // Without alloc
        self.datas.swap(1, 4);
        self.datas.swap(2, 8);
        self.datas.swap(3, 12);
        self.datas.swap(6, 9);
        self.datas.swap(7, 13);
        self.datas.swap(11, 14);
    }

    fn delete_zero(&mut self, pos: i32) -> bool {
        let mut flag = false;
        for i in 0..4 {
            for j in 0 - pos..4 - pos {
                if self.datas[((j.abs()) * 4 + i) as usize] == 0 {
                    for k in j + 1..4 - pos {
                        if self.datas[((k.abs()) * 4 + i) as usize] != 0 {
                            flag = true;
                            self.datas[((j.abs()) * 4 + i) as usize] =
                                self.datas[((k.abs()) * 4 + i) as usize];
                            self.datas[((k.abs()) * 4 + i) as usize] = 0;
                            break;
                        }
                    }
                }
            }
        }
        flag
    }

    fn merge(&mut self, dir: u32, pos: i32) -> bool {
        if dir == 1 {
            self.transpose();
        }
        let flag1 = self.delete_zero(pos);
        let mut flag2 = false;
        for i in 0..4 {
            for j in 0 - pos..3 - pos {
                if self.datas[((j.abs()) * 4 + i) as usize] != 0
                    && self.datas[((j.abs()) * 4 + i) as usize]
                        == self.datas[(((j + 1).abs()) * 4 + i) as usize]
                {
                    flag2 = true;
                    self.datas[((j.abs()) * 4 + i) as usize] <<= 1;
                    self.score = self
                        .score
                        .saturating_add(self.datas[((j.abs()) * 4 + i) as usize]);
                    (self.best_score < self.score).then(|| {
                        self.best_score = self.score;
                        self.save_best_score();
                    });
                    self.datas[(((j + 1).abs()) * 4 + i) as usize] = 0;
                }
            }
        }
        self.delete_zero(pos);
        if dir == 1 {
            self.transpose();
        }
        flag1 | flag2
    }
    fn check_fail(&mut self) -> bool {
        let count = self.datas.iter().filter(|&&x| x == 0).count();
        if count != 0 {
            false
        } else {
            for i in 0..16 {
                let row = i / 4;
                let col = i % 4;
            
                if col < 3 && self.datas[i] == self.datas[i + 1] {
                    return false;
                }
                if row < 3 && self.datas[i] == self.datas[i + 4] {
                    return false;
                }
            }
            true
        }
    }
}

impl Game {
    // about actions for keyboard and mouse
    fn move_up(&mut self, _: &Up, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_started {
            return;
        }
        self.new_tiles.clear();
        if self.merge(0, 0) {
            self.spawn_tile(cx);
        }
        if self.check_fail() {
            self.is_started = false;
            self.is_game_over = true;
        };
        cx.notify();
    }

    fn move_left(&mut self, _: &Left, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_started {
            return;
        }
        self.new_tiles.clear();
        if self.merge(1, 0) {
            self.spawn_tile(cx);
        }
        if self.check_fail() {
            self.is_started = false;
            self.is_game_over = true;
        };
        cx.notify();
    }

    fn move_down(&mut self, _: &Down, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_started {
            return;
        }
        self.new_tiles.clear();
        if self.merge(0, 3) {
            self.spawn_tile(cx);
        }
        if self.check_fail() {
            self.is_started = false;
            self.is_game_over = true;
        };
        cx.notify();
    }

    fn move_right(&mut self, _: &Right, _window: &mut Window, cx: &mut Context<Self>) {
        if !self.is_started {
            return;
        }
        self.new_tiles.clear();
        if self.merge(1, 3) {
            self.spawn_tile(cx);
        }
        if self.check_fail() {
            self.is_started = false;
            self.is_game_over = true;
        };
        cx.notify();
    }

    fn new_game_mouse(
        &mut self,
        _: &MouseDownEvent,
        _window: &mut Window,
        _cx: &mut Context<Self>,
    ) {
        self.new_game(_window, _cx);
    }

    fn new_game_keyboard(&mut self, _: &Enter, _window: &mut Window, _cx: &mut Context<Self>) {
        self.new_game(_window, _cx);
    }
}

impl Focusable for Game {
    fn focus_handle(&self, _: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}

impl Render for Game {
    fn render(&mut self, _window: &mut Window, cx: &mut Context<Self>) -> impl IntoElement {
        div()
            .flex()
            .flex_col()
            .size_full()
            .justify_center()
            .items_center()
            .bg(rgb(0xfaf8ef))
            .track_focus(&self.focus_handle)
            .on_action(cx.listener(Self::move_up))
            .on_action(cx.listener(Self::move_down))
            .on_action(cx.listener(Self::move_left))
            .on_action(cx.listener(Self::move_right))
            .on_action(cx.listener(Self::new_game_keyboard))
            .child(
                div()
                    .flex()
                    .w(px(420.0))
                    .justify_between()
                    .items_end()
                    .mb_4()
                    .child(
                        div()
                            .text_3xl()
                            .font_weight(FontWeight::BOLD)
                            .text_color(rgb(0x776e65))
                            .child("2048"),
                    )
                    .child(
                        div()
                            .flex()
                            .gap_2()
                            .child(self.render_box("SCORE", self.score))
                            .child(self.render_box("BEST", self.best_score)),
                    ),
            )
            .child(
                div().flex().w(px(420.0)).justify_end().mb_4().child(
                    div()
                        .id("new-game")
                        .px_4()
                        .py_2()
                        .bg(rgb(0x8f7a66))
                        .text_color(rgb(0xf9f6f2))
                        .rounded_md()
                        .font_weight(FontWeight::BOLD)
                        .on_mouse_down(MouseButton::Left, cx.listener(Self::new_game_mouse))
                        .child("New Game"),
                ),
            )
            .child(
                div()
                    .relative()
                    .child(self.render_grid())
                    .children(self.render_tiles())
                    .children(self.is_game_over.then(|| {
                        div()
                            .absolute()
                            .inset_0()
                            .bg(rgba(0xfaf8efcc))
                            .rounded_lg()
                            .flex()
                            .flex_col()
                            .justify_center()
                            .items_center()
                            .child(
                                div()
                                    .text_3xl()
                                    .font_weight(FontWeight::BOLD)
                                    .text_color(rgb(0x776e65))
                                    .child("Game Over!"),
                            )
                            .child(
                                div()
                                    .mt_4()
                                    .text_lg()
                                    .text_color(rgb(0x776e65))
                                    .child("Press Enter to Try Again"),
                            )
                    })),
            )
    }
}
