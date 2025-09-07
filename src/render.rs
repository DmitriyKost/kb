use std::io::{self, Read, Write};
use std::process::Command;

const CTRL_C: u8 = 3;
const ESC: u8 = 27;
const BACKSPACE: u8 = 8;
const DELETE: u8 = 127;
const CR: u8 = b'\r';
const LF: u8 = b'\n';

fn set_raw_mode() {
    let _ = Command::new("sh")
        .arg("-c")
        .arg("stty raw -echo < /dev/tty")
        .status();
}

fn unset_raw_mode() {
    let _ = Command::new("sh")
        .arg("-c")
        .arg("stty -raw echo < /dev/tty")
        .status();
}

fn render_status<W: Write>(out: &mut W, target: &[char], typed: &[char]) -> io::Result<()> {
    write!(out, "\x1b[2K")?;
    for (i, &ch) in target.iter().enumerate() {
        if i < typed.len() {
            if typed[i] == ch {
                write!(out, "\x1b[32m{}\x1b[0m", ch)?;
            } else {
                if ch == ' ' {
                    write!(out, "\x1b[31m_\x1b[0m")?;
                } else {
                    write!(out, "\x1b[31m{}\x1b[0m", ch)?;
                }
            }
        } else if i == typed.len() {
            write!(out, "\x1b[7m{}\x1b[0m", ch)?;
        } else {
            write!(out, "\x1b[2m{}\x1b[0m", ch)?;
        }
    }
    write!(out, "\r")?;
    out.flush()
}

fn render_final<W: Write>(out: &mut W, target: &[char], typed: &[char]) -> io::Result<()> {
    for (i, &ch) in target.iter().enumerate() {
        if i < typed.len() {
            if typed[i] == ch {
                write!(out, "\x1b[32m{}\x1b[0m", ch)?;
            } else {
                if ch == ' ' {
                    write!(out, "\x1b[31m_\x1b[0m")?;
                } else {
                    write!(out, "\x1b[31m{}\x1b[0m", ch)?;
                }
            }
        } else {
            write!(out, "\x1b[2m{}\x1b[0m", ch)?;
        }
    }
    writeln!(out)?;
    out.flush()
}

pub fn render_loop(target: &str) -> io::Result<()> {
    let target: Vec<char> = target.chars().collect();
    let mut typed: Vec<char> = Vec::new();
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    set_raw_mode();
    write!(stdout, "\x1b[?25l")?;
    render_status(&mut stdout, &target, &typed)?;

    let mut buf = [0u8; 1];

    while stdin.read(&mut buf)? > 0 {
        let b = buf[0];

        if b == CTRL_C || b == ESC {
            break;
        } else if b == BACKSPACE || b == DELETE {
            if !typed.is_empty() {
                typed.pop();
            }
        } else if b == CR || b == LF {
            continue;
        } else if b.is_ascii() {
            typed.push(b as char);
        }

        render_status(&mut stdout, &target, &typed)?;
        if typed.len() >= target.len() {
            break;
        }
    }
    unset_raw_mode();
    render_final(&mut stdout, &target, &typed)?;
    write!(stdout, "\x1b[?25h")?;
    Ok(())
}
