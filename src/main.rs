// #![allow(clippy::cognitive_complexity)]
// #![allow(dead_code)]
// #![allow(unused_imports)]
// #![allow(unused_variables)]

use std::{
    io,
    vec::Vec,
};

use crossterm::{
    cursor,
    event::{
        self, 
        EnableMouseCapture, 
        DisableMouseCapture, 
        Event, 
        read, 
        KeyCode,
        KeyModifiers,
        MouseEventKind,
    },
    execute, queue, style,
    terminal::{self, ClearType},
};

const MENU: &str = r#"┌─────────────────────┐
│                     │
├─────┬─────┬─────┬───┤
│  C  │ +/- │  ⌫  │ ÷ │
├─────┼─────┼─────┼───┤
│  7  │  8  │  9  │ × │
├─────┼─────┼─────┼───┤
│  4  │  5  │  6  │ + │
├─────┼─────┼─────┼───┤
│  1  │  2  │  3  │ - │
├─────┴─────┼─────┼───┤
│     0     │  .  │ = │
└───────────┴─────┴───┘
^C to exit or click HERE"#;

#[derive(Debug, PartialEq, Clone)]
enum OperationType {
    None,
    Add,
    Subtract,
    Multiply,
    Divide,
}

#[derive(Debug, Clone)]
struct Operation {
    operation_type: OperationType,
    number: String,
}

#[derive(Debug)]
enum CalcMode {
    EnteringNumber,
    EnteringOperation,
    ShowingAns,
}

#[derive(Debug)]
struct SystemVars {
    cursor_position: (u16, u16), // u16 used by crossterm
    ans: String, // the anwser
    pending_operations: Vec<Operation>, // The list of pending operations
    current_operation: Operation, // The operation that is currently being entered
    calc_mode: CalcMode, // The current mode of the calculator, used for handling events
}

impl SystemVars {
    fn try_push_opp(&mut self) -> bool {
        if is_this_num_empty(&self.current_operation.number) {
            return false;
        }
        self.pending_operations.push(self.current_operation.clone());
        self.current_operation = EMPTY_OPERATION;
        true 
    }
}

const EMPTY_OPERATION: Operation = Operation{operation_type: OperationType::None, number: String::new()};
const DISPLAY_WIDTH: usize = 19;

