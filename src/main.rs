use enigo::{Enigo, MouseControllable, KeyboardControllable, MouseButton};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "step")]
enum Step {
    #[serde(rename = "click")]
    Click {
        x: i32,
        y: i32,
        #[serde(default = "default_delay")]
        delay: f64,
    },
    #[serde(rename = "right_click")]
    RightClick {
        x: i32,
        y: i32,
        #[serde(default = "default_delay")]
        delay: f64,
    },
    #[serde(rename = "text_input")]
    TextInput {
        text: String,
        #[serde(default = "default_delay")]
        delay: f64,
    },
    #[serde(rename = "press_key")]
    PressKey {
        key: String,
        #[serde(default = "default_delay")]
        delay: f64,
    },
    #[serde(rename = "hotkey")]
    Hotkey {
        keys: Vec<String>,
        #[serde(default = "default_delay")]
        delay: f64,
    },
    #[serde(rename = "loop")]
    Loop {
        repetitions: u32,
        actions: Vec<Step>,
    },
}

fn default_delay() -> f64 {
    0.2
}

fn main() {
        let args: Vec<String> = env::args().collect();

    // Check for --run / -r flag among all arguments (after the exe name).
    let auto_run = args.iter().skip(1).any(|a| a == "--run" || a == "-r");

    // If a workflow path was given (first non-flag arg), skip the menu and
    // go straight to run_automation(). With --run / -r also skip all prompts.
    let has_path_arg = args.iter().skip(1).any(|a| !a.starts_with('-'));

    if has_path_arg {
        run_automation(auto_run);
        return;
    }

    println!("Scripted Autoclicker Tool");
    println!("{}\n", "=".repeat(50));

    loop {
        println!("1. Run workflow");
        println!("2. Record new workflow");
        println!("3. Show live mouse position");
        println!("4. Exit\n");

        print!("Choose (1-4): ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => run_automation(false),
            "2" => record_workflow(),
            "3" => show_mouse_position(),
            "4" | "q" | "quit" | "exit" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("Invalid option.\n"),
        }
        println!("\n\nBack to menu...\n{}\n", "=".repeat(50));
    }
}

// ====================== DURATION ESTIMATION ======================
fn estimate_steps_secs(steps: &[Step]) -> f64 {
    steps.iter().map(|step| match step {
        Step::Click      { delay, .. } => *delay,
        Step::RightClick { delay, .. } => *delay,
        Step::TextInput  { delay, .. } => *delay,
        Step::PressKey   { delay, .. } => *delay,
        Step::Hotkey     { delay, .. } => *delay,
        Step::Loop       { repetitions, actions } => {
            *repetitions as f64 * estimate_steps_secs(actions)
        }
    }).sum()
}

fn format_duration(secs: f64) -> String {
    let total = secs.round() as u64;
    let h = total / 3600;
    let m = (total % 3600) / 60;
    let s = total % 60;
    match (h, m) {
        (0, 0) => format!("{}s", s),
        (0, _) => format!("{}m {}s", m, s),
        _      => format!("{}h {}m {}s", h, m, s),
    }
}

