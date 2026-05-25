use std::io::{self, Read, Write};
use std::process::Command;
use std::sync::{Arc, Mutex, OnceLock};
use std::thread;
use std::time::{Duration, Instant};

const CTRL_C: u8 = 3;
const ESC: u8 = 27;
const BACKSPACE: u8 = 8;
const DELETE: u8 = 127;
const CR: u8 = b'\r';
const LF: u8 = b'\n';

static ORIGINAL_TTY: OnceLock<String> = OnceLock::new();

// POSIX signal numbers (stable across macOS and Linux).
const SIGHUP: i32 = 1;
const SIGQUIT: i32 = 3;
const SIGTERM: i32 = 15;

// Restore the terminal when killed by a signal. Drop won't run in that case.
// `unset_raw_mode` isn't technically async-signal-safe (it forks a shell), but in
// practice this is safe here: no allocator-holding thread races in our workload.
unsafe extern "C" fn restore_terminal_on_signal(_: i32) {
    unset_raw_mode();
    // Use the raw write() syscall — avoids the stdio Mutex that might be held
    // by the thread that was interrupted.
    unsafe extern "C" {
        fn write(fd: i32, buf: *const u8, count: usize) -> isize;
    }
    let seq = b"\r\x1b[?25h";
    unsafe { write(1, seq.as_ptr(), seq.len()) };
    std::process::exit(1);
}

struct TerminalGuard;

impl TerminalGuard {
    fn activate<W: Write>(out: &mut W) -> io::Result<Self> {
        set_raw_mode();
        // Install signal handlers so SIGTERM/SIGHUP/SIGQUIT restore the terminal
        // before the process dies. Drop won't run for signals, only for
        // normal exit and panics (unwind). SIGKILL cannot be handled.
        unsafe extern "C" {
            fn signal(
                signum: i32,
                handler: unsafe extern "C" fn(i32),
            ) -> unsafe extern "C" fn(i32);
        }
        unsafe {
            signal(SIGHUP, restore_terminal_on_signal);
            signal(SIGQUIT, restore_terminal_on_signal);
            signal(SIGTERM, restore_terminal_on_signal);
        }
        write!(out, "\x1b[?25l")?;
        out.flush()?;
        Ok(Self)
    }
}

impl Drop for TerminalGuard {
    fn drop(&mut self) {
        unset_raw_mode();
        let mut stdout = io::stdout();
        let _ = write!(stdout, "\r\x1b[?25h");
        let _ = stdout.flush();
    }
}

fn set_raw_mode() {
    if ORIGINAL_TTY.get().is_none()
        && let Ok(output) = Command::new("sh")
            .arg("-c")
            .arg("stty -g < /dev/tty")
            .output()
        && let Ok(state) = String::from_utf8(output.stdout)
    {
        let _ = ORIGINAL_TTY.set(state.trim().to_string());
    }

    let _ = Command::new("sh")
        .arg("-c")
        .arg("stty raw -echo < /dev/tty")
        .status();
}

fn unset_raw_mode() {
    if let Some(state) = ORIGINAL_TTY.get() {
        let cmd = format!("stty {} < /dev/tty", state);
        let _ = Command::new("sh").arg("-c").arg(&cmd).status();
    } else {
        let _ = Command::new("sh")
            .arg("-c")
            .arg("stty sane < /dev/tty")
            .status();
    }
}

fn render_line<W: Write>(
    out: &mut W,
    target: &[char],
    typed: &[char],
    highlight_cursor: bool,
) -> io::Result<()> {
    write!(out, "\x1b[2K")?;
    for (i, &ch) in target.iter().enumerate() {
        if i < typed.len() {
            if typed[i] == ch {
                write!(out, "\x1b[32m{}\x1b[0m", ch)?;
            } else if ch == ' ' {
                write!(out, "\x1b[31m_\x1b[0m")?;
            } else {
                write!(out, "\x1b[31m{}\x1b[0m", ch)?;
            }
        } else if highlight_cursor && i == typed.len() {
            write!(out, "\x1b[7m{}\x1b[0m", ch)?;
        } else {
            write!(out, "\x1b[2m{}\x1b[0m", ch)?;
        }
    }
    Ok(())
}

