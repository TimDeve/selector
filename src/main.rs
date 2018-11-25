#[macro_use]
extern crate enum_primitive;
extern crate libc;
extern crate num;
extern crate termios;

use libc::STDIN_FILENO;
use num::FromPrimitive;
use std::io;
use std::io::Read;
use std::io::Write;
use std::process;
use termios::{cfmakeraw, tcsetattr, Termios, TCSANOW};

enum_from_primitive! {
    #[derive(Debug, PartialEq)]
    enum Keys {
        CtrlC = 3,
        Enter = 13,
        Escape = 27,
        Space = 32,
        J = 106,
        K = 107,
        Q = 113,
    }
}

#[derive(Debug, PartialEq)]
struct File {
    status: String,
    path: String,
    isSelected: bool,
}

fn main() {
    let original_termios = Termios::from_fd(STDIN_FILENO).unwrap();

    let output = get_git_status();

    if output.status.success() {
        let files = marshal_status_in_files(
            String::from_utf8(output.stdout).expect("Problem parsing status"),
        );

        display(fmt_files_to_strings(files));

        println!("");

        loop {
            print!("\x1b[1F");
            print!("\x1b[1G");

            set_terminal_to_raw();
            read_key(&original_termios).unwrap();
            reset_terminal(&original_termios);
        }
    } else {
        print!("{}", String::from_utf8_lossy(&output.stderr))
    }
}

fn get_git_status() -> process::Output {
    process::Command::new("git")
        .arg("status")
        .arg("--porcelain")
        .output()
        .expect("failed to execute process")
}

fn marshal_status_in_files(status: String) -> Vec<File> {
    status
        .lines()
        .map(|line| File {
            status: line[0..2].to_string(),
            path: line[3..].to_string(),
            isSelected: false,
        }).collect()
}

fn set_terminal_to_raw() {
    let mut termios = Termios::from_fd(STDIN_FILENO).unwrap();
    cfmakeraw(&mut termios);
    tcsetattr(STDIN_FILENO, TCSANOW, &mut termios).unwrap();
}

fn reset_terminal(original_termios: &Termios) {
    tcsetattr(STDIN_FILENO, TCSANOW, original_termios).unwrap();
}

fn fmt_files_to_strings(files: Vec<File>) -> Vec<String> {
    files
        .iter()
        .map(|file| {
            format!(
                "[{}] {} {}",
                if file.isSelected { "*" } else { " " },
                file.status,
                file.path
            )
        }).collect()
}

fn display(lines: Vec<String>) {
    for line in lines {
        println!("{}", line);
    }
}

fn read_key(original_termios: &Termios) -> io::Result<()> {
    let stdout = io::stdout();
    let mut reader = io::stdin();
    let mut buffer = [0; 3];

    stdout.lock().flush()?;
    reader.read(&mut buffer)?;

    if buffer[1] != 0 {
        return Ok(());
    }

    if let Some(key) = Keys::from_u8(buffer[0]) {
        match key {
            Keys::J => println!("J"),
            Keys::K => println!("K"),
            Keys::Q | Keys::Escape => {
                reset_terminal(original_termios);
                process::exit(0);
            }
            Keys::CtrlC => {
                reset_terminal(original_termios);
                process::exit(130);
            }
            Keys::Enter => println!("Enter"),
            Keys::Space => println!("Space"),
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn turns_status_into_files() {
        let status = String::from(" M src/main.rs\n?? wow");
        let files = marshal_status_in_files(status);

        assert_eq!(
            vec![
                File {
                    status: String::from(" M"),
                    path: String::from("src/main.rs"),
                    isSelected: false,
                },
                File {
                    status: String::from("??"),
                    path: String::from("wow"),
                    isSelected: false,
                },
            ],
            files
        )
    }

    #[test]
    fn files_to_strings() {
        let files = vec![
            File {
                status: String::from("??"),
                path: String::from("/hello"),
                isSelected: true,
            },
            File {
                status: String::from(" M"),
                path: String::from("/is-it-me-you're-looking-for"),
                isSelected: false,
            },
        ];

        assert_eq!(
            fmt_files_to_strings(files),
            vec![
                String::from("[*] ?? /hello"),
                String::from("[ ]  M /is-it-me-you're-looking-for")
            ]
        )
    }
}
