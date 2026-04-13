use enigo::{Button, Coordinate, Direction, Enigo, Keyboard, Mouse, Settings};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type")]
enum Step {
    #[serde(rename = "click")]
    Click {
        x: i32,
        y: i32,
        #[serde(default = "default_delay")]
        delay: f64,
    },
    #[serde(rename = "type")]
    Type {
        text: String,
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
    0.1  // default delay in sec. to meet possible GUI freezes
}

fn main() {
    println!("🚀 Rust Autoclicker Suite");
    println!("==================================================\n");

    loop {
        println!("1. Run automation");
        println!("2. Record new workflow");
        println!("3. Show live mouse position");
        println!("4. Exit\n");

        print!("Choose (1-4): ");
        io::stdout().flush().unwrap();

        let mut choice = String::new();
        io::stdin().read_line(&mut choice).unwrap();
        let choice = choice.trim();

        match choice {
            "1" => run_automation(),
            "2" => record_workflow(),
            "3" => show_mouse_position(),
            "4" => {
                println!("👋 Goodbye!");
                break;
            }
            _ => println!("❌ Invalid option.\n"),
        }
    }
}

// ====================== EXECUTION ======================
fn run_automation() {
    let (steps, top_repetitions) = load_workflow();

    if steps.is_empty() {
        println!("⚠️  No actions to run. Please record a workflow first (option 2).");
        pause_to_menu();
        return;
    }

    println!("✅ Loaded workflow — {} top-level actions × {} repetitions", 
             steps.len(), top_repetitions);

    let mut enigo = Enigo::new(&Settings::default()).expect("Enigo init failed");

    println!("\nPress ENTER to START the workflow...");
    let _ = std::io::stdin().read_line(&mut String::new());

    // Treat top-level as a regular loop (even if repetitions = 1)
    for i in 1..=top_repetitions {
        println!("🔄 Top-level iteration {}/{}", i, top_repetitions);
        let mut rep_stack = vec![i];
        execute_steps(&mut enigo, &steps, &mut rep_stack);
    }

    println!("\n✅ Workflow completed successfully!");
    pause_to_menu();
}

fn execute_steps(enigo: &mut Enigo, steps: &[Step], rep_stack: &mut Vec<u32>) {
    for step in steps {
        match step {
            Step::Click { x, y, delay } => {
                let _ = enigo.move_mouse(*x, *y, Coordinate::Abs);
                let _ = enigo.button(Button::Left, Direction::Click);
                println!("✅ Clicked at ({}, {})", x, y);
                thread::sleep(Duration::from_secs_f64(*delay));
            }
            Step::Type { text, delay } => {
                let mut final_text = text.clone();
                // {$} = closest (innermost) repetition number
                if let Some(&last_rep) = rep_stack.last() {
                    final_text = final_text.replace("{$}", &last_rep.to_string());
                }
                let _ = enigo.text(&final_text);
                println!("✅ Typed: {}", final_text);
                thread::sleep(Duration::from_secs_f64(*delay));
            }
            Step::Loop { repetitions, actions } => {
                for i in 1..=*repetitions {
                    println!("   🔄 Loop iteration {}/{}", i, repetitions);
                    rep_stack.push(i);
                    execute_steps(enigo, actions, rep_stack);
                    rep_stack.pop();
                }
            }
        }
    }
}

// ====================== RECORDER ======================
fn record_workflow() {
    println!("\n🎥 Let's record your workflow!");
    println!("Commands:");
    println!("   ENTER → Record Click (default delay 0.1s)");
    println!("   t     → Record Type   (use {{$}} to type current loop iteration number)");
    println!("   [     → Start new nested loop");
    println!("   ]     → End current (innermost) loop");
    println!("   q     → Finish recording\n");

    let steps = Vec::new();
    let mut loop_stack: Vec<Vec<Step>> = vec![steps];

    loop {
        print!("\nCommand (ENTER/t/[/]/q): ");
        io::stdout().flush().unwrap();

        let mut cmd = String::new();
        io::stdin().read_line(&mut cmd).unwrap();
        let cmd = cmd.trim().to_lowercase();

        match cmd.as_str() {
            "q" => break,
            "[" => start_new_loop(&mut loop_stack),
            "]" => end_current_loop(&mut loop_stack),
            "t" => record_type_action(&mut loop_stack),
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

    let workflow = serde_json::json!({
        "actions": final_steps,
        "repetitions": top_reps
    });

    let json = serde_json::to_string_pretty(&workflow).unwrap();

    println!("\n{}", "=".repeat(80));
    println!("✅ RECORDING FINISHED!");
    println!("{}", json);
    println!("{}", "=".repeat(80));

    pause_to_menu();
}

fn start_new_loop(stack: &mut Vec<Vec<Step>>) {
    stack.push(Vec::new());
    println!("   ➕ Started new inner loop");
}

fn end_current_loop(stack: &mut Vec<Vec<Step>>) {
    if stack.len() <= 1 {
        println!("   ⚠️ No open loop to close");
        return;
    }

    print!("   How many repetitions for this loop? (default 1): ");
    io::stdout().flush().unwrap();
    let mut rep_str = String::new();
    io::stdin().read_line(&mut rep_str).unwrap();
    let repetitions: u32 = rep_str.trim().parse().unwrap_or(1);

    let inner = stack.pop().unwrap();
    let loop_step = Step::Loop { repetitions, actions: inner };
    stack.last_mut().unwrap().push(loop_step);
    println!("   ✅ Closed loop with {} repetitions", repetitions);
}

fn record_click_action(stack: &mut Vec<Vec<Step>>) {
    let enigo = Enigo::new(&Settings::default()).expect("Enigo failed");
    let (x, y) = enigo.location().unwrap_or((0, 0));

    print!("   Click at ({}, {}) → delay (default 0.1): ", x, y);
    io::stdout().flush().unwrap();
    let mut d = String::new();
    io::stdin().read_line(&mut d).unwrap();
    let delay: f64 = d.trim().parse().unwrap_or(0.1);

    stack.last_mut().unwrap().push(Step::Click { x, y, delay });
    println!("   ✅ Click recorded");
}

fn record_type_action(stack: &mut Vec<Vec<Step>>) {
    // let _enigo = Enigo::new(&Settings::default()).expect("Enigo failed");
    // let (_x, _y) = enigo.location().unwrap_or((0, 0));

    print!("   Text (use {{$}} for current loop number): ");
    io::stdout().flush().unwrap();
    let mut text = String::new();
    io::stdin().read_line(&mut text).unwrap();
    let text = text.trim().to_string();

    print!("   Delay (default 0.1): ");
    io::stdout().flush().unwrap();
    let mut d = String::new();
    io::stdin().read_line(&mut d).unwrap();
    let delay: f64 = d.trim().parse().unwrap_or(0.1);

    stack.last_mut().unwrap().push(Step::Type { text, delay });
    println!("   ✅ Type recorded");
}

// ====================== LOADING ======================
fn load_workflow() -> (Vec<Step>, u32) {
    let path = if let Some(arg) = env::args().nth(1) {
        PathBuf::from(arg)
    } else {
        let mut p = env::current_exe().unwrap();
        p.pop();
        p.push("workflow.json");
        p
    };

    if !path.exists() {
        println!("❌ workflow.json not found.");
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

    let data: serde_json::Value = match serde_json::from_str(&content) {
        Ok(d) => d,
        Err(_) => return (Vec::new(), 1),
    };

    let repetitions = data.get("repetitions")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    let steps: Vec<Step> = if let Some(actions) = data.get("actions") {
        serde_json::from_value(actions.clone()).unwrap_or_default()
    } else {
        Vec::new()
    };

    println!("✅ Loaded: {}", path.file_name().unwrap().to_string_lossy());
    (steps, repetitions)
}

fn show_mouse_position() {
    let enigo = Enigo::new(&Settings::default()).expect("Enigo init failed");
    println!("\n🖱️ Live position (Ctrl+C to stop)\n");
    loop {
        if let Ok((x, y)) = enigo.location() {
            print!("\rX: {:4} | Y: {:4}", x, y);
            io::stdout().flush().unwrap();
        }
        thread::sleep(Duration::from_millis(200));
    }
}

fn pause_to_menu() {
    println!("\nPress ENTER to return to menu...");
    let _ = std::io::stdin().read_line(&mut String::new());
}