// ====================== KEY PARSING ======================
// Shared by PressKey and Hotkey so both use the same name table.
fn parse_key(name: &str) -> Option<enigo::Key> {
    use enigo::Key;
    match name.to_lowercase().as_str() {
        // --- Navigation ---
        "tab"                       => Some(Key::Tab),
        "escape" | "esc"            => Some(Key::Escape),
        "space"                     => Some(Key::Space),
        "backspace"                 => Some(Key::Backspace),
        "delete" | "del"            => Some(Key::Delete),
        "insert" | "ins"            => Some(Key::Insert),
        "up"                        => Some(Key::UpArrow),
        "down"                      => Some(Key::DownArrow),
        "left"                      => Some(Key::LeftArrow),
        "right"                     => Some(Key::RightArrow),
        "home"                      => Some(Key::Home),
        "end"                       => Some(Key::End),
        "pageup"   | "page_up"      => Some(Key::PageUp),
        "pagedown" | "page_down"    => Some(Key::PageDown),
        // --- Modifiers ---
        "ctrl" | "control"          => Some(Key::Control),
        "lctrl" | "lcontrol"        => Some(Key::LControl),
        "rctrl" | "rcontrol"        => Some(Key::RControl),
        "alt"                       => Some(Key::Alt),
        "shift"                     => Some(Key::Shift),
        "lshift"                    => Some(Key::LShift),
        "rshift"                    => Some(Key::RShift),
        "super" | "win" | "meta"    => Some(Key::Meta),
        "capslock" | "caps"         => Some(Key::CapsLock),
        "numlock"                   => Some(Key::Numlock),
        // --- System / misc ---
        "return" | "enter"          => Some(Key::Return),
        "pause"                     => Some(Key::Pause),
        "print" | "printscreen"     => Some(Key::Print),
        "help"                      => Some(Key::Help),
        "select"                    => Some(Key::Select),
        "execute"                   => Some(Key::Execute),
        "clear"                     => Some(Key::Clear),
        "cancel"                    => Some(Key::Cancel),
        // --- Media ---
        "volup"   | "volumeup"      => Some(Key::VolumeUp),
        "voldown" | "volumedown"    => Some(Key::VolumeDown),
        "mute" | "volumemute"       => Some(Key::VolumeMute),
        "medianext" | "nexttrack"   => Some(Key::MediaNextTrack),
        "mediaprev" | "prevtrack"   => Some(Key::MediaPrevTrack),
        "mediastop"                 => Some(Key::MediaStop),
        "mediaplay" | "playpause"   => Some(Key::MediaPlayPause),
        // --- Numpad ---
        "num0" | "numpad0"          => Some(Key::Numpad0),
        "num1" | "numpad1"          => Some(Key::Numpad1),
        "num2" | "numpad2"          => Some(Key::Numpad2),
        "num3" | "numpad3"          => Some(Key::Numpad3),
        "num4" | "numpad4"          => Some(Key::Numpad4),
        "num5" | "numpad5"          => Some(Key::Numpad5),
        "num6" | "numpad6"          => Some(Key::Numpad6),
        "num7" | "numpad7"          => Some(Key::Numpad7),
        "num8" | "numpad8"          => Some(Key::Numpad8),
        "num9" | "numpad9"          => Some(Key::Numpad9),
        "numadd"  | "numplus"       => Some(Key::Add),
        "numsub"  | "numminus"      => Some(Key::Subtract),
        "nummul"  | "nummultiply"   => Some(Key::Multiply),
        "numdiv"  | "numdivide"     => Some(Key::Divide),
        "numdec"  | "numdecimal"    => Some(Key::Decimal),
        // --- F-keys (extended to F20) ---
        "f1"  => Some(Key::F1),  "f2"  => Some(Key::F2),
        "f3"  => Some(Key::F3),  "f4"  => Some(Key::F4),
        "f5"  => Some(Key::F5),  "f6"  => Some(Key::F6),
        "f7"  => Some(Key::F7),  "f8"  => Some(Key::F8),
        "f9"  => Some(Key::F9),  "f10" => Some(Key::F10),
        "f11" => Some(Key::F11), "f12" => Some(Key::F12),
        "f13" => Some(Key::F13), "f14" => Some(Key::F14),
        "f15" => Some(Key::F15), "f16" => Some(Key::F16),
        "f17" => Some(Key::F17), "f18" => Some(Key::F18),
        "f19" => Some(Key::F19), "f20" => Some(Key::F20),
        // --- Single character fallback (layout-dependent) ---
        // A single printable char like "a", "1", "+" maps to Key::Layout.
        s if s.chars().count() == 1 => Some(Key::Layout(s.chars().next().unwrap())),
        _ => None,
    }
}

