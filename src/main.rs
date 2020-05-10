use rand::Rng;
use std::convert::TryInto;
use std::io::{stderr, stdout, Read, Write};
use std::thread;
use std::time::Duration;
use termion::async_stdin;
use termion::raw::IntoRawMode;
use termion::terminal_size;

#[derive(PartialEq, Eq, Debug)]
struct Vec2 {
    x: i16,
    y: i16,
}

impl Vec2 {
    fn new(x: i16, y: i16) -> Self {
        Self { x: x, y: y }
    }

    fn random(min: Vec2, max: Vec2) -> Self {
        let mut rng = rand::thread_rng();

        Self::new(rng.gen_range(min.x, max.x), rng.gen_range(min.y, max.y))
    }

    fn left() -> Self {
        Self::new(-2, 0)
    }

    fn down() -> Self {
        Self::new(0, 1)
    }

    fn up() -> Self {
        Self::new(0, -1)
    }

    fn right() -> Self {
        Self::new(2, 0)
    }

    fn add(&self, vec: &Vec2) -> Self {
        Vec2 {
            x: self.x + vec.x,
            y: self.y + vec.y,
        }
    }

    fn render(&self, w: &mut dyn Write) {
        let x: Option<u16> = (self.x).try_into().ok();
        let y: Option<u16> = (self.y).try_into().ok();

        match (x, y) {
            (Some(x), Some(y)) => write!(w, "{}██", termion::cursor::Goto(x + 1, y + 1))
                .expect("could not render pixel"),
            _ => {}
        }
    }
}

struct Snake {
    dir: Vec2,
    head: Vec2,
}

impl Snake {
    fn new() -> Self {
        Self {
            head: Vec2 { x: 2, y: 1 },
            dir: Vec2::right(),
        }
    }

    fn go_left(&mut self) {
        if self.dir.x != 1 {
            self.dir = Vec2::left();
        }
    }

    fn go_up(&mut self) {
        if self.dir.y != -1 {
            self.dir = Vec2::up();
        }
    }

    fn go_down(&mut self) {
        if self.dir.y != 1 {
            self.dir = Vec2::down();
        }
    }

    fn go_right(&mut self) {
        if self.dir.x != -1 {
            self.dir = Vec2::right();
        }
    }

    fn update(&mut self) {
        self.head = self.head.add(&self.dir);
    }

    fn render(&self, w: &mut dyn Write) {
        self.head.render(w);
    }
}

struct Game {
    snake: Snake,
    apple: Vec2,
    w: u16,
    h: u16,
}

impl Game {
    fn new(w: u16, h: u16) -> Self {
        Self {
            snake: Snake::new(),
            apple: Vec2::random(Vec2::new(0, 0), Vec2::new(w as i16, h as i16)),
            w: w,
            h: h,
        }
    }

    fn spawn_apple(&mut self) {
        self.apple = Vec2::random(Vec2::new(0, 0), Vec2::new(self.w as i16, self.h as i16));
    }

    fn update(&mut self, input: Option<char>) {
        match input {
            Some('h') => self.snake.go_left(),
            Some('j') => self.snake.go_down(),
            Some('k') => self.snake.go_up(),
            Some('l') => self.snake.go_right(),
            _ => {}
        };

        write!(stderr(), "{:#?}", self.apple);

        if self.snake.head == self.apple {
            self.spawn_apple();
        }

        self.snake.update();
    }

    fn render(&self, w: &mut dyn Write) {
        write!(w, "{}", termion::clear::All).expect("could not clear screen");
        for x in 0..self.w {
            write!(w, "{}██", termion::cursor::Goto(x + 1, 1))
                .expect("could not render border pixel");
            write!(w, "{}██", termion::cursor::Goto(x + 1, self.h - 1))
                .expect("could not render border pixel");
        }
        for y in 0..self.h {
            write!(w, "{}██", termion::cursor::Goto(1, y)).expect("could not render border pixel");
            write!(w, "{}██", termion::cursor::Goto(self.w - 1, y))
                .expect("could not render border pixel");
        }

        self.snake.render(w);
        self.apple.render(w);

        w.flush().expect("could not flush renderer");
    }
}

fn main() {
    let stdout = stdout();
    let mut stdout = stdout
        .lock()
        .into_raw_mode()
        .expect("could not enter raw mode");
    let mut stdin = async_stdin().bytes();

    write!(stdout, "{}", termion::cursor::Hide).expect("could not hide cursor");

    let (w, h) = terminal_size().expect("could not get terminal size");

    let mut game = Game::new(u16::min(100, w), u16::min(100, h));

    loop {
        let input = stdin.next().and_then(|res| res.ok()).map(|b| b as char);
        match input {
            Some('q') => break,
            _ => {}
        };
        game.update(input);

        game.render(&mut stdout);
        thread::sleep(Duration::from_millis(100));
    }
    write!(
        stdout,
        "{}{}{}",
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Show
    )
    .unwrap();
}
