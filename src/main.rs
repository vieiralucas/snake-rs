use std::convert::TryInto;
use std::io::{stdout, Read, Write};
use std::thread;
use std::time::Duration;
use termion::async_stdin;
use termion::raw::IntoRawMode;

#[derive(Clone, Copy)]
struct Vec2 {
    x: i16,
    y: i16,
}

impl Vec2 {
    fn add(self, vec: Vec2) -> Self {
        Vec2 {
            x: self.x + vec.x,
            y: self.y + vec.y,
        }
    }
}

#[derive(Clone, Copy)]
struct Snake {
    dir: Vec2,
    head: Vec2,
}

impl Snake {
    fn new() -> Self {
        Self {
            head: Vec2 { x: 2, y: 2 },
            dir: Vec2 { x: 1, y: 0 },
        }
    }

    fn go_left(self) -> Self {
        if self.dir.x == 1 {
            return self;
        }

        Self {
            head: self.head,
            dir: Vec2 { x: -1, y: 0 },
        }
    }

    fn go_up(self) -> Self {
        if self.dir.y == 1 {
            return self;
        }

        Self {
            head: self.head,
            dir: Vec2 { x: 0, y: -1 },
        }
    }

    fn go_down(self) -> Self {
        if self.dir.y == -1 {
            return self;
        }

        Self {
            head: self.head,
            dir: Vec2 { x: 0, y: 1 },
        }
    }

    fn go_right(self) -> Self {
        if self.dir.x == -1 {
            return self;
        }

        Self {
            head: self.head,
            dir: Vec2 { x: 1, y: 0 },
        }
    }

    fn update(self) -> Self {
        Self {
            head: self.head.add(self.dir),
            dir: self.dir,
        }
    }
}

fn main() {
    let stdout = stdout();
    let mut stdout = stdout.lock().into_raw_mode().unwrap();
    let mut stdin = async_stdin().bytes();

    write!(stdout, "{}{}", termion::clear::All, termion::cursor::Hide).unwrap();

    let mut snake = Snake::new();
    loop {
        write!(stdout, "{}", termion::clear::CurrentLine).unwrap();

        let b = stdin.next();
        match b {
            Some(Ok(b'q')) => break,
            Some(Ok(b'h')) => snake = snake.go_left(),
            Some(Ok(b'j')) => snake = snake.go_down(),
            Some(Ok(b'k')) => snake = snake.go_up(),
            Some(Ok(b'l')) => snake = snake.go_right(),
            _ => {}
        };

        write!(
            stdout,
            "{}{}██",
            termion::clear::All,
            termion::cursor::Goto(
                snake.head.x.try_into().unwrap_or(0),
                snake.head.y.try_into().unwrap_or(0)
            ),
        )
        .unwrap();
        stdout.flush().unwrap();
        thread::sleep(Duration::from_millis(500));

        snake = snake.update();
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