// ====================== EXECUTION ======================
fn run_automation(silent: bool) {
    let (steps, top_repetitions) = load_workflow();

    if steps.is_empty() {
        println!("No actions to run. \nWorkflow file is empty or contains syntax errors.");
        return;        
    }

    let per_rep_secs = estimate_steps_secs(&steps);
    let total_secs   = per_rep_secs * top_repetitions as f64;

    println!(
        "Loaded workflow -- {} top-level actions x {} repetitions",
        steps.len(),
        top_repetitions
    );
    println!(
        "Estimated time:   {} total; {} per repetition",
        format_duration(total_secs),
        format_duration(per_rep_secs)
    );

    let mut enigo = Enigo::new();

    if !silent {
        println!("\nTo emergency STOP the workflow, place mouse to UPPER-LEFT corner");
        println!("Press ENTER to START the workflow...");
        let _ = std::io::stdin().read_line(&mut String::new());
    }

    for i in 1..=top_repetitions {
        println!("    Top-level iteration {}/{}", i, top_repetitions);
        let mut rep_stack = vec![i];
        if execute_steps(&mut enigo, &steps, &mut rep_stack) {
            println!("Exit.");
            return; // stopped by user, exit immediately
        }
    }

    println!("\nWorkflow completed!");
}

fn execute_steps(enigo: &mut Enigo, steps: &[Step], rep_stack: &mut Vec<u32>) -> bool {
    for step in steps {
        if is_stopped(enigo) {
            println!("\nEXECUTION STOPPED BY USER");
            return true; // stopped
        }

        match step {
            Step::Click { x, y, delay } => {
                enigo.mouse_move_to(*x, *y);
                enigo.mouse_click(MouseButton::Left);
                println!("    Clicked at ({}, {})", x, y);
                thread::sleep(Duration::from_secs_f64(*delay));
            }
            Step::RightClick { x, y, delay } => {
                enigo.mouse_move_to(*x, *y);
                enigo.mouse_click(MouseButton::Right);
                println!("    Right-clicked at ({}, {})", x, y);
                thread::sleep(Duration::from_secs_f64(*delay));
            }
            Step::TextInput { text, delay } => {
                let final_text = if let Some(&last_rep) = rep_stack.last() {
                    text.replace("{$}", &last_rep.to_string())
                } else {
                    text.clone()
                };
                enigo.key_sequence(&final_text);
                println!("    Typed: {}", final_text);
                thread::sleep(Duration::from_secs_f64(*delay));
            }
            Step::PressKey { key, delay } => {
                match parse_key(key) {
                    Some(key_code) => {
                        enigo.key_click(key_code);
                        println!("    Pressed key: {}", key);
                    }
                    None => println!("    Unknown key '{}', skipping", key),
                }
                thread::sleep(Duration::from_secs_f64(*delay));
            }
            Step::Hotkey { keys, delay } => {
                // Resolve all names first; skip the whole combo if any is unknown.
                let resolved: Vec<enigo::Key> = keys.iter()
                    .filter_map(|k| {
                        let r = parse_key(k);
                        if r.is_none() {
                            println!("    Unknown key '{}' in hotkey, skipping combo", k);
                        }
                        r
                    })
                    .collect();

                if resolved.len() == keys.len() {
                    // Hold all keys down in order, release in reverse.
                    for &k in &resolved          { enigo.key_down(k); }
                    for &k in resolved.iter().rev() { enigo.key_up(k); }
                    println!("    Hotkey: {}", keys.join(" + "));
                }
                thread::sleep(Duration::from_secs_f64(*delay));
            }
            Step::Loop { repetitions, actions } => {
                for i in 1..=*repetitions {
                    println!("        Loop iteration {}/{}", i, repetitions);
                    rep_stack.push(i);
                    let stopped = execute_steps(enigo, actions, rep_stack);
                    rep_stack.pop();
                    if stopped {
                        return true; // propagate stop upward
                    }
                }
            }
        }
    }
    false // completed normally
}