// fn run<W>(w: &mut W) -> io::Result<()> where W: io::Write {
fn run<W>(w: &mut W) -> Result<(), String> where W: io::Write {
    execute!(w, terminal::EnterAlternateScreen).map_err(|e| e.to_string())?;

    terminal::enable_raw_mode().map_err(|e| e.to_string())?;
    queue!(
        w,
        style::ResetColor,
        terminal::Clear(ClearType::All),
        cursor::MoveTo(0, 0),
        cursor::SetCursorStyle::BlinkingBar,
        EnableMouseCapture
    ).map_err(|e| e.to_string())?;

    for line in MENU.split('\n') {
        queue!(w, style::Print(line), cursor::MoveToNextLine(1)).map_err(|e| e.to_string())?;
    }

    w.flush().map_err(|e| e.to_string())?;

    clear_display(w).map_err(|e| e.to_string())?;
    let mut system_vars = SystemVars{
        cursor_position: (0u16, 0u16),
        ans: String::new(), 
        pending_operations: Vec::new(),
        current_operation: EMPTY_OPERATION,
        calc_mode: CalcMode::EnteringNumber,
    };

    loop {
        let event = read().map_err(|e| e.to_string())?;

        match event {
            Event::Key(key_event) => {
                match key_event.modifiers {
                    KeyModifiers::NONE => { // none + _
                        match key_event.code {
                            KeyCode::Char(c) if c.is_numeric() => { // none + 0-9
                                handle_press(w, &mut system_vars, key_event.code)?;
                            }
                            KeyCode::Esc => { handle_press(w, &mut system_vars, key_event.code)?; } // none + Esc
                            KeyCode::Tab => { handle_press(w, &mut system_vars, key_event.code)?; } // none + Tab
                            KeyCode::Backspace => { handle_press(w, &mut system_vars, key_event.code)?; } // none + Backspace
                            KeyCode::Enter => { handle_press(w, &mut system_vars, key_event.code)?; } // none + Enter
                            KeyCode::Char(c) if c == '=' => { handle_press(w, &mut system_vars, KeyCode::Enter)?; } // none + =
                            KeyCode::Char(c) if matches!(c, '*' | '/' | '+' | '-' | '.') => {
                                handle_press(w, &mut system_vars, key_event.code)?; 
                            }
                            _ => {}
                        }
                    }
                    KeyModifiers::CONTROL => { // ctrl + _
                        match key_event.code {
                            KeyCode::Char('c') => { // ctrl + c
                                break;
                            }
                            _ => {}
                        }
                    }
                    _ => {}
                }
            }
            Event::Mouse(mouse_event) => {
                if mouse_event.kind == MouseEventKind::Up(event::MouseButton::Left) {
                    system_vars.cursor_position.0 = mouse_event.row;
                    system_vars.cursor_position.1 = mouse_event.column;
                    match system_vars.cursor_position.0 {
                        3 => {
                            match system_vars.cursor_position.1 {
                                1..=5 => { handle_press(w, &mut system_vars, KeyCode::Esc)?; } // Clear
                                7..=11 => { handle_press(w, &mut system_vars, KeyCode::Tab)?; } // +/-
                                13..=17 => { handle_press(w, &mut system_vars, KeyCode::Backspace)?; } // Backspace
                                19..=21 => { handle_press(w, &mut system_vars, KeyCode::Char('/'))?; } // /
                                _ => {}
                            }
                        }
                        5 => {
                            match system_vars.cursor_position.1 {
                                1..=5 => { handle_press(w, &mut system_vars, KeyCode::Char('7'))?; }
                                7..=11 => { handle_press(w, &mut system_vars, KeyCode::Char('8'))?; }
                                13..=17 => { handle_press(w, &mut system_vars, KeyCode::Char('9'))?; }
                                19..=21 => { handle_press(w, &mut system_vars, KeyCode::Char('*'))?; } // *
                                _ => {}
                            }
                        }
                        7 => {
                            match system_vars.cursor_position.1 {
                                1..=5 => { handle_press(w, &mut system_vars, KeyCode::Char('4'))?; }
                                7..=11 => { handle_press(w, &mut system_vars, KeyCode::Char('5'))?; }
                                13..=17 => { handle_press(w, &mut system_vars, KeyCode::Char('6'))?; }
                                19..=21 => { handle_press(w, &mut system_vars, KeyCode::Char('+'))?; } // +
                                _ => {}
                            }
                        }
                        9 => {
                            match system_vars.cursor_position.1 {
                                1..=5 => { handle_press(w, &mut system_vars, KeyCode::Char('1'))?; }
                                7..=11 => { handle_press(w, &mut system_vars, KeyCode::Char('2'))?; }
                                13..=17 => { handle_press(w, &mut system_vars, KeyCode::Char('3'))?; }
                                19..=21 => { handle_press(w, &mut system_vars, KeyCode::Char('-'))?; } // -
                                _ => {}
                            }
                        }
                        11 => {
                            match system_vars.cursor_position.1 {
                                1..=11 => { handle_press(w, &mut system_vars, KeyCode::Char('0'))?; }
                                13..=17 => { handle_press(w, &mut system_vars, KeyCode::Char('.'))?; }
                                19..=21 => { handle_press(w, &mut system_vars, KeyCode::Enter)?; } // Enter
                                _ => {}
                            }
                        }
                        13 => {
                            match system_vars.cursor_position.1 {
                                20..=23 => { break; }
                                _ => {}
                            }
                        }
                        _ => {}
                    }
                    // println!("{:?}\r", system_vars.cursor_position);
                }
            }
            _ => {}
        }
    }

    execute!(
        w,
        style::ResetColor,
        cursor::Show,
        cursor::SetCursorStyle::DefaultUserShape,
        terminal::LeaveAlternateScreen,
        DisableMouseCapture
    ).map_err(|e| e.to_string())?;
    terminal::disable_raw_mode().map_err(|e| e.to_string())?;
    Ok(())
}

