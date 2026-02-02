use game_2048::{Down, Enter, Game, Left, Right, Up};
use gpui::{
    App, AppContext, Application, Bounds, KeyBinding, WindowBounds, WindowOptions, px, size,
};

fn main() {
    Application::new().run(|cx: &mut App| {
        cx.bind_keys([
            KeyBinding::new("up", Up, None),
            KeyBinding::new("left", Left, None),
            KeyBinding::new("down", Down, None),
            KeyBinding::new("right", Right, None),
            KeyBinding::new("w", Up, None),
            KeyBinding::new("a", Left, None),
            KeyBinding::new("s", Down, None),
            KeyBinding::new("d", Right, None),
            KeyBinding::new("enter", Enter, None),
        ]);

        let bounds = Bounds::centered(None, size(px(500.), px(600.0)), cx);
        cx.open_window(
            WindowOptions {
                window_bounds: Some(WindowBounds::Windowed(bounds)),
                ..Default::default()
            },
            |_, cx| cx.new(Game::new),
        )
        .unwrap();
    });
}

#[test]
fn test_swap() {
    for _ in 0..5 {
        let mut data = rand::random_iter().take(16).collect::<Vec<i32>>();

        let tran = (0..16)
            .map(|i| data[(i % 4) * 4 + i / 4])
            .collect::<Vec<_>>();

        data.swap(1, 4);
        data.swap(2, 8);
        data.swap(3, 12);
        data.swap(6, 9);
        data.swap(7, 13);
        data.swap(11, 14);

        assert_eq!(data, tran);
    }
}