// ====================== RECORDER ======================
fn record_workflow() {
    println!("\nLet's record your workflow!");
    println!("Commands:");
    println!("   ENTER -> Record Click");
    println!("   r     -> Record Right Click");
    println!("   t     -> Record Text Input (use {{$}} to yield current (innermost) loop iteration number)");
    println!("   k     -> Record Press Key  (enter, tab, esc, space, backspace, delete, f1 to f12,");
    println!("                              up, down, left, right, home, end, pageup, pagedown)");
    println!("   h     -> Record Hotkey     (e.g. ctrl+c  or  ctrl+alt+delete)");
    println!("   [     -> Start new nested loop");
    println!("   ]     -> End current (innermost) loop");
    println!("   q     -> Finish recording\n");

    let steps = Vec::new();
    let mut loop_stack: Vec<Vec<Step>> = vec![steps];

    loop {
        print!("\nCommand (ENTER/r/t/k/h/[/]/q): ");
        io::stdout().flush().unwrap();

        let mut cmd = String::new();
        io::stdin().read_line(&mut cmd).unwrap();
        let cmd = cmd.trim().to_lowercase();

        match cmd.as_str() {
            "q" => break,
            "[" => start_new_loop(&mut loop_stack),
            "]" => end_current_loop(&mut loop_stack),
            "t" => record_text_input_action(&mut loop_stack),
            "r" => record_right_click_action(&mut loop_stack),
            "k" => record_press_key_action(&mut loop_stack),
            "h" => record_hotkey_action(&mut loop_stack),
            "" => record_click_action(&mut loop_stack),
            _ => println!("Unknown command"),
        }
    }

    let final_steps = loop_stack.remove(0);

    print!("\nTop-level repetitions (default 1): ");
    io::stdout().flush().unwrap();
    let mut rep_str = String::new();
    io::stdin().read_line(&mut rep_str).unwrap();
    let top_reps: u32 = rep_str.trim().parse().unwrap_or(1);

    let workflow = serde_yaml::Mapping::from_iter([
        (serde_yaml::Value::String("repetitions".into()), serde_yaml::to_value(top_reps).unwrap()),
        (serde_yaml::Value::String("actions".into()),     serde_yaml::to_value(final_steps).unwrap()),
    ]);

    let yaml = serde_yaml::to_string(&workflow).unwrap();

    println!("\nRECORDING FINISHED!");
    println!("Save this to a YAML file for later use:");
    println!("{}", "=".repeat(50));
    println!("\n{}", yaml);
    println!("{}", "=".repeat(50));

    print!("\nDo you want to save into the workflow.yaml? y/N:");
    io::stdout().flush().unwrap();
    let mut save_answer = String::new();
    io::stdin().read_line(&mut save_answer).unwrap();
    let save_answer = save_answer.trim().to_lowercase();

    if save_answer == "y" || save_answer == "Y" {
        let mut save_path = env::current_exe().unwrap();
        save_path.pop();
        save_path.push("workflow.yaml");

        match fs::write(&save_path, &yaml) {
            Ok(_) => println!("Workflow saved to: {}", save_path.display()),
            Err(e) => println!("Failed to save workflow: {}", e),
        }
    } else {
        println!("Workflow not saved.");
    }

}

fn start_new_loop(stack: &mut Vec<Vec<Step>>) {
    stack.push(Vec::new());
    println!("   Started new inner loop");
}

fn end_current_loop(stack: &mut Vec<Vec<Step>>) {
    if stack.len() <= 1 {
        println!("   No open loop to close");
        return;
    }

    print!("   How many repetitions for this loop? (default 1): ");
    io::stdout().flush().unwrap();
    let mut rep_str = String::new();
    io::stdin().read_line(&mut rep_str).unwrap();
    let repetitions: u32 = rep_str.trim().parse().unwrap_or(1);

    let inner = stack.pop().unwrap();
    let loop_step = Step::Loop {
        repetitions,
        actions: inner,
    };
    stack.last_mut().unwrap().push(loop_step);
    println!("   Closed loop with {} repetitions", repetitions);
}