fn render_metrics_line<W: Write>(
    out: &mut W,
    typed_len: usize,
    accumulated_errors: usize,
    total_keystrokes: usize,
    start: Option<Instant>,
) -> io::Result<()> {
    write!(out, "\x1b[2K\x1b[2m")?;

    match start {
        None => write!(out, "wpm: -   errors: 0   acc: -   time: -")?,
        Some(t) => {
            let elapsed = t.elapsed().as_secs_f64();
            let wpm = if elapsed >= 0.5 {
                (typed_len as f64 / 5.0) / (elapsed / 60.0)
            } else {
                0.0
            };
            let acc = if total_keystrokes == 0 {
                100.0
            } else {
                total_keystrokes.saturating_sub(accumulated_errors) as f64
                    / total_keystrokes as f64
                    * 100.0
            };
            write!(
                out,
                "wpm: {:.0}   errors: {}   acc: {:.0}%   time: {:.1}s",
                wpm, accumulated_errors, acc, elapsed
            )?;
        }
    }

    write!(out, "\x1b[0m")?;
    Ok(())
}

// Redraws the text line only; leaves cursor at start of text line.
fn render_text<W: Write>(out: &mut W, target: &[char], typed: &[char]) -> io::Result<()> {
    render_line(out, target, typed, true)?;
    write!(out, "\r")?;
    out.flush()
}

// Redraws both lines; leaves cursor at start of text line.
fn render_full<W: Write>(
    out: &mut W,
    target: &[char],
    typed: &[char],
    accumulated_errors: usize,
    total_keystrokes: usize,
    start: Option<Instant>,
) -> io::Result<()> {
    render_line(out, target, typed, true)?;
    write!(out, "\n\r")?;
    render_metrics_line(out, typed.len(), accumulated_errors, total_keystrokes, start)?;
    write!(out, "\r\x1b[A")?;
    out.flush()
}

fn render_final<W: Write>(
    out: &mut W,
    target: &[char],
    typed: &[char],
    accumulated_errors: usize,
    total_keystrokes: usize,
    start: Option<Instant>,
) -> io::Result<()> {
    render_line(out, target, typed, false)?;
    write!(out, "\n\r")?;
    render_metrics_line(out, typed.len(), accumulated_errors, total_keystrokes, start)?;
    write!(out, "\n\r")?;
    out.flush()
}

struct State {
    typed: Vec<char>,
    accumulated_errors: usize,
    total_keystrokes: usize,
    start: Option<Instant>,
    done: bool,
}

pub fn render_loop(target: &str) -> io::Result<()> {
    let target: Vec<char> = target.chars().collect();
    let mut stdin = io::stdin();
    let mut stdout = io::stdout();

    let state = Arc::new(Mutex::new(State {
        typed: Vec::new(),
        accumulated_errors: 0,
        total_keystrokes: 0,
        start: None,
        done: false,
    }));

    let _guard = TerminalGuard::activate(&mut stdout)?;

    {
        let s = state.lock().unwrap();
        render_full(&mut stdout, &target, &s.typed, 0, 0, None)?;
    }

    // Background thread: redraws the metrics line every 100ms independent of keypresses.
    // Acquires the same lock as the main thread so stdout writes never interleave.
    let state_ref = Arc::clone(&state);
    let metrics_thread = thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_millis(100));
            let s = state_ref.lock().unwrap();
            if s.done {
                break;
            }
            let mut out = io::stdout();
            let _ = write!(out, "\n\r");
            let _ = render_metrics_line(
                &mut out,
                s.typed.len(),
                s.accumulated_errors,
                s.total_keystrokes,
                s.start,
            );
            let _ = write!(out, "\r\x1b[A");
            let _ = out.flush();
        }
    });

    let mut buf = [0u8; 1];

    while stdin.read(&mut buf)? > 0 {
        let b = buf[0];

        if b == CTRL_C || b == ESC {
            break;
        } else if b == CR || b == LF {
            continue;
        }

        let mut s = state.lock().unwrap();

        if b == BACKSPACE || b == DELETE {
            s.typed.pop();
        } else if b.is_ascii() {
            if s.start.is_none() {
                s.start = Some(Instant::now());
            }
            let pos = s.typed.len();
            if pos < target.len() && b as char != target[pos] {
                s.accumulated_errors += 1;
            }
            s.total_keystrokes += 1;
            s.typed.push(b as char);
        }

        render_text(&mut stdout, &target, &s.typed)?;
        let finished = s.typed.len() >= target.len();
        drop(s);

        if finished {
            break;
        }
    }

    state.lock().unwrap().done = true;
    metrics_thread.join().unwrap();

    let s = state.lock().unwrap();
    render_final(
        &mut stdout,
        &target,
        &s.typed,
        s.accumulated_errors,
        s.total_keystrokes,
        s.start,
    )?;

    Ok(())
}