fn handle_press<W>(w: &mut W, system_vars: &mut SystemVars, key: KeyCode) -> Result<(), String> where W: io::Write {
    match key {
        KeyCode::Char(char) if char.is_numeric() => {
            match system_vars.calc_mode {
                CalcMode::EnteringNumber => { 
                    system_vars.ans.push(char);
                    display_number(w, &system_vars.ans)?;
                }
                CalcMode::EnteringOperation => {
                    system_vars.current_operation.number.push(char);
                    display_operation(w, &system_vars.current_operation)?;
                }
                _ => {}
            }
        }
        KeyCode::Char(char) if matches!(char, '*' | '/' | '+' | '-') => {
            match system_vars.calc_mode {
                CalcMode::EnteringOperation => {
                    if system_vars.current_operation.number == "" || system_vars.current_operation.number == "-" { return Ok(()); }
                    system_vars.pending_operations.push(system_vars.current_operation.clone());
                    system_vars.current_operation = EMPTY_OPERATION;
                }
                CalcMode::EnteringNumber |
                CalcMode::ShowingAns => {
                    if system_vars.ans == "" || system_vars.ans == "-" { return   Ok(()); }
                    system_vars.calc_mode = CalcMode::EnteringOperation;
                }
            }
            
            match char {
                '*' => { system_vars.current_operation.operation_type = OperationType::Multiply; }
                '/' => { system_vars.current_operation.operation_type = OperationType::Divide; }
                '+' => { system_vars.current_operation.operation_type = OperationType::Add; }
                '-' => { system_vars.current_operation.operation_type = OperationType::Subtract; }
                _ => {}
            }
            display_operation(w, &system_vars.current_operation)?;
        }
        // KeyCode::Char(char) if char == '%' => { // FUCK PERCENT NOBODY USES IT
        //     match system_vars.calc_mode {
        //         CalcMode::EnteringNumber => {
        //             if is_this_num_empty(&system_vars.ans) { return; }
        //             let mut this_num: f64 = system_vars.ans.parse().unwrap();
        //             this_num *= 0.01f64;
        //             system_vars.ans = this_num.to_string();
        //             display_number(w, &system_vars.ans);
        //         }
        //         CalcMode::EnteringOperation => {
        //             if is_this_num_empty(&system_vars.current_operation.number) { return; }
        //             let mut this_num: f64 = system_vars.current_operation.number.parse().unwrap();
        //             this_num *= 0.01f64;
        //             system_vars.current_operation.number = this_num.to_string();
        //             display_operation(w, &system_vars.current_operation);
        //         }
        //         _ => {}
        //     }
        // }
        KeyCode::Char(char) if char == '.' => {
            match system_vars.calc_mode {
                CalcMode::EnteringNumber => {
                    if !system_vars.ans.contains('.') && !is_this_num_empty(&system_vars.ans) {
                        system_vars.ans.push('.');
                        display_number(w, &system_vars.ans)?;
                    }
                }
                CalcMode::EnteringOperation => {
                    if !system_vars.current_operation.number.contains('.') &&
                    !is_this_num_empty(&system_vars.current_operation.number) {
                        system_vars.current_operation.number.push('.');
                        display_operation(w, &system_vars.current_operation)?;
                    }
                }
                _ => {}
            }
        }
        KeyCode::Esc => {
            system_vars.pending_operations = Vec::new();
            system_vars.ans = String::new();
            system_vars.current_operation = EMPTY_OPERATION;
            system_vars.calc_mode = CalcMode::EnteringNumber;
            clear_display(w)?;
        }
        KeyCode::Tab => {
            match system_vars.calc_mode {
                CalcMode::EnteringNumber => { 
                    if system_vars.ans.chars().next() == Some('-') {
                        system_vars.ans = system_vars.ans[1..].to_string();
                    } else {
                        system_vars.ans = "-".to_string() + &system_vars.ans;
                    }
                    display_number(w, &system_vars.ans)?;
                }
                CalcMode::EnteringOperation => {
                    if system_vars.current_operation.number.chars().next() == Some('-') {
                        system_vars.current_operation.number = system_vars.current_operation.number[1..].to_string();
                    } else {
                        system_vars.current_operation.number = "-".to_string() + &system_vars.current_operation.number;
                    }
                    display_operation(w, &system_vars.current_operation)?;
                }
                _ => {}
            }
        }
        KeyCode::Backspace => {
            match system_vars.calc_mode {
                CalcMode::EnteringNumber => {
                    if !system_vars.ans.is_empty() {
                        system_vars.ans.pop();
                        display_number(w, &system_vars.ans)?;
                    }
                }
                CalcMode::EnteringOperation => {
                    if !system_vars.current_operation.number.is_empty() {
                        system_vars.current_operation.number.pop();
                        display_operation(w, &system_vars.current_operation)?;
                    }
                }
                _ => {}
            }
        }
        KeyCode::Enter => {

            if system_vars.try_push_opp() {
                execute_operation(system_vars)?;
                system_vars.pending_operations = Vec::new();
                system_vars.current_operation = EMPTY_OPERATION;
                system_vars.calc_mode = CalcMode::ShowingAns;
                display_number(w, &system_vars.ans)?;
            }
        }
        _ => {}
    }
    Ok(())
}