fn record_click_action(stack: &mut Vec<Vec<Step>>) {
    // enigo 0.1.3: mouse_location() returns (i32, i32) directly
    let enigo = Enigo::new();
    let (x, y) = enigo.mouse_location();

    print!("   Click at ({}, {}) -> subsequent delay (default {}s): ", x, y, default_delay());
    io::stdout().flush().unwrap();
    let mut d = String::new();
    io::stdin().read_line(&mut d).unwrap();
    let delay: f64 = d.trim().parse().unwrap_or(default_delay());

    stack.last_mut().unwrap().push(Step::Click { x, y, delay });
    println!("   Click recorded");
}

fn record_right_click_action(stack: &mut Vec<Vec<Step>>) {
    let enigo = Enigo::new();
    let (x, y) = enigo.mouse_location();

    print!("   Right click at ({}, {}) -> subsequent delay (default {}s): ", x, y, default_delay());
    io::stdout().flush().unwrap();
    let mut d = String::new();
    io::stdin().read_line(&mut d).unwrap();
    let delay: f64 = d.trim().parse().unwrap_or(default_delay());

    stack.last_mut().unwrap().push(Step::RightClick { x, y, delay });
    println!("   Right click recorded");
}

fn record_text_input_action(stack: &mut Vec<Vec<Step>>) {
    print!("   Text (use {{$}} for current loop number): ");
    io::stdout().flush().unwrap();
    let mut text = String::new();
    io::stdin().read_line(&mut text).unwrap();
    let text = text.trim().to_string();

    print!("   Subsequent delay (default {}s): ", default_delay());
    io::stdout().flush().unwrap();
    let mut d = String::new();
    io::stdin().read_line(&mut d).unwrap();
    let delay: f64 = d.trim().parse().unwrap_or(default_delay());

    stack.last_mut().unwrap().push(Step::TextInput { text, delay });
    println!("   Text input recorded");
}

fn print_key_map() {
    println!("   ┌──────────────────────────────────────────────────────────────┐");
    println!("   │ KEY MAP                                                      │");
    println!("   ├───────────────────────────────┬──────────────────────────────┤");
    println!("   │ Navigation:                   │ Modifiers:                   │");
    println!("   │   enter / return              │   ctrl  / lctrl / rctrl      │");
    println!("   │   tab                         │   alt                        │");
    println!("   │   esc                         │   shift / lshift / rshift    │");
    println!("   │   space                       │   super / win / meta         │");
    println!("   │   backspace                   │   caps  / capslock           │");
    println!("   │   delete / del                │   numlock                    │");
    println!("   │   insert / ins                │                              │");
    println!("   │   up / down / left / right    ├──────────────────────────────┤");
    println!("   │   home / end                  │ System:                      │");
    println!("   │   pageup / pagedown           │   pause  print               │");
    println!("   ├───────────────────────────────┤   sysreq  break  help        │");
    println!("   │ F-keys:                       │   select                     │");
    println!("   │   f1 .. f20                   │   execute  clear  cancel     │");
    println!("   ├───────────────────────────────┤──────────────────────────────┤");
    println!("   │ Numpad:                       │ Media:                       │");
    println!("   │   num0 .. num9                │   volup  voldown  mute       │");
    println!("   │   numadd  numsub              │   medianext / nexttrack      │");
    println!("   │   nummul  numdiv  numdec      │   mediaprev / prevtrack      │");
    println!("   │                               │   mediastop                  │");
    println!("   │                               │   mediaplay / playpause      │");
    println!("   ├───────────────────────────────┴──────────────────────────────┤");
    println!("   │ Single char: any one character (a, 1, +, ...) is also valid  │");
    println!("   └──────────────────────────────────────────────────────────────┘");
}

