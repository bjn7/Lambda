// This file contains built-in abstractions.

use super::interpreter::EvaluationValue;

use anyhow::{Result, bail};
use crossterm::event::{self, KeyCode};
use std::time::{SystemTime, UNIX_EPOCH};
use std::{
    io::{self, Write},
    time::Duration,
};

// Input's modes are separated, due to unnessary complexicity it creates.
pub fn abstraction_input_char() -> Result<EvaluationValue> {
    crossterm::terminal::enable_raw_mode().unwrap();
    let result;
    loop {
        if event::poll(Duration::from_millis(10)).unwrap() {
            if let event::Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    KeyCode::Char(c) => {
                        result = c as u8;
                        break;
                    }
                    KeyCode::Enter => {
                        result = 10;
                        break;
                    }
                    _ => {}
                }
            }
        }
    }
    crossterm::terminal::disable_raw_mode().unwrap();
    Ok(EvaluationValue::Literal(result as f64))
}

pub fn abstraction_input_numeric() -> Result<EvaluationValue> {
    crossterm::terminal::enable_raw_mode().unwrap();
    let mut is_dot_allowed = true;
    let mut is_e_allowed = false;
    let mut is_sign_allowed = false;
    let mut result = String::new();
    let mut print = false;
    loop {
        if event::poll(Duration::from_millis(10)).unwrap() {
            if let event::Event::Key(key_event) = event::read().unwrap() {
                match key_event.code {
                    // is_e_allowed along isn't sufficient to determine if e can be added.
                    KeyCode::Char('e') if is_e_allowed && !result.contains('e') => {
                        result.push('e');
                        is_dot_allowed = false;
                        is_e_allowed = false;
                        is_sign_allowed = true;
                        print = true;
                    }
                    KeyCode::Char('+') if is_sign_allowed => {
                        result.push('+');
                        is_sign_allowed = false;
                        print = true;
                    }
                    KeyCode::Char('-') if is_sign_allowed => {
                        result.push('-');
                        is_sign_allowed = false;
                        print = true;
                    }
                    KeyCode::Char('.') if is_dot_allowed => {
                        result.push('.');
                        is_dot_allowed = false;
                        print = true;
                        // After, e if there is a digit, sign is no longer allowed;
                        // i.e 10e10;
                        is_sign_allowed = !is_e_allowed;
                        is_e_allowed = true
                    }

                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        result.push(c);
                        is_e_allowed = true;
                        print = true;
                    }
                    KeyCode::Enter => {
                        break;
                    }
                    _ => {}
                }
            }
        }
        if print {
            write!(io::stdout(), "{}", result.chars().last().unwrap_or('\r'))?;
            io::stdout().flush()?;
            print = false;
        }
    }
    crossterm::terminal::disable_raw_mode().unwrap();
    let result_num = result.parse::<f64>()?;
    Ok(EvaluationValue::Literal(result_num))
}

// Bob Dylan is quite good
// listening to Knocking on Heaven's Door rn.
pub fn abstraction_ascii(ascii: u8) -> Result<EvaluationValue> {
    io::stdout().write_all(&[ascii]).unwrap();
    io::stdout().flush()?;
    Ok(EvaluationValue::Literal(ascii as f64))
}

pub fn abstraction_print(numeric_value: f64) -> Result<EvaluationValue> {
    write!(io::stdout(), "{}", numeric_value)?;
    io::stdout().flush()?;
    Ok(EvaluationValue::Literal(numeric_value))
}

pub fn abstraction_time() -> Result<EvaluationValue> {
    let current_time = SystemTime::now();
    match current_time.duration_since(UNIX_EPOCH) {
        Ok(stamp) => Ok(EvaluationValue::Literal(stamp.as_millis() as f64)),
        Err(e) => bail!("SystemTimeError difference: {:?}", e.duration()),
    }
}

pub fn abstraction_sleep(time: f64) -> Result<EvaluationValue> {
    std::thread::sleep(Duration::from_millis(time as u64));
    Ok(EvaluationValue::Literal(time))
}