fn execute_operation(system_vars: &mut SystemVars) -> Result<(), String> {
    let mut ans_num;
    match str_to_f64(system_vars.ans.to_string()) {
        Ok(ok) => { ans_num = ok; }
        Err(err) => { return Err(err); }
    }

    for element in &system_vars.pending_operations {
        let Operation{operation_type: op, number: num_str} = element;

        let num: f64;
        match num_str.parse::<f64>() {
            Ok(parsed_num) => { num = parsed_num; }
            Err(_) => {
                return Err("Failed to convert a String to a float.".to_string());
            }
        }
        
        match op {
            OperationType::None => {}
            OperationType::Add => { ans_num += num; }
            OperationType::Subtract => { ans_num -= num; }
            OperationType::Multiply => { ans_num *= num; }
            OperationType::Divide => { ans_num /= num; }
        }
    }
    system_vars.calc_mode = CalcMode::ShowingAns;
    system_vars.ans = ans_num.to_string();
    Ok(())
}

fn display_number<W>(w: &mut W, ans: &String) -> Result<(), String> where W: io::Write {
    clear_display(w)?;
    let formated_num;
    match format_number(ans.to_string(), DISPLAY_WIDTH) {
        Ok(ok) => { formated_num = ok; }
        Err(err) => { return Err(err);}
    }

    match execute!(w, style::Print(formated_num)) {
        Ok(()) => { Ok(()) }
        Err(err) => { Err(format!("Failed to execute: {}", err)) }
    }
}

fn display_operation<W>(w: &mut W, operation: &Operation) -> Result<(), String> where W: io::Write {
    clear_display(w)?;
    let Operation{operation_type: op, number: raw_num} = operation;
    
    match op {
        OperationType::None => { queue!(w, style::Print("  ")).map_err(|e| e.to_string())?; }
        OperationType::Add => { queue!(w, style::Print("+ ")).map_err(|e| e.to_string())?; }
        OperationType::Subtract => { queue!(w, style::Print("- ")).map_err(|e| e.to_string())?; }
        OperationType::Multiply => { queue!(w, style::Print("× ")).map_err(|e| e.to_string())?; }
        OperationType::Divide => { queue!(w, style::Print("÷ ")).map_err(|e| e.to_string())?; }
    }

    let formated_num;
    match format_number(raw_num.to_string(), DISPLAY_WIDTH - 2) {
        Ok(ok) => { formated_num = ok; }
        Err(err) => { return Err(err);}
    }
    queue!(w, style::Print(formated_num)).map_err(|e| e.to_string())?;
    w.flush().map_err(|e| e.to_string())?;
    Ok(())
}

fn clear_display<W>(w: &mut W) -> Result<(), String> where W: io::Write {
    queue!(w, cursor::MoveTo(2, 1), style::Print("                   ")).map_err(|e| e.to_string())?;
    queue!(w, cursor::MoveTo(2, 1)).map_err(|e| e.to_string())?;
    w.flush().map_err(|e| e.to_string())?;
    Ok(())
}

fn format_number(cool_str: String, max_width: usize) -> Result<String, String> {
    if cool_str.len() == 0 { Ok(cool_str) }
    else if cool_str.len() > max_width {
        Ok(format!("{:.*e}", max_width-7, cool_str.parse::<f64>().map_err(|e| e.to_string())?) )
    } 
    else { Ok(cool_str) }
}

fn str_to_f64(the_string: String) -> Result<f64, String> {
    if the_string.len() == 0 { return Ok(0f64); }
    Ok(the_string.parse::<f64>().map_err(|e| e.to_string())?)
}

fn is_this_num_empty(this_string: &String) -> bool {
    let this_len = this_string.len();
    if this_len == 0 || (this_string.chars().next() == Some('-') && this_len == 1) {
        true
    } else {
        false
    }
}

fn main() {
    let mut stdout = io::stdout();
    match run(&mut stdout) {
        Ok(()) => {}
        Err(error) => { println!("Encounterrd an unexpected error.\n{}", error); }
    }
}