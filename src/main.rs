use rand::Rng;
use std::convert::TryInto;
use std::io::{stdout, Read, Write};
use std::ops;
use std::thread;
use std::time::Duration;
use termion::async_stdin;
use termion::raw::IntoRawMode;
use termion::terminal_size;

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
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

    fn render(&self, w: &mut dyn Write) {
        let x: Option<u16> = (self.x).try_into().ok();
        let y: Option<u16> = (self.y).try_into().ok();

        match (x, y) {
            (Some(x), Some(y)) => write!(w, "{}  ", termion::cursor::Goto(x + 1, y + 1))
                .expect("could not render pixel"),
            _ => {}
        }
    }
}

impl ops::AddAssign<Vec2> for Vec2 {
    fn add_assign(&mut self, rhs: Vec2) {
        self.x += rhs.x;
        self.y += rhs.y;
    }
}

impl ops::Add<Vec2> for Vec2 {
    type Output = Vec2;

    fn add(self, rhs: Vec2) -> Self {
        Self {
            x: self.x + rhs.x,
            y: self.y + rhs.y,
        }
    }
}

impl ops::Mul<Vec2> for Vec2 {
    type Output = Vec2;

    fn mul(self, rhs: Vec2) -> Self {
        Self {
            x: self.x * rhs.x,
            y: self.y * rhs.y,
        }
    }
}

#[derive(Debug)]
struct Snake {
    dir: Vec2,
    head: Vec2,
    tail: Vec<Vec2>,
}

impl Snake {
    fn new() -> Self {
        Self {
            head: Vec2 { x: 2, y: 1 },
            dir: Vec2::right(),
            tail: vec![],
        }
    }

    fn go_left(&mut self) {
        if self.dir != Vec2::right() {
            self.dir = Vec2::left();
        }
    }

    fn go_up(&mut self) {
        if self.dir != Vec2::down() {
            self.dir = Vec2::up();
        }
    }

    fn go_down(&mut self) {
        if self.dir != Vec2::up() {
            self.dir = Vec2::down();
        }
    }

    fn go_right(&mut self) {
        if self.dir != Vec2::left() {
            self.dir = Vec2::right();
        }
    }

    fn update(&mut self) {
        for i in (0..self.tail.len()).rev() {
            let mut prev = self.head;
            if i > 0 {
                prev = self.tail[i - 1];
            }

            self.tail[i].x = prev.x;
            self.tail[i].y = prev.y;
        }
        self.head += self.dir;
    }

    fn grow(&mut self) {
        let mut last_part = self.head;
        if self.tail.len() > 0 {
            last_part = self.tail[self.tail.len() - 1]
        }

        if self.dir == Vec2::left() {
            self.tail.push(last_part + Vec2::right())
        } else if self.dir == Vec2::down() {
            self.tail.push(last_part + Vec2::up())
        } else if self.dir == Vec2::up() {
            self.tail.push(last_part + Vec2::down())
        } else if self.dir == Vec2::right() {
            self.tail.push(last_part + Vec2::left())
        }
    }

    fn render(&self, w: &mut dyn Write) {
        self.head.render(w);
        for t in self.tail.iter() {
            t.render(w);
        }
    }
}

struct Game {
    snake: Snake,
    apple: Vec2,
    w: u16,
    h: u16,
    game_over: Option<(Vec2, bool)>,
}

impl Game {
    fn new(w: u16, h: u16) -> Self {
        Self {
            snake: Snake::new(),
            apple: Game::spawn_apple(w, h),
            w: w,
            h: h,
            game_over: None,
        }
    }

    fn spawn_apple(w: u16, h: u16) -> Vec2 {
        Vec2::random(Vec2::new(2, 1), Vec2::new((w as i16 - 2) / 2, h as i16 - 1)) * Vec2::new(2, 1)
    }

    fn snake_hits_walls(&self) -> Option<Vec2> {
        let future_head = self.snake.head + self.snake.dir;

        if future_head.x == 0 || future_head.x == (self.w - 2) as i16 {
            return Some(future_head);
        }

        if future_head.y == 0 || future_head.y == (self.h - 1) as i16 {
            return Some(future_head);
        }

        None
    }

    fn snake_hits_itself(&self) -> Option<Vec2> {
        let future_head = self.snake.head + self.snake.dir;

        for t in self.snake.tail.iter() {
            if t == &future_head {
                return Some(future_head);
            }
        }

        None
    }

    fn update(&mut self, input: Option<char>) {
        if self.game_over.is_none() {
            match input {
                Some('h') => self.snake.go_left(),
                Some('j') => self.snake.go_down(),
                Some('k') => self.snake.go_up(),
                Some('l') => self.snake.go_right(),
                _ => {}
            };
        }

        if self.game_over.is_none() {
            self.game_over = self
                .snake_hits_walls()
                .or(self.snake_hits_itself())
                .map(|pos| (pos, false));
        }

        if self.game_over.is_some() {
            self.game_over = self.game_over.map(|(pos, blink)| (pos, !blink));
            return;
        }

        if self.snake.head == self.apple {
            self.snake.grow();
            self.apple = Game::spawn_apple(self.w, self.h);
        }

        self.snake.update();
    }

    fn render(&self, w: &mut dyn Write) {
        write!(w, "{}", termion::clear::All).expect("could not clear screen");
        write!(w, "{}", termion::color::Bg(termion::color::White))
            .expect("could not set background color");
        for x in (0..self.w).step_by(2) {
            write!(w, "{}  ", termion::cursor::Goto(x + 1, 1))
                .expect("could not render border pixel");
            write!(w, "{}  ", termion::cursor::Goto(x + 1, self.h))
                .expect("could not render border pixel");
        }
        for y in 0..self.h {
            write!(w, "{}  ", termion::cursor::Goto(1, y)).expect("could not render border pixel");
            write!(w, "{}  ", termion::cursor::Goto(self.w - 1, y))
                .expect("could not render border pixel");
        }

        self.snake.render(w);
        self.apple.render(w);

        match self.game_over {
            Some((pos, false)) => {
                write!(
                    w,
                    "{}{}",
                    termion::color::Fg(termion::color::White),
                    termion::color::Bg(termion::color::Black),
                )
                .expect("could not set colors for rendering collision");
                pos.render(w);
            }
            Some((pos, true)) => {
                write!(
                    w,
                    "{}{}",
                    termion::color::Fg(termion::color::Black),
                    termion::color::Bg(termion::color::White),
                )
                .expect("could not set colors for rendering collision");
                pos.render(w);
            }
            _ => {}
        }

        write!(
            w,
            "{}{}{}q to quit, hjkl to move{}{}",
            termion::cursor::Goto(4, self.h),
            termion::color::Fg(termion::color::Black),
            termion::color::Bg(termion::color::White),
            termion::color::Fg(termion::color::Reset),
            termion::color::Bg(termion::color::Reset),
        )
        .expect("could not render instructions");

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

    let mut game = Game::new(u16::min(200, w), u16::min(100, h));

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
        "{}{}{}{}{}",
        termion::color::Fg(termion::color::Reset),
        termion::color::Bg(termion::color::Reset),
        termion::clear::All,
        termion::cursor::Goto(1, 1),
        termion::cursor::Show
    )
    .unwrap();
}
