use std::io::{stdout, Write};
use crossterm::{
    cursor::{DisableBlinking, EnableBlinking, MoveTo, RestorePosition, SavePosition},
    event::{self, Event, KeyCode},
    execute,
    style::{Color, Print, ResetColor, SetBackgroundColor, SetForegroundColor},
    terminal::{disable_raw_mode, enable_raw_mode, size, Clear, ClearType, SetSize},
};
use std::{io::Result, time};

fn main_loop() -> Result<()> {
    let mut global_buffer = Vec::new();
    let mut cursor = 0;
    let mut cursor_x = 0;
    let mut cursor_y = 0;

    loop {
        let mut stdout = stdout();
        let (width, height) = size()?;
        execute!(
            stdout,
            SetSize(width, height),
            Clear(ClearType::All),
            EnableBlinking,
        )?;

        let mut x = 0;
        let mut y = 0;
        for &byte in &global_buffer {
            if byte == b'\n' {
                x = 0;
                y += 1;
            } else {
                execute!(
                    stdout,
                    MoveTo(x as u16, y as u16),
                    SetForegroundColor(Color::White),
                    Print(byte as char),
                    ResetColor,
                )?;
                x += 1;
            }
        }
        execute!(
            stdout,
            MoveTo(cursor_x as u16, cursor_y as u16),
            SetBackgroundColor(Color::Blue),
            SetForegroundColor(Color::White),
            ResetColor,
        )?;
        let mut exit = false;

        if event::poll(time::Duration::from_millis(100))? {
            if let Event::Key(key_event) = event::read()? {
                match key_event.code {
                    KeyCode::Esc => {
                        exit = true;
                    }
                    KeyCode::Char(c) => {
                        global_buffer.insert(cursor, c as u8);
                        cursor += 1;
                        if c == '\n' {
                            cursor_y += 1;
                            cursor_x = 0;
                        } else {
                            cursor_x += 1;
                        }
                    }
                    KeyCode::Backspace => {
                        if cursor > 0 {
                            cursor -= 1;
                            global_buffer.remove(cursor);
                            if cursor_x > 0 {
                                cursor_x -= 1;
                            } else if cursor_y > 0 {
                                cursor_y -= 1;
                                //calculate x position
                                cursor_x = global_buffer
                                    .iter()
                                    .take(cursor)
                                    .rev()
                                    .take_while(|&&b| b != b'\n')
                                    .count();
                            }
                        }
                    }
                    KeyCode::Enter => {
                        global_buffer.insert(cursor, b'\n');
                        cursor += 1;
                        cursor_x = 0;
                        cursor_y += 1;
                    }
                    KeyCode::Left => {
                        if cursor_x > 0 {
                            cursor -= 1;
                            cursor_x -= 1;
                        }
                    }
                    KeyCode::Right => {
                        if cursor < global_buffer.len() && global_buffer[cursor] != b'\n' {
                            cursor += 1;
                            cursor_x += 1;
                        }
                    }
                    _ => {}
                }
            }
        }
        //calculate cursor position
        execute!(
            stdout,
            MoveTo(cursor_x as u16, cursor_y as u16),
            DisableBlinking,
        )?;

        if exit {
            return Ok(());
        }
    }
}

fn main() -> Result<()> {
    let (width, height) = size()?;
    execute!(stdout(), SavePosition)?;
    enable_raw_mode()?;
    let result = main_loop();
    disable_raw_mode()?;
    execute!(stdout(), RestorePosition)?;

    result
}
