use enigo::{Enigo, MouseControllable, KeyboardControllable, MouseButton};
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
    0.1
}

fn main() {
    println!("Scripted Autoclicker Tool");
    println!("==================================================\n");

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
            "1" => run_automation(),
            "2" => record_workflow(),
            "3" => show_mouse_position(),
            "4" => {
                println!("Goodbye!");
                break;
            }
            _ => println!("Invalid option.\n"),
        }
    }
}

// ====================== EXECUTION ======================
fn run_automation() {
    let (steps, top_repetitions) = load_workflow();

    if steps.is_empty() {
        println!("No actions to run. Please record a workflow first (option 2).");
        pause_to_menu();
        return;
    }

    println!(
        "Loaded workflow -- {} top-level actions x {} repetitions",
        steps.len(),
        top_repetitions
    );

    let mut enigo = Enigo::new();

    println!("\nPress ENTER to START the workflow...");
    let _ = std::io::stdin().read_line(&mut String::new());

    for i in 1..=top_repetitions {
        println!("Top-level iteration {}/{}", i, top_repetitions);
        let mut rep_stack = vec![i];
        execute_steps(&mut enigo, &steps, &mut rep_stack);
    }

    println!("\nWorkflow completed successfully!");
    pause_to_menu();
}

fn execute_steps(enigo: &mut Enigo, steps: &[Step], rep_stack: &mut Vec<u32>) {
    for step in steps {
        match step {
            Step::Click { x, y, delay } => {
                enigo.mouse_move_to(*x, *y);
                enigo.mouse_click(MouseButton::Left);
                println!("Clicked at ({}, {})", x, y);
                thread::sleep(Duration::from_secs_f64(*delay));
            }
            Step::Type { text, delay } => {
                let mut final_text = text.clone();
                if let Some(&last_rep) = rep_stack.last() {
                    final_text = final_text.replace("{$}", &last_rep.to_string());
                }
                enigo.key_sequence(&final_text);
                println!("Typed: {}", final_text);
                thread::sleep(Duration::from_secs_f64(*delay));
            }
            Step::Loop {
                repetitions,
                actions,
            } => {
                for i in 1..=*repetitions {
                    println!("   Loop iteration {}/{}", i, repetitions);
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
    println!("\nLet's record your workflow!");
    println!("Commands:");
    println!("   ENTER -> Record Click");
    println!("   t     -> Record Type   (use {{$}} to yield current (innermost) loop iteration number)");
    println!("   [     -> Start new nested loop");
    println!("   ]     -> End current (innermost) loop");
    println!("   q     -> Finish recording\n");

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


    println!("\nRECORDING FINISHED!");
    println!("Save this to a JSON file for later use:");
    println!("{}", "=".repeat(80));
    println!("\n{}", json);
    println!("\n{}", "=".repeat(80));

    print!("\nDo you want to save into the workflow.json? y/N:");
    io::stdout().flush().unwrap();
    let mut save_answer = String::new();
    io::stdin().read_line(&mut save_answer).unwrap();
    let save_answer = save_answer.trim().to_lowercase();

    if save_answer.is_empty() || save_answer == "n" || save_answer == "N"{
        println!("Workflow not saved.");
    } else {
        let mut save_path = env::current_exe().unwrap();
        save_path.pop();
        save_path.push("workflow.json");

        match fs::write(&save_path, &json) {
            Ok(_) => println!("Workflow saved to: {}", save_path.display()),
            Err(e) => println!("Failed to save workflow: {}", e),
        }
    }

    pause_to_menu();
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

    print!("   Click at ({}, {}) -> subsequent delay (default 0.1s): ", x, y);
    io::stdout().flush().unwrap();
    let mut d = String::new();
    io::stdin().read_line(&mut d).unwrap();
    let delay: f64 = d.trim().parse().unwrap_or(0.1);

    stack
        .last_mut()
        .unwrap()
        .push(Step::Click { x, y, delay });
    println!("   Click recorded");
}

fn record_type_action(stack: &mut Vec<Vec<Step>>) {
    print!("   Text (use {{$}} for current loop number): ");
    io::stdout().flush().unwrap();
    let mut text = String::new();
    io::stdin().read_line(&mut text).unwrap();
    let text = text.trim().to_string();

    print!("   Subsequent delay (default 0.1s): ");
    io::stdout().flush().unwrap();
    let mut d = String::new();
    io::stdin().read_line(&mut d).unwrap();
    let delay: f64 = d.trim().parse().unwrap_or(0.1);

    stack.last_mut().unwrap().push(Step::Type { text, delay });
    println!("   Type recorded");
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
        println!("workflow.json not found.");
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

    let repetitions = data
        .get("repetitions")
        .and_then(|v| v.as_u64())
        .unwrap_or(1) as u32;

    let steps: Vec<Step> = if let Some(actions) = data.get("actions") {
        serde_json::from_value(actions.clone()).unwrap_or_default()
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
    println!("\nLive position \ngo to [0, 0] position to stop\n");
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
    pause_to_menu();
}

fn pause_to_menu() {
    println!("\nPress ENTER to return to menu...");
    let _ = std::io::stdin().read_line(&mut String::new());
}