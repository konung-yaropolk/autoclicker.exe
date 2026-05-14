# autoclicker.exe
Scripted Autoclicker Tool written in Rust  
For legacy software automation, game cheating, repetitive task automation, and more

<img width="578" height="395" alt="image" src="https://github.com/user-attachments/assets/ec0be864-ac0e-482b-8c84-9b98641be523" />

---

## Features

- **Record & replay** mouse clicks, keyboard input, and key combinations
- **YAML workflow files** — human-readable, support `#` comments, easy to edit by hand
- **Nested loops** — repeat any sequence of actions any number of times, infinitely nestable
- **Estimated execution time** — shown before every run, calculated from your delays
- **Emergency stop** — move mouse to the upper-left corner `(0, 0)` at any time to abort
- **Live mouse position tracker** — helps you find coordinates for clicks

---

## Step Types

Workflows are defined as a list of steps in `workflow.yaml`. Each step has a `step` and an optional `delay` (in seconds, default `0.2s`) that is waited after the action executes.

### `click` — Left mouse click
```yaml
- step: click
  x: 540
  y: 300
  delay: 0.2
```

### `right_click` — Right mouse click
```yaml
- step: right_click
  x: 540
  y: 300
  delay: 0.2
```

### `text_input` — Type a string of text
Use `{$}` as a placeholder that is replaced with the current loop iteration number.
```yaml
- step: text_input
  text: "Hello world"
  delay: 0.1

- step: text_input
  text: "Item {$}"   # yields "Item 1", "Item 2", etc. inside a loop
  delay: 0.1
```

### `press_key` — Press a single special key
```yaml
- step: press_key
  key: enter
  delay: 0.2

- step: press_key
  key: f5
  delay: 0.5
```

### `hotkey` — Press a key combination
Keys are held down left-to-right and released right-to-left, matching real OS behavior.
```yaml
- step: hotkey
  keys:
    - ctrl
    - c
  delay: 0.2

- step: hotkey
  keys:
    - ctrl
    - alt
    - delete
  delay: 0.5
```

### `loop` — Repeat a block of steps
```yaml
- step: loop
  repetitions: 10
  actions:
    - step: click
      x: 200
      y: 400
    - step: press_key
      key: enter
```

---

## Key Names Reference

These names are accepted by `press_key` and inside `hotkey` combos (case-insensitive).  
Any single printable character (`a`, `z`, `1`, `+`, etc.) is also valid.

| Category | Keys |
|----------|------|
| **Navigation** | `enter` `return` `tab` `esc` `space` `backspace` `delete` `del` `insert` `ins` |
| | `up` `down` `left` `right` `home` `end` `pageup` `pagedown` |
| **Modifiers** | `ctrl` `lctrl` `rctrl` `alt` `shift` `lshift` `rshift` |
| | `super` `win` `meta` `caps` `capslock` `numlock` |
| **F-keys** | `f1` – `f20` |
| **System** | `pause` `print` `printscreen` `help` `select` `execute` `clear` `cancel` |
| **Numpad** | `num0`–`num9` `numadd` `numsub` `nummul` `numdiv` `numdec` |
| **Media** | `volup` `voldown` `mute` `medianext` `mediaprev` `mediastop` `mediaplay` `playpause` |

---

## Workflow File

By default the tool reads and saves `workflow.yaml` from the same directory as the executable.  
You can also pass a custom path as a command-line argument:

```cmd
autoclicker.exe path\to\my_workflow.yaml
```

### Full example
```yaml
# Log in and search 5 times
repetitions: 5
actions:
  # Click the username field and type
  - step: click
    x: 640
    y: 400
  - step: text_input
    text: "myuser"
  - step: press_key
    key: tab
  - step: text_input
    text: "mypassword"
  - step: press_key
    key: enter
    delay: 1.0   # wait for page load

  # Search loop
  - step: loop
    repetitions: 3
    actions:
      - step: click
        x: 800
        y: 200
      - step: text_input
        text: "query {$}"
      - step: hotkey
        keys:
          - ctrl
          - enter
        delay: 0.5
```

---

## Interactive Recorder

Launch the tool and choose **option 2** to record a workflow interactively.

| Key | Action |
|-----|--------|
| `ENTER` | Record left click at current mouse position |
| `r` | Record right click at current mouse position |
| `t` | Record text input |
| `k` | Record press key (key map is shown automatically) |
| `h` | Record hotkey combination (key map is shown automatically) |
| `[` | Start a new nested loop |
| `]` | End the current loop (prompts for repetition count) |
| `q` | Finish recording |

After recording you will be asked for the top-level repetition count, then shown the generated YAML and offered to save it as `workflow.yaml`.

---

## Building

You will need Rust installed. The project targets Windows (MSVC toolchain).

```cmd
rustup install 1.75
rustup override set 1.75
```

**32-bit (x86):**
```cmd
rustup target add i686-pc-windows-msvc
cargo build --release --target i686-pc-windows-msvc
```

**64-bit (x86_64):**
```cmd
rustup target add x86_64-pc-windows-msvc
cargo build --release --target x86_64-pc-windows-msvc
```

The compiled binary will be at `target\<target>\release\autoclicker.exe`.

---

## Dependencies

| Crate | Purpose |
|-------|---------|
| `enigo 0.1.3` | Mouse and keyboard simulation |
| `serde 1.0` | Serialization framework |
| `serde_yaml 0.8.26` | YAML workflow file parsing and saving |