fn record_press_key_action(stack: &mut Vec<Vec<Step>>) {
    print_key_map();
    print!("   Key name: ");
    io::stdout().flush().unwrap();
    let mut key = String::new();
    io::stdin().read_line(&mut key).unwrap();
    let key = key.trim().to_string();

    if key.is_empty() {
        println!("   No key entered, skipping.");
        return;
    }

    print!("   Subsequent delay (default {}s): ", default_delay());
    io::stdout().flush().unwrap();
    let mut d = String::new();
    io::stdin().read_line(&mut d).unwrap();
    let delay: f64 = d.trim().parse().unwrap_or(default_delay());

    stack.last_mut().unwrap().push(Step::PressKey { key: key.clone(), delay });
    println!("   Press key '{}' recorded", key);
}

fn record_hotkey_action(stack: &mut Vec<Vec<Step>>) {
    print_key_map();
    print!("   Hotkey combo (e.g. ctrl+c  or  ctrl+alt+delete): ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();

    // Split on '+', trim whitespace, lowercase each part.
    let keys: Vec<String> = input.trim()
        .split('+')
        .map(|s| s.trim().to_lowercase())
        .filter(|s| !s.is_empty())
        .collect();

    if keys.is_empty() {
        println!("   No keys entered, hotkey not recorded.");
        return;
    }

    // Validate every key name before recording.
    let unknown: Vec<&str> = keys.iter()
        .filter(|k| parse_key(k).is_none())
        .map(|k| k.as_str())
        .collect();

    if !unknown.is_empty() {
        println!("   Unknown key(s): {}. Hotkey not recorded.", unknown.join(", "));
        return;
    }

    print!("   Subsequent delay (default {}s): ", default_delay());
    io::stdout().flush().unwrap();
    let mut d = String::new();
    io::stdin().read_line(&mut d).unwrap();
    let delay: f64 = d.trim().parse().unwrap_or(default_delay());

    println!("   Hotkey '{}' recorded", keys.join(" + "));
    stack.last_mut().unwrap().push(Step::Hotkey { keys, delay });
}

// ====================== LOADING ======================
fn load_workflow() -> (Vec<Step>, u32) {
    // First argument that doesn't start with '-' is the workflow path.
    let path = if let Some(arg) = env::args().skip(1).find(|a| !a.starts_with('-')) {
        PathBuf::from(arg)
    } else {
        let mut p = env::current_exe().unwrap();
        p.pop();
        p.push("workflow.yaml");
        p
    };

    if !path.exists() {
        println!("workflow.yaml not found.");
        println!("Please record a new workflow first using option 2.\n");

        print!("Enter full path to workflow file (or press Enter to cancel): ");
        io::stdout().flush().unwrap();

        let mut input = String::new();
        io::stdin().read_line(&mut input).unwrap();
        let input = input.trim();

        if input.is_empty() {
            return (Vec::new(), 1);
        }
        let custom_path = PathBuf::from(input);
        return load_file(&custom_path);
    }

    load_file(&path)
}

fn load_file(path: &PathBuf) -> (Vec<Step>, u32) {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return (Vec::new(), 1),
    };

    let data: serde_yaml::Value = match serde_yaml::from_str(&content) {
        Ok(d) => d,
        Err(_) => return (Vec::new(), 1),
    };

    let repetitions = data
        .get("repetitions")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    let steps: Vec<Step> = if let Some(actions) = data.get("actions") {
        serde_yaml::from_value(actions.clone()).unwrap_or_default()
    } else {
        Vec::new()
    };

    println!(
        "Loaded: {}",
        path.file_name().unwrap().to_string_lossy()
    );
    (steps, repetitions)
}

fn show_mouse_position() {
    let enigo = Enigo::new();
    println!("\nLive position \nMove mouse to UPPER-LEFT corner (0, 0) to stop\n");
    loop {
        let (x, y) = enigo.mouse_location();
        print!("\rX: {:4} | Y: {:4}", x, y);
        io::stdout().flush().unwrap();
        thread::sleep(Duration::from_millis(200));
        if x == 0 && y == 0 {
            break
        } else {
            continue
        };
    }
}

fn is_stopped(enigo: &mut Enigo) -> bool {
    let (x, y) = enigo.mouse_location();
    x == 0 && y == 0
}