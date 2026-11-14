//! English (`en-US`) course translations for Classic and V3 curricula.
//!
//! Keeps exercise and practice identifiers tested by `cargo test` intact while
//! translating all user-facing prose, lesson blocks, demos, quizzes, hints,
//! practice prompts, and V3 blueprint metadata.

use crate::curriculum::{Block, Curriculum};

fn set_text(block: &mut Block, new_text: &str) {
    if let Block::Text { text } = block {
        *text = new_text.to_string();
    }
}

fn set_callout(block: &mut Block, new_text: &str) {
    if let Block::Callout { text, .. } = block {
        *text = new_text.to_string();
    }
}

fn set_code(
    block: &mut Block,
    new_code: Option<&str>,
    new_file: Option<&str>,
    new_explain: Option<&str>,
) {
    if let Block::Code {
        code,
        file,
        explain,
    } = block
    {
        if let Some(c) = new_code {
            *code = c.to_string();
        }
        if let Some(f) = new_file {
            *file = Some(f.to_string());
        }
        if let Some(e) = new_explain {
            *explain = Some(e.to_string());
        }
    }
}

fn set_demo(block: &mut Block, new_name: Option<&str>, new_steps: &[&str], new_caption: &str) {
    if let Block::Demo {
        name,
        steps,
        caption,
        ..
    } = block
    {
        if let Some(n) = new_name {
            *name = n.to_string();
        }
        *steps = new_steps.iter().map(|s| (*s).to_string()).collect();
        *caption = Some(new_caption.to_string());
    }
}

fn set_quiz(block: &mut Block, new_q: &str, new_opts: &[&str], new_why: &str) {
    if let Block::Quiz {
        question,
        options,
        why,
        ..
    } = block
    {
        *question = new_q.to_string();
        *options = new_opts.iter().map(|s| (*s).to_string()).collect();
        *why = Some(new_why.to_string());
    }
}

/// Translates all Classic chapters, lessons, and exercises into English (`en-US`).
pub fn translate_curriculum_to_en(cur: &mut Curriculum) {
    for ch in &mut cur.chapters {
        match ch.id.as_str() {
            "etapa-1" => {
                ch.title = "Stage 1 — First Contact".to_string();
                ch.summary =
                    "What programming is, how a program runs, and your first Rust program."
                        .to_string();
            }
            "etapa-2" => {
                ch.title = "Stage 2 — Core Programming".to_string();
                ch.summary = "Variables, mutability, types, operators, control flow, and functions: the building blocks of every program.".to_string();
            }
            "etapa-3" => {
                ch.title = "Stage 3 — Working with Data".to_string();
                ch.summary = "Strings, slices (&str), tuples, arrays, vectors, and maps to store and iterate over data.".to_string();
            }
            "etapa-4" => {
                ch.title = "Stage 4 — Understanding Rust".to_string();
                ch.summary = "Ownership, borrowing (& and &mut), and lifetimes: memory safety without a garbage collector.".to_string();
            }
            "etapa-5" => {
                ch.title = "Stage 5 — Structuring Programs".to_string();
                ch.summary = "Structs, enums, methods, traits, modules, and crates to organize real systems.".to_string();
            }
            "etapa-6" => {
                ch.title = "Stage 6 — Reliable Software".to_string();
                ch.summary =
                    "Option, Result, explicit error handling, automated tests, and documentation."
                        .to_string();
            }
            "etapa-7" => {
                ch.title = "Stage 7 — Intermediate Rust".to_string();
                ch.summary =
                    "Generics, closures, lazy iterators, and safe file handling.".to_string();
            }
            "etapa-8" => {
                ch.title = "Stage 8 — Real-World Development".to_string();
                ch.summary = "CLI apps, configuration, serialization, basic networking, and async programming.".to_string();
            }
            "etapa-9" => {
                ch.title = "Stage 9 — Guided Projects".to_string();
                ch.summary = "Hands-on blueprints with increasing challenge (calculator, game, tasks, organizer, and notes).".to_string();
            }
            "etapa-10" => {
                ch.title = "Stage 10 — Independent Project".to_string();
                ch.summary =
                    "Plan in PLANO.md, implement, test, and ship an original Rust program."
                        .to_string();
            }
            _ => {}
        }
    }

    for (id, lesson) in &mut cur.lessons {
        translate_lesson_to_en(id.as_str(), lesson);
    }

    for (id, ex) in &mut cur.exercises {
        translate_exercise_to_en(id.as_str(), ex);
    }
}

fn translate_lesson_to_en(id: &str, lesson: &mut crate::curriculum::Lesson) {
    let b = &mut lesson.blocks;
    match id {
        "o-que-e-programar" => {
            lesson.title = "What is programming?".to_string();
            if b.len() >= 4 {
                set_text(
                    &mut b[0],
                    "Programming is writing, with very precise words, a sequence of\ninstructions that a machine executes. A program is like a recipe:\nevery step must be clear, in the right order, with zero ambiguity.\n\nA machine never \"guesses\". If you say \"stir a little\", it asks\n(metaphorically): how much is a little? Programming is learning to\nanswer those questions ahead of time.",
                );
                set_callout(
                    &mut b[1],
                    "You do not need advanced math or perfect theory to start\nprogramming. You need patience, precision, and curiosity — three\nthings everyone has.",
                );
                set_text(
                    &mut b[2],
                    "Every programming language offers a small set of core tools:\n\n  · STORE information (variables)\n  · DECIDE between paths (conditions)\n  · REPEAT tasks (loops)\n  · ORGANIZE reusable steps (functions)\n\nEverything else — apps, games, operating systems — is built by\ncombining these four pieces. In this course, you will master each one in Rust.",
                );
                set_quiz(
                    &mut b[3],
                    "Which sentence best describes a program?",
                    &[
                        "A drawing that shows how a website looks.",
                        "A precise sequence of instructions that a machine executes.",
                        "A file that stores photos and documents.",
                        "A button that installs other programs.",
                    ],
                    "Program = precise instructions, in order, executed by the computer.\nDrawings are design, photo files are data, and installers are... programs!",
                );
            }
        }
        "como-um-programa-executa" => {
            lesson.title = "How does a program run?".to_string();
            if b.len() >= 5 {
                set_text(
                    &mut b[0],
                    "Computers only understand numbers (machine instructions). Your code,\nwritten in Rust, must be TRANSLATED into that language. The tool that\ndoes this translation is called a compiler.\n\n  you write     →  source code (text)\n  compiler      →  program (binary)\n  system        →  runs the program, step by step",
                );
                set_demo(
                    &mut b[1],
                    Some("step"),
                    &[
                        "you write the code",
                        "the compiler translates it",
                        "the program becomes a binary",
                        "the machine executes it",
                    ],
                    "An example of the path to execution — press d to advance steps",
                );
                set_text(
                    &mut b[2],
                    "Rust uses ahead-of-time compilation: the program is translated BEFORE\nit runs. If there is a mistake, the compiler stops translating and\nexplains exactly what happened — line, column, and reason. You will see\nits messages throughout this course; far from being an obstacle, they\nare Rust's best teacher.\n\nLanguages like Python work differently: they interpret line by line while\nrunning. Those are design trade-offs, not right vs. wrong.",
                );
                set_callout(
                    &mut b[3],
                    "Inside RustLab you can compile and run code with a single keypress\nright here in the terminal. Read every example carefully first: they\ncarry straight into the hands-on challenges.",
                );
                set_quiz(
                    &mut b[4],
                    "What does a compiler do?",
                    &[
                        "Runs the program faster.",
                        "Translates source code into a format the machine understands.",
                        "Automatically fixes any logic bug.",
                        "Downloads libraries from the internet.",
                    ],
                    "The compiler is the translator: it turns your code into machine\ninstructions. It catches syntax and type errors, though it cannot fix\nyour logic for you.",
                );
            }
        }
        "conhecendo-o-rust" => {
            lesson.title = "Getting to know Rust".to_string();
            if b.len() >= 3 {
                set_text(
                    &mut b[0],
                    "Rust was created in 2010 by Graydon Hoare at Mozilla with an ambitious\ngoal: programs as fast as C, as safe as modern languages, without a\ngarbage collector and with zero-cost abstractions.\n\nWhy does that matter when you are starting out?\n\n  · REAL SAFETY: common memory bugs (a program accessing data it\n    should not) are caught at compile time, before anything runs.\n  · THE COMPILER TEACHES: Rust error messages are famous for explaining\n    the root issue and suggesting how to fix it.\n  · ONE UNIFIED TOOL: Cargo (included with Rust) creates projects,\n    fetches dependencies, compiles, tests, and builds docs.",
                );
                set_text(
                    &mut b[1],
                    "Rust's mascot is Ferris the crab (\"crustacean\").\n\nThroughout the course you will use three modes for what you learn:\n\n  1. Learn — guided lessons and hands-on challenges (right here!)\n  2. Create — RustLab's built-in mini-IDE for your own programs\n  3. Conquer — guided and independent projects through Stage 10",
                );
                set_callout(
                    &mut b[2],
                    "If you ever get stuck on a concept, ask! You can toggle the\nbuilt-in AI Mentor at any time, inspect compiler hints in plain\nlanguage, and explore Rust documentation right inside RustLab.",
                );
            }
        }
        "hello-world" => {
            lesson.title = "Hello, world! — your first program".to_string();
            if b.len() >= 6 {
                set_text(
                    &mut b[0],
                    "Every programming course starts the same way: we make the computer say\n\"Hello, world!\". It is your program waving to the world — and proves all\nthree pieces work: you wrote it, the compiler translated it, the machine\nexecuted it.\n\nIn Rust, a minimal program looks exactly like this:",
                );
                set_code(
                    &mut b[1],
                    Some("fn main() {\n    println!(\"Hello, world!\");\n}\n"),
                    None,
                    Some(
                        "fn main() { ... }  is the entry point: every Rust binary starts executing\nat the main function. Curly braces { } delimit the body — the steps.\n\nprintln!(...)  prints a line of text to the screen. The ! means it is a\nmacro (a smart compiler shortcut) — we will cover macros later.\nThe quoted text is printed verbatim.",
                    ),
                );
                set_text(
                    &mut b[2],
                    "To compile and run this program in a standard Cargo setup, you use:\n\n  $ cargo new hello_world    # creates the project\n  $ cd hello_world\n  $ cargo run                # compiles and runs\n\nWhat you would see on screen:",
                );
                set_code(
                    &mut b[3],
                    Some("Hello, world!\n"),
                    Some("terminal output"),
                    None,
                );
                set_callout(
                    &mut b[4],
                    "Whenever a lesson includes a hands-on challenge below, press Tab to\nfocus the editor and F5 to compile and run it for real. Predict the\noutput in your head first — seasoned developers simulate the compiler\nin their heads all the time.",
                );
                set_quiz(
                    &mut b[5],
                    "Why does the program above use `fn main()`?",
                    &[
                        "Because `main` is mandatory in every Rust file, even libraries.",
                        "Because `main` is the entry point where the program begins executing.",
                        "Because `println!` only works inside a function named `main`.",
                        "Because without `main`, the program prints only 'error'.",
                    ],
                    "Every executable program begins at `main` — that is the convention\nthe system uses to know where to take the first step. Rust libraries\ndo not need a `main` function.",
                );
            }
        }
        "anatomia-do-hello-world" => {
            lesson.title = "Anatomy of Hello, world".to_string();
            if b.len() >= 5 {
                set_text(
                    &mut b[0],
                    "Let us dissect it piece by piece. Recognizing each token now prevents\nfrustration later — beginners spend most of their time decoding compiler\nmessages, and messages make much more sense once you know what every\nsymbol is supposed to do.",
                );
                set_code(
                    &mut b[1],
                    Some("fn main() {\n    println!(\"Hello, world!\");\n}"),
                    None,
                    Some(
                        "fn        → \"function\"\nmain      → name of the entry function\n( )       → function parameters (empty here: main takes nothing)\n{ }       → body: the steps to execute\n;         → end of a statement (semicolon is required here)\n\"...\"     → string literal (text)",
                    ),
                );
                set_text(
                    &mut b[2],
                    "Details that C-style tradition often hides from beginners:\n\n  · Indentation (4 spaces) does not change execution, but improves readability.\n    Rust ships an automatic formatter, `rustfmt`, to standardize this for you.\n  · `println!` appends a newline automatically. Use `print!` when you do\n    NOT want a line break.\n  · Rust is case-sensitive: `main` is a function (lowercase); core types\n    usually start uppercase (`String`, `Vec`).",
                );
                set_callout(
                    &mut b[3],
                    "Classic typo: writing `printn!` (swapping letters). The compiler will\ncomplain and kindly suggest: \"did you mean: println!\". Reading the\nsuggestion through to the end solves 80% of beginner errors.",
                );
                set_quiz(
                    &mut b[4],
                    "What happens if you forget the semicolon on the `println!` statement?",
                    &[
                        "Nothing: Rust is loose like Python.",
                        "The program runs, but prints twice.",
                        "The compiler reports an error at the exact line and column.",
                        "The program compiles, but skips the statement.",
                    ],
                    "`;` terminates a statement. Rust has strict syntax — and that is a\ngift: strictness means the compiler catches slips BEFORE users see\nthem, pointing straight to the line, column, and reason.",
                );
            }
        }
        "variaveis" => {
            lesson.title = "Variables: labeled boxes".to_string();
            if b.len() >= 6 {
                set_text(
                    &mut b[0],
                    "A variable is a value stored under a name for later use. Think of a\nbox with a label on it: inside goes ONE value; the label is the name\nyou use to inspect what is inside.\n\nIn Rust, you create a variable with `let`:",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn main() {\n    let age = 25;\n    let city = \"Curitiba\";\n    println!(\"{city} has someone aged {age}!\");\n}\n",
                    ),
                    None,
                    Some(
                        "let age = 25;       → box named \"age\" holding 25\nlet city = \"...\";  → box named \"city\" holding text\n\n{city} inside quotes is string interpolation: the variable's value is\nformatted right in place of its name.",
                    ),
                );
                set_demo(
                    &mut b[2],
                    Some("age"),
                    &["25", "26", "30", "42"],
                    "The 'age' box holding its current value (d advances)",
                );
                set_text(
                    &mut b[3],
                    "A box keeps ONE label, and naming follows a few clear rules:\n\n  · names start with a letter or `_`, and use letters, digits, and `_`\n  · community convention is `snake_case` — `customer_age`, not `CustomerAge`\n  · `=` means \"bind/assign\", not mathematical equality:\n    `let age = 25;` reads as \"`age` now holds 25\"",
                );
                set_callout(
                    &mut b[4],
                    "Notice we did not annotate variable types in the examples. Rust INFERS\nthe type (25 looks like an integer, \"Curitiba\" looks like text) — and\nlocks that type down. Best of both worlds: clean syntax plus compiler rigor.",
                );
                set_quiz(
                    &mut b[5],
                    "Which statement declares a valid variable in Rust?",
                    &[
                        "var height = 1.80;",
                        "let height = 1.80;",
                        "height := 1.80;",
                        "define height 1.80;",
                    ],
                    "The keyword to create variables in Rust is `let`. `var` comes from older JavaScript, `:=` belongs to other languages.",
                );
            }
        }
        "mutabilidade" => {
            lesson.title = "Mutability: boxes that can change".to_string();
            if b.len() >= 6 {
                set_text(
                    &mut b[0],
                    "Surprise: variables you created so far CANNOT change. `let age = 25;`\nseals the box. Writing `age = 26;` later fails at compile time — not\nat runtime.\n\nThat sounds picky at first, but it is not: countless bugs happen when\nany part of a program quietly mutates state by accident. Rust asks you\nto opt in explicitly: \"this box will change\" using `mut`:",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn main() {\n    let mut counter = 0;\n    counter = counter + 1;\n    counter = counter + 1;\n    println!(\"Counter: {counter}\");\n}\n",
                    ),
                    None,
                    Some(
                        "let mut counter = 0;   → box open to changes\ncounter = counter + 1; → overwrites contents\n\nThe program counts up to 2 and prints Counter: 2.",
                    ),
                );
                set_demo(
                    &mut b[2],
                    Some("counter (mut)"),
                    &["0", "1", "2"],
                    "A mutable box receiving new values — watch the label (d advances)",
                );
                set_text(
                    &mut b[3],
                    "Rule of thumb: write plain `let`; whenever the compiler tells you\n(\"cannot assign twice\"), add `mut` if mutation is actually intended.\nBy default, data stays frozen and predictable.",
                );
                set_callout(
                    &mut b[4],
                    "Common beginner mistake: reassigning a variable without `mut`.\nThe compiler nearly always tells you the fix right inline:\n\"cannot assign to `counter` ... consider marking it as mutable: `mut counter`\".\nRead to the end and follow the hint.",
                );
                set_quiz(
                    &mut b[5],
                    "What does this code do?\n\n    let x = 1;\n    x = 2;",
                    &[
                        "Runs and prints 2.",
                        "Crashes at runtime.",
                        "Does not compile: `x` was not declared mutable.",
                        "Runs, but `x` keeps the value 1.",
                    ],
                    "Variables without `mut` are immutable, so accidental reassignment is\ncaught at COMPILE time — before any user ever runs it.",
                );
            }
        }
        "tipos-numericos" => {
            lesson.title = "Numeric types and text".to_string();
            if b.len() >= 5 {
                set_text(
                    &mut b[0],
                    "Rust has explicit types and several kinds of numbers. The core ones:\n\n  signed integers:    i8 i16 i32 i64 i128   (can be negative)\n  unsigned integers:  u8 u16 u32 u64 u128   (0 and above only)\n  pointer-sized:      usize                 (counts, collection lengths, indices)\n  floating-point:     f32 f64               (decimals)\n\nWriting `25` with no suffix defaults to `i32`.\nWriting `1.8` defaults to `f64`.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn main() {\n    let score: i32 = -10;\n    let age: u8 = 25;\n    let height: f64 = 1.80;\n    let total_items: usize = age as usize + 3;\n    println!(\"{score} {age} {height:.2} {total_items}\");\n}\n",
                    ),
                    None,
                    Some(
                        ":type      → annotate explicitly when you do not want the default.\nu8 holds 0..=255 (age fits; -1 does not). i32 allows negatives.\nas usize   → explicit cast: Rust never mixes numeric types silently;\n             conversions stay visible in code.\n{height:.2} → prints with two decimal places (format specifier).",
                    ),
                );
                set_text(
                    &mut b[2],
                    "In Rust, text also has two foundational types — which return in Stages 3 & 4:\n\n  &str   → borrowed string slice, fixed view (e.g. literal \"hi\")\n  String → owned growable text on the heap (e.g. String::from(\"hi\"))\n\nFor now: double quotes produce `&str`, and `println!` works with both.",
                );
                set_callout(
                    &mut b[3],
                    "`u8` for age is a design choice (1 byte, 0..=255). `i32` is the sensible\ndefault for everyday signed numbers, and `usize` is standard for sizes\nand indices. When unsure between `i32` and `u32`, start with `i32`.",
                );
                set_quiz(
                    &mut b[4],
                    "You need to store how many items are in a list (never negative, could be huge). Best type?",
                    &["i8", "u8", "usize", "f64"],
                    "`usize` is the type for counts and indices: never negative and scales to\nyour CPU architecture (64-bit on modern desktops). `i8`/`u8` overflow at 255,\nand `f64` is for decimal measurements, not exact item counts.",
                );
            }
        }
        "operadores" => {
            lesson.title = "Operators: arithmetic and comparison".to_string();
            if b.len() >= 4 {
                set_text(
                    &mut b[0],
                    "Operators compute values and compare data. In Rust they use familiar\nsymbols — with two precision rules that prevent classic bugs.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn main() {\n    println!(\"{}\", 7 + 3);   // 10\n    println!(\"{}\", 7 - 3);   // 4\n    println!(\"{}\", 7 * 3);   // 21\n    println!(\"{}\", 7 / 2);   // 3 (!)\n    println!(\"{}\", 7 % 2);   // 1 (division remainder)\n\n    println!(\"{}\", 7 > 3);   // true\n    println!(\"{}\", 7 == 3);  // false\n    println!(\"{}\", 7 != 3);  // true\n}\n",
                    ),
                    None,
                    Some(
                        "The two rules to watch:\n\n1) INTEGER DIVISION: 7 / 2 evaluates to 3, NOT 3.5. Integer divided by\n   integer truncates toward zero. Need 3.5? Use floats: 7.0 / 2.0.\n\n2) `==` compares, `=` binds. Accidentally typing a single `=` inside\n   a condition fails to compile because Rust never treats integers as booleans.",
                    ),
                );
                set_text(
                    &mut b[2],
                    "`%` (remainder / modulo) is more useful than it first seems:\n\n  · even check?        n % 2 == 0\n  · 12-hour clock?     hour % 12\n  · wrap around list?  index % len\n\nAlso, the compiler refuses to compare integers of different signedness\n(`i32` vs `u32`), catching subtle mismatch bugs early.",
                );
                set_quiz(
                    &mut b[3],
                    "What is the value of `9 / 2` in Rust with integers?",
                    &["4.5", "4", "5", "compile error"],
                    "Integer division truncates the fractional part: `9 / 2 = 4`. Write\n`9.0 / 2.0` if you want floating-point `4.5`.",
                );
            }
        }
        "condicoes-lacos-e-funcoes" => {
            lesson.title = "Conditions, loops, and functions".to_string();
            if b.len() >= 5 {
                set_text(
                    &mut b[0],
                    "Programs branch with `if`/`else`, repeat work with `for`, `while`, and `loop`, and group reusable logic into functions (`fn`). In Rust, `if` is also an expression: it can evaluate to and return a value.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn classify(grade: i32) -> &'static str {\n    if grade >= 7 {\n        \"passed\"\n    } else {\n        \"studying\"\n    }\n}\n\nfn main() {\n    let mut sum = 0;\n    for n in 1..=4 {\n        sum += n;\n    }\n    println!(\"sum = {sum}, status = {}\", classify(sum));\n}\n",
                    ),
                    None,
                    Some(
                        "Notice the last expression of `classify` has NO `;`: in Rust that returns the expression's value automatically.",
                    ),
                );
                set_demo(
                    &mut b[2],
                    Some("1..=4"),
                    &[
                        "n = 1 -> sum becomes 1",
                        "n = 2 -> sum becomes 3",
                        "n = 3 -> sum becomes 6",
                        "n = 4 -> sum becomes 10 (end of 1..=4)",
                    ],
                    "Step-by-step execution of a `for` loop",
                );
                set_demo(
                    &mut b[3],
                    Some("double"),
                    &["input x = 5", "computes x * 2", "returns 10"],
                    "A function maps input to output",
                );
                set_quiz(
                    &mut b[4],
                    "In `fn double(x: i32) -> i32 { x * 2 }`, why is there no `;` after `x * 2`?",
                    &[
                        "Because Rust forbids semicolons inside functions.",
                        "Because without `;`, `x * 2` is the function's return expression.",
                        "Because `*` automatically ends statements.",
                    ],
                    "In Rust, adding `;` turns an expression into a statement producing `()` (unit); leaving `;` off returns the value.",
                );
            }
        }
        "strings-e-fatias" => {
            lesson.title = "Strings and slices (&str)".to_string();
            if b.len() >= 5 {
                set_text(
                    &mut b[0],
                    "Rust has two primary UTF-8 text types: `String` (owns its heap buffer and can grow) and `&str` (a borrowed view into existing UTF-8 text, such as literals `\"hello\"`).",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn greet(name: &str) -> String {\n    format!(\"Hello, {name}!\")\n}\n\nfn main() {\n    let mut sentence = greet(\"RustLab\");\n    sentence.push_str(\" Let's code.\");\n    println!(\"{sentence} (bytes: {})\", sentence.len());\n}\n",
                    ),
                    None,
                    None,
                );
                set_demo(
                    &mut b[2],
                    Some("String"),
                    &[
                        "Stack: ptr -> Heap: [R,u,s,t] (len=4, cap=4)",
                        "push_str(\"!\") -> Heap reallocates if capacity is full",
                        "When owner scope ends, Heap buffer is freed automatically",
                    ],
                    "Stack holds ptr/len/capacity; Heap stores the UTF-8 bytes",
                );
                set_callout(
                    &mut b[3],
                    "Prefer `&str` for read-only text parameters: callers can pass both string literals `\"abc\"` and `&my_string` without extra heap allocations.",
                );
                set_quiz(
                    &mut b[4],
                    "What is the practical difference between `String` and `&str`?",
                    &[
                        "`String` owns growable heap text; `&str` is a borrowed slice viewing text.",
                        "`&str` supports UTF-8 accents while `String` is ASCII-only.",
                        "Both types are identical and always interchangeable.",
                    ],
                    "`String` manages an owned heap allocation; `&str` borrows a slice without taking ownership.",
                );
            }
        }
        "tuplas-arrays-e-vetores" => {
            lesson.title = "Tuples, arrays, and vectors".to_string();
            if b.len() >= 4 {
                set_text(
                    &mut b[0],
                    "To group multiple values: **tuples** `(i32, bool)` combine heterogeneous fixed slots; **arrays** `[i32; 3]` hold a compile-time fixed count of one type; and **vectors** `Vec<T>` grow and shrink dynamically.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn main() {\n    let point = (10, 20);\n    let days = [\"mon\", \"tue\", \"wed\"];\n    let mut grades = vec![7, 8];\n    grades.push(10);\n    println!(\"x={}, day={}, grades={:?}\", point.0, days[1], grades);\n}\n",
                    ),
                    None,
                    None,
                );
                set_demo(
                    &mut b[2],
                    Some("i32"),
                    &[
                        "[ 7, 8 ]  (len = 2)",
                        "push(10) -> [ 7, 8, 10 ]  (len = 3)",
                        "pop() -> Some(10), leaving [ 7, 8 ]",
                    ],
                    "Pushing and popping at the tail of a `Vec<i32>`",
                );
                set_quiz(
                    &mut b[3],
                    "Which collection should you choose when the number of items changes at runtime?",
                    &["Tuple `(T, U)`", "Array `[T; 4]`", "Vector `Vec<T>`"],
                    "`Vec<T>` supports `.push()`, `.pop()`, and dynamic heap resizing at runtime.",
                );
            }
        }
        "colecoes-e-iteracao" => {
            lesson.title = "Collections and safe iteration".to_string();
            if b.len() >= 4 {
                set_text(
                    &mut b[0],
                    "Beyond `Vec<T>`, the standard library provides `HashMap<K, V>` for key-value lookups and `.get(idx)` for bounds-checked indexing without risking out-of-bounds panics.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "use std::collections::HashMap;\n\nfn main() {\n    let mut scores = HashMap::new();\n    scores.insert(\"Ana\", 12);\n    scores.insert(\"Bia\", 18);\n\n    for (name, value) in &scores {\n        println!(\"{name}: {value}\");\n    }\n    println!(\"Safe lookup: {:?}\", [10, 20].get(5));\n}\n",
                    ),
                    None,
                    None,
                );
                set_callout(
                    &mut b[2],
                    "Writing `for x in &collection` iterates by borrowing, keeping the collection alive after the loop finishes.",
                );
                set_quiz(
                    &mut b[3],
                    "Why is `list.get(99)` safer than `list[99]` when an index may be out of bounds?",
                    &[
                        "`get` returns `Option`, giving `None` instead of panicking.",
                        "`get` automatically pads zeros up to the index.",
                        "`get` bypasses the type checker.",
                    ],
                    "`slice::get` returns `Some(&item)` or `None`, preventing runtime crashes.",
                );
            }
        }
        "ownership-na-pratica" => {
            lesson.title = "Ownership in practice".to_string();
            if b.len() >= 4 {
                set_text(
                    &mut b[0],
                    "Every value in Rust has a single **owner**. When that owner leaves scope, the value is automatically dropped — no garbage collector and no manual `free`. Passing a non-`Copy` value (like `String`) by value transfers ownership (*move semantics*).",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn consume(text: String) {\n    println!(\"received: {text}\");\n} // `text` is dropped here\n\nfn main() {\n    let name = String::from(\"Ferris\");\n    consume(name);\n    // println!(\"{name}\"); // ERROR E0382: ownership already moved into `consume`\n}\n",
                    ),
                    None,
                    None,
                );
                set_demo(
                    &mut b[2],
                    Some("name: String"),
                    &[
                        "main owns `name` (\"Ferris\")",
                        "calling `consume(name)` moves ownership into parameter `text`",
                        "main loses access to `name` (compiler rejects use-after-move)",
                        "`consume` returns -> memory freed exactly once",
                    ],
                    "Follow the transfer of ownership (move)",
                );
                set_quiz(
                    &mut b[3],
                    "Why does Rust forbid using `name` after calling `consume(name)`?",
                    &[
                        "To prevent double frees and use-after-free bugs on freed memory.",
                        "Because `String` values can only hold 5 characters.",
                        "Because Rust functions cannot accept parameters.",
                    ],
                    "Enforcing a single owner prevents double-free and dangling pointer bugs at compile time.",
                );
            }
        }
        "borrowing-e-referencias" => {
            lesson.title = "Borrowing and references".to_string();
            if b.len() >= 4 {
                set_text(
                    &mut b[0],
                    "You do not always want to transfer ownership just to read or update data. **Borrowing** creates references: `&T` allows multiple concurrent readers; `&mut T` grants exclusive access to mutate.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn count_chars(s: &str) -> usize {\n    s.chars().count()\n}\n\nfn append_bang(s: &mut String) {\n    s.push('!');\n}\n\nfn main() {\n    let mut msg = String::from(\"Rust\");\n    println!(\"chars = {}\", count_chars(&msg));\n    append_bang(&mut msg);\n    println!(\"{msg}\");\n}\n",
                    ),
                    None,
                    None,
                );
                set_demo(
                    &mut b[2],
                    Some("msg"),
                    &[
                        "1+ shared readers `&msg` at once -> ALLOWED",
                        "1 exclusive writer `&mut msg` -> ALLOWED",
                        "`&msg` and `&mut msg` overlapping -> BLOCKED (prevents data races)",
                    ],
                    "Borrow checker rules",
                );
                set_quiz(
                    &mut b[3],
                    "How many active `&mut T` references to the same value can exist at the same time?",
                    &[
                        "As many as you want.",
                        "At most one at a time, with no active `&T` references alongside it.",
                        "None: `&mut` does not exist in functions.",
                    ],
                    "Exclusivity of `&mut T` stops unexpected concurrent mutation while anyone else reads the data.",
                );
            }
        }
        "tempo-de-vida-introducao" => {
            lesson.title = "Introduction to lifetimes".to_string();
            if b.len() >= 4 {
                set_text(
                    &mut b[0],
                    "Every reference has a **lifetime**: the region of code where it remains valid. Usually the compiler infers lifetimes automatically; when a function takes two references and returns one, we annotate `'a` to state that the returned slice lives as long as both inputs do.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn longer<'a>(x: &'a str, y: &'a str) -> &'a str {\n    if x.len() >= y.len() { x } else { y }\n}\n\nfn main() {\n    let a = String::from(\"crustacean\");\n    let b = String::from(\"rust\");\n    println!(\"longer: {}\", longer(&a, &b));\n}\n",
                    ),
                    None,
                    None,
                );
                set_callout(
                    &mut b[2],
                    "Lifetimes (`'a`) **never extend** how long a value lives: they purely describe relationships between input and output references so the borrow checker proves no dangling reference can be returned.",
                );
                set_quiz(
                    &mut b[3],
                    "What is the purpose of `'a` in `fn longer<'a>(x: &'a str, y: &'a str) -> &'a str`?",
                    &[
                        "Ties the returned reference's lifetime to the shorter lifetime of `x` and `y`.",
                        "Automatically converts `&str` into a heap-allocated `String`.",
                        "Disables memory-safety checks.",
                    ],
                    "`'a` proves at compile time that the returned reference cannot outlive its backing inputs.",
                );
            }
        }
        "structs-e-metodos" => {
            lesson.title = "Structs and methods".to_string();
            if b.len() >= 3 {
                set_text(
                    &mut b[0],
                    "`struct` bundles related fields into a single domain type. `impl` blocks attach methods (`&self`, `&mut self`) and constructors (`Self::new`) directly to the type.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "struct Rectangle {\n    width: u32,\n    height: u32,\n}\n\nimpl Rectangle {\n    fn area(&self) -> u32 {\n        self.width * self.height\n    }\n}\n\nfn main() {\n    let r = Rectangle { width: 4, height: 5 };\n    println!(\"area = {}\", r.area());\n}\n",
                    ),
                    None,
                    None,
                );
                set_quiz(
                    &mut b[2],
                    "In `fn area(&self) -> u32`, what does `&self` mean?",
                    &[
                        "The method consumes and destroys the struct when done.",
                        "The method borrows the current instance immutably for reading.",
                        "`area` has no access to fields of the struct.",
                    ],
                    "`&self` is an immutable borrow of the instance, leaving the caller with ownership.",
                );
            }
        }
        "enums-e-pattern-matching" => {
            lesson.title = "Enums and pattern matching".to_string();
            if b.len() >= 4 {
                set_text(
                    &mut b[0],
                    "`enum` models a value that can exist in **one of several valid states**. Each variant may carry its own data. With `match`, the compiler ensures you handle **every** possible variant.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "enum Command {\n    Move { x: i32, y: i32 },\n    Pause,\n}\n\nfn run(cmd: Command) {\n    match cmd {\n        Command::Move { x, y } => println!(\"moving to ({x}, {y})\"),\n        Command::Pause => println!(\"paused\"),\n    }\n}\n\nfn main() {\n    run(Command::Move { x: 2, y: 3 });\n}\n",
                    ),
                    None,
                    None,
                );
                set_callout(
                    &mut b[2],
                    "If you later add a new variant to an `enum`, the compiler immediately highlights every `match` expression that must handle it.",
                );
                set_quiz(
                    &mut b[3],
                    "What is the key guarantee `match` provides when handling an `enum`?",
                    &[
                        "Exhaustiveness: all possible cases must be handled.",
                        "Executes all variants in parallel.",
                        "Silently ignores unhandled variants.",
                    ],
                    "Rust `match` arms must be exhaustive, preventing forgotten state cases.",
                );
            }
        }
        "traits-modulos-e-cargo" => {
            lesson.title = "Traits, modules, and Cargo".to_string();
            if b.len() >= 3 {
                set_text(
                    &mut b[0],
                    "**Traits** define shared behavior (like `Display`, `Debug`, or your own interfaces). **Modules** (`mod`, `pub`, `use`) split larger codebases into clean files, while **Cargo** manages crates and dependencies.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "trait Describable {\n    fn summary(&self) -> String;\n}\n\nstruct Task {\n    title: String,\n}\n\nimpl Describable for Task {\n    fn summary(&self) -> String {\n        format!(\"[ ] {}\", self.title)\n    }\n}\n\nfn main() {\n    let t = Task { title: \"Study traits\".into() };\n    println!(\"{}\", t.summary());\n}\n",
                    ),
                    None,
                    None,
                );
                set_quiz(
                    &mut b[2],
                    "What does `pub` before a function inside `mod util;` accomplish?",
                    &[
                        "Makes the function visible outside module `util`.",
                        "Turns the function into a mutable global variable.",
                        "Skips running unit tests on that function.",
                    ],
                    "Items inside a module are private by default; `pub` exposes the public API.",
                );
            }
        }
        "option-e-ausencia" => {
            lesson.title = "Option and explicit absence".to_string();
            if b.len() >= 3 {
                set_text(
                    &mut b[0],
                    "Rust has no bare `null`. When a value can legitimately be missing, we return `Option<T>` (`Some(T)` or `None`). This forces callers to explicitly handle missing values via `match`, `if let`, or `.unwrap_or(...)`.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn find_even(nums: &[i32]) -> Option<i32> {\n    for &n in nums {\n        if n % 2 == 0 {\n            return Some(n);\n        }\n    }\n    None\n}\n\nfn main() {\n    let val = find_even(&[1, 3, 8]).unwrap_or(0);\n    println!(\"found: {val}\");\n}\n",
                    ),
                    None,
                    None,
                );
                set_quiz(
                    &mut b[2],
                    "When should you prefer `Option<T>` instead of returning a sentinel like `-1`?",
                    &[
                        "Whenever absence is possible: the type system keeps callers from forgetting the `None` case.",
                        "Only when type `T` is `bool`.",
                        "Never; `-1` is checked statically by the compiler.",
                    ],
                    "`Option<T>` encodes optionality directly in the function signature.",
                );
            }
        }
        "result-e-propagacao" => {
            lesson.title = "Result and `?` propagation".to_string();
            if b.len() >= 4 {
                set_text(
                    &mut b[0],
                    "For operations that can fail with a reason (opening files, parsing strings, dividing by zero), Rust uses `Result<T, E>` with `Ok(T)` and `Err(E)`. The `?` operator propagates `Err` upward automatically.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn divide(a: i32, b: i32) -> Result<i32, String> {\n    if b == 0 {\n        Err(String::from(\"division by zero\"))\n    } else {\n        Ok(a / b)\n    }\n}\n\nfn compute() -> Result<i32, String> {\n    let q = divide(20, 4)?;\n    Ok(q + 1)\n}\n\nfn main() {\n    println!(\"{:?}\", compute());\n}\n",
                    ),
                    None,
                    None,
                );
                set_callout(
                    &mut b[2],
                    "Avoid `.unwrap()` outside tests and quick prototypes: if it hits `Err`, the program panics and crashes immediately.",
                );
                set_quiz(
                    &mut b[3],
                    "What does the `?` operator do when given `Err(e)`?",
                    &[
                        "Turns the error into zero and keeps going.",
                        "Returns early from the current function with `Err(e)`.",
                        "Prints the error and ignores the failure.",
                    ],
                    "On `Ok(v)` it unwraps `v`; on `Err(e)` it immediately returns `Err(e)` from the function.",
                );
            }
        }
        "testes-documentacao-e-depuracao" => {
            lesson.title = "Testing, docs, and debugging".to_string();
            if b.len() >= 3 {
                set_text(
                    &mut b[0],
                    "Reliable code pairs automated `#[test]` suites, inline `///` doc comments (for `cargo doc` and IDE hover), and clean diagnostics with `dbg!(&value)` and `cargo clippy`.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "/// Adds two integers with clear Rustdoc documentation.\npub fn add(a: i32, b: i32) -> i32 {\n    a + b\n}\n\n#[cfg(test)]\nmod tests {\n    use super::add;\n\n    #[test]\n    fn adds_positive_and_negative() {\n        assert_eq!(add(2, 3), 5);\n        assert_eq!(add(-4, 4), 0);\n    }\n}\n",
                    ),
                    None,
                    None,
                );
                set_quiz(
                    &mut b[2],
                    "Which Cargo subcommand catches common anti-patterns and suggests idiomatic improvements beyond standard compiler errors?",
                    &["`cargo clippy`", "`cargo init`", "`cargo clean`"],
                    "`cargo clippy` is Rust's official linter; in Create Mode run it with Ctrl+L or Alt+C.",
                );
            }
        }
        "genericos-e-closures" => {
            lesson.title = "Generics and closures".to_string();
            if b.len() >= 3 {
                set_text(
                    &mut b[0],
                    "**Generics** (`<T>`) let you write functions and structs across many types with full compile-time safety. **Closures** (`|x| x * 2`) are concise anonymous functions passed into adapters, sorting, and callbacks.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn first<T>(slice: &[T]) -> Option<&T> {\n    slice.first()\n}\n\nfn main() {\n    let factor = 3;\n    let multiply = |x: i32| x * factor;\n    println!(\"first = {:?}, mult = {}\", first(&[10, 20]), multiply(4));\n}\n",
                    ),
                    None,
                    None,
                );
                set_quiz(
                    &mut b[2],
                    "What advantage do `<T>` generics give in Rust over an unchecked dynamic any-type?",
                    &[
                        "The compiler checks types statically and generates specialized code for each concrete type.",
                        "They turn off type checking at compile time.",
                        "They only work for `i32` integers.",
                    ],
                    "Monomorphization produces type-checked, fast machine code per concrete type.",
                );
            }
        }
        "iteradores-e-arquivos" => {
            lesson.title = "Iterators and files".to_string();
            if b.len() >= 3 {
                set_text(
                    &mut b[0],
                    "**Iterators** compose lazy pipelines (`.iter()`, `.filter()`, `.map()`, `.sum()`, `.collect()`) without allocating intermediate vectors. For files, `std::fs` provides straightforward APIs backed by `Result`.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "use std::fs;\n\nfn sum_evens(nums: &[i32]) -> i32 {\n    nums.iter().copied().filter(|n| n % 2 == 0).sum()\n}\n\nfn main() -> std::io::Result<()> {\n    let total = sum_evens(&[1, 2, 3, 4, 6]);\n    println!(\"even sum = {total}\");\n    let exists = fs::metadata(\"Cargo.toml\").is_ok();\n    println!(\"Cargo.toml present? {exists}\");\n    Ok(())\n}\n",
                    ),
                    None,
                    None,
                );
                set_quiz(
                    &mut b[2],
                    "What does it mean that Rust iterators are *lazy*?",
                    &[
                        "They only pull and process items when a consumer (`.collect()`, `.sum()`, `for`) drives them.",
                        "They fail whenever there are more than 10 items.",
                        "They keep running forever in the background.",
                    ],
                    "Adapters like `.map` and `.filter` describe transformations; work runs only inside the consumer.",
                );
            }
        }
        "cli-configuracao-e-serde" => {
            lesson.title = "CLI, configuration, and serialization".to_string();
            if b.len() >= 4 {
                set_text(
                    &mut b[0],
                    "Real CLI tools inspect arguments (`std::env::args()`), follow platform config directories, and serialize settings via `serde` + `toml`/`json`. Inside RustLab's Create Mode you add crates with `F9` (`cargo add`).",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn main() {\n    let args: Vec<String> = std::env::args().skip(1).collect();\n    if args.is_empty() {\n        println!(\"Usage: my_app <file>\");\n        return;\n    }\n    println!(\"Received args: {:?}\", args);\n}\n",
                    ),
                    None,
                    None,
                );
                set_callout(
                    &mut b[2],
                    "Combine `serde::Serialize` and `serde::Deserialize` with `#[serde(default)]` on config structs so older config files stay compatible when you add fields.",
                );
                set_quiz(
                    &mut b[3],
                    "Why skip the first item in `std::env::args().skip(1)`?",
                    &[
                        "Because index 0 usually holds the path or name of the executable itself.",
                        "Because Rust discards even-indexed strings.",
                        "Because index 0 is reserved for the OS password.",
                    ],
                    "`argv[0]` identifies the invoked binary; user arguments begin at index 1.",
                );
            }
        }
        "rede-e-programacao-assincrona" => {
            lesson.title = "Basic networking and async programming".to_string();
            if b.len() >= 4 {
                set_text(
                    &mut b[0],
                    "`std::net` lets you build simple synchronous TCP connections, while `async fn` paired with a runtime like `tokio` lets you await many concurrent I/O tasks (sockets, timers, child processes) without blocking.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "use std::net::TcpListener;\n\nfn bind_ephemeral_port() -> std::io::Result<u16> {\n    let listener = TcpListener::bind(\"127.0.0.1:0\")?;\n    Ok(listener.local_addr()?.port())\n}\n\nfn main() {\n    match bind_ephemeral_port() {\n        Ok(port) => println!(\"listening temporarily on port {port}\"),\n        Err(e) => eprintln!(\"network error: {e}\"),\n    }\n}\n",
                    ),
                    None,
                    None,
                );
                set_callout(
                    &mut b[2],
                    "In async code (`tokio`), never block a runtime worker thread with heavy synchronous I/O: use async APIs or `spawn_blocking`.",
                );
                set_quiz(
                    &mut b[3],
                    "What does an `async fn` return before `.await` is applied?",
                    &[
                        "The computed value already evaluated eagerly.",
                        "A `Future` representing suspended async work driven by a runtime.",
                        "A separate OS process.",
                    ],
                    "Rust `Future`s are inert until polled via `.await` or an async executor.",
                );
            }
        }
        "projetos-guiados-calculadora-e-jogo" => {
            lesson.title = "Guided Projects: Calculator and Guessing Game".to_string();
            if b.len() >= 4 {
                set_text(
                    &mut b[0],
                    "Time to build larger apps inside **Create Mode**! In this first blueprint you build two classics step by step: a **Calculator** parsing operations with `Result`, and a **Guessing Game** looping over `stdin` input.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "fn evaluate(op: &str, a: f64, b: f64) -> Result<f64, String> {\n    match op {\n        \"+\" => Ok(a + b),\n        \"-\" => Ok(a - b),\n        \"*\" => Ok(a * b),\n        \"/\" if b != 0.0 => Ok(a / b),\n        \"/\" => Err(\"division by zero\".into()),\n        _ => Err(format!(\"unknown operator: {op}\")),\n    }\n}\n\nfn main() {\n    println!(\"10 / 2 = {:?}\", evaluate(\"/\", 10.0, 2.0));\n}\n",
                    ),
                    None,
                    None,
                );
                set_callout(
                    &mut b[2],
                    "Start a project in Create Mode (`Ctrl+N`), separate pure logic (`evaluate`) from `stdin` reading so `F6` (`cargo test`) can verify it quickly, and test interactive runs with `F5`.",
                );
                set_quiz(
                    &mut b[3],
                    "Why isolate pure calculator/game logic into standalone functions separate from `main()`?",
                    &[
                        "So fast, deterministic unit tests (`#[test]`) can verify it without manual typing.",
                        "Because `fn main()` is not allowed to contain `match`.",
                        "Because Cargo requires at least 10 functions per file.",
                    ],
                    "Pure functions are straightforward to test automatically with `cargo test` as projects grow.",
                );
            }
        }
        "projetos-guiados-tarefas-e-notas" => {
            lesson.title = "Guided Projects: Tasks, File Organizer, and CLI Notes".to_string();
            if b.len() >= 4 {
                set_text(
                    &mut b[0],
                    "Stepping up: build everyday CLI utilities inside **Create Mode** — a **Task Manager**, a `std::fs` **File Organizer**, or a persistent **CLI Notebook** saved to disk.",
                );
                set_code(
                    &mut b[1],
                    Some(
                        "#[derive(Debug, PartialEq, Eq)]\npub struct Task {\n    pub title: String,\n    pub done: bool,\n}\n\npub fn add_task(list: &mut Vec<Task>, title: &str) {\n    if !title.trim().is_empty() {\n        list.push(Task {\n            title: title.trim().to_string(),\n            done: false,\n        });\n    }\n}\n",
                    ),
                    None,
                    None,
                );
                set_callout(
                    &mut b[2],
                    "Progression tip: start saving lines to a plain file, then add CLI subcommands (`list`, `add`, `done`), `///` docs, and clean `cargo clippy` checks (`Ctrl+L`).",
                );
                set_quiz(
                    &mut b[3],
                    "When writing a CLI file organizer, what check avoids accidentally overwriting existing files?",
                    &[
                        "Checking whether the target path already exists (`dest.exists()`) before moving or copying.",
                        "Ignoring errors with `.unwrap()`.",
                        "Deleting the whole directory before running.",
                    ],
                    "Checking target existence and permissions keeps real user files safe.",
                );
            }
        }
        "projeto-independente-planejamento" => {
            lesson.title = "Independent Project: Planning (PLANO.md)".to_string();
            if b.len() >= 4 {
                set_text(
                    &mut b[0],
                    "In Stage 10, the lab is yours! Design and build an original Rust program from scratch inside **Create Mode**. Before writing lots of code, capture the problem, modules, data structures, and verifiable milestones in **`PLANO.md`**.",
                );
                set_callout(
                    &mut b[1],
                    "Inside Create Mode, press **`Alt+M`** to generate a starter `PLANO.md` template in your project (it never overwrites an existing edited file).",
                );
                set_code(
                    &mut b[2],
                    Some(
                        "# My RustLab Project\n- Problem: ...\n- Commands / Core flow: ...\n- Structs and modules: ...\n- Completion criteria:\n  1. `cargo check` and `cargo clippy` clean\n  2. `cargo test` green including edge cases\n  3. `///` doc comments and working `cargo run`\n",
                    ),
                    None,
                    None,
                );
                set_quiz(
                    &mut b[3],
                    "How should progress on an independent RustLab project be measured?",
                    &[
                        "By real verified capabilities in code (compilation, green tests, clippy quality, running), not XP alone.",
                        "Just opening and closing the menu ten times.",
                        "Ignoring compiler errors as long as enough text was written.",
                    ],
                    "Real milestones require code that builds cleanly, passes `cargo test`, and executes properly.",
                );
            }
        }
        "projeto-independente-qualidade-e-entrega" => {
            lesson.title = "Independent Project: Tests, Clippy, and Delivery".to_string();
            if b.len() >= 3 {
                set_text(
                    &mut b[0],
                    "As you build your own project in Create Mode, every real check (`F4` Build / `F7` Check, `Ctrl+L` Clippy, `F6` Test, `F5` Run) alongside `PLANO.md` and `#[test]` suites unlocks **real engineering milestones** on your Progress screen.",
                );
                set_callout(
                    &mut b[1],
                    "Remember the professional Rust workflow: `F8` (`cargo fmt`) for formatting, `Ctrl+L`/`Alt+C` (`cargo clippy`) for best practices, `F1`/`F11` for docs/symbols, and `Ctrl+T` whenever you want a second opinion from the AI Mentor.",
                );
                set_quiz(
                    &mut b[2],
                    "Which verification workflow ensures a solid Rust project release?",
                    &[
                        "Format (`cargo fmt`), lint (`cargo clippy`), test (`cargo test`), and execute (`cargo run`).",
                        "Delete `Cargo.toml` to build faster.",
                        "Avoid writing any `#[test]` functions.",
                    ],
                    "Formatting, linting, automated tests, and real execution form the full quality loop in Rust.",
                );
            }
        }
        _ => {}
    }
}

fn translate_exercise_to_en(id: &str, ex: &mut crate::curriculum::Exercise) {
    match id {
        "dobro" => {
            ex.title = "Compute double".to_string();
            ex.statement = "The `*` operator multiplies values. Implement `dobro(n)` returning twice `n`, without printing. The last expression in a function without a semicolon is returned automatically. Tests use small `i32` values (no overflow).".to_string();
            ex.skeleton =
                "pub fn dobro(n: i32) -> i32 {\n    todo!(\"return twice n\")\n}\n".to_string();
            ex.hints = vec![
                "Multiplying by 2 is the same as adding the number to itself.".to_string(),
                "Use parameter `n` as one operand to `*`.".to_string(),
                "`n * 2` already has type `i32`. Do not put `;` at the end.".to_string(),
                "Solution: `pub fn dobro(n: i32) -> i32 { n * 2 }`".to_string(),
            ];
        }
        "soma" => {
            ex.title = "Add two numbers".to_string();
            ex.statement = "Implement `soma(a, b)` to return the sum of two `i32` values. Do not print: return the result. `cargo test` checks positive, zero, and negative inputs.".to_string();
            ex.skeleton =
                "pub fn soma(a: i32, b: i32) -> i32 {\n    todo!(\"implement addition\")\n}\n"
                    .to_string();
            ex.hints = vec![
                "The `+` operator produces a new value; the function needs to return it."
                    .to_string(),
                "The body can be a single tail expression: `a + b`.".to_string(),
                "Replace `todo!(...)` with `a + b`.".to_string(),
                "Full implementation: `pub fn soma(a: i32, b: i32) -> i32 { a + b }`.".to_string(),
            ];
        }
        "media" => {
            ex.title = "Average without losing the fraction".to_string();
            ex.statement = "Implement `media(a, b)` returning the average of two `f64` values. Integer division would drop the decimal part; here inputs are floats. Add first before dividing by `2.0`: parentheses control precedence.".to_string();
            ex.skeleton =
                "pub fn media(a: f64, b: f64) -> f64 {\n    todo!(\"return the average\")\n}\n"
                    .to_string();
            ex.hints = vec![
                "Arithmetic average: sum of the two values divided by two.".to_string(),
                "`a + b / 2.0` divides only `b`, because `/` binds tighter than `+`.".to_string(),
                "Group the addition: `(a + b)`, then divide by `2.0`.".to_string(),
                "Solution: `pub fn media(a: f64, b: f64) -> f64 { (a + b) / 2.0 }`".to_string(),
            ];
        }
        "paridade" => {
            ex.title = "Check if even".to_string();
            ex.statement = "Implement `eh_par(n)`: return `true` when the integer is divisible by 2 and `false` otherwise. Tests cover negatives and zero as well.".to_string();
            ex.skeleton =
                "pub fn eh_par(n: i32) -> bool {\n    todo!(\"check divisibility by 2\")\n}\n"
                    .to_string();
            ex.hints = vec![
                "Integer remainder is computed with `%`.".to_string(),
                "A number is divisible by 2 when `n % 2` equals zero.".to_string(),
                "The boolean expression `n % 2 == 0` already yields `bool`.".to_string(),
                "Return `n % 2 == 0`.".to_string(),
            ];
        }
        "intervalo" => {
            ex.title = "Inside the range".to_string();
            ex.statement = "Implement `no_intervalo(n, minimo, maximo)` returning `bool`. `n` must be >= `minimo` AND <= `maximo` (inclusive bounds). `>=` and `<=` compare values; `&&` requires both conditions to hold. If `minimo > maximo`, return `false`.".to_string();
            ex.skeleton = "pub fn no_intervalo(n: i32, minimo: i32, maximo: i32) -> bool {\n    todo!(\"combine both bounds\")\n}\n".to_string();
            ex.hints = vec![
                "You need to verify both bounds, not only one side.".to_string(),
                "The two conditions are `n >= minimo` and `n <= maximo`.".to_string(),
                "Combine them with logical AND (`&&`), not OR (`||`).".to_string(),
                "Solution: `pub fn no_intervalo(n: i32, minimo: i32, maximo: i32) -> bool { n >= minimo && n <= maximo }`".to_string(),
            ];
        }
        "maior" => {
            ex.title = "Find the maximum".to_string();
            ex.statement = "Implement `maior(a, b)` returning the larger of two integers. If both are equal, return that common value.".to_string();
            ex.skeleton =
                "pub fn maior(a: i32, b: i32) -> i32 {\n    todo!(\"compare both values\")\n}\n"
                    .to_string();
            ex.hints = vec![
                "Compare the values with `>`.".to_string(),
                "An `if` expression can pick which value to return.".to_string(),
                "If `a > b`, return `a`; otherwise return `b`.".to_string(),
                "Use `if a > b { a } else { b }` as the final expression.".to_string(),
            ];
        }
        "contar-vogais" => {
            ex.title = "Count vowels".to_string();
            ex.statement = "Implement `contar_vogais(texto: &str) -> usize` returning how many ASCII vowels (`a, e, i, o, u`, upper or lower case) appear in `texto`. Iterate `texto.chars()` and normalize with `.to_ascii_lowercase()`.".to_string();
            ex.skeleton = "pub fn contar_vogais(texto: &str) -> usize {\n    todo!(\"count ASCII vowels in texto\")\n}\n".to_string();
            ex.hints = vec![
                "Use `for ch in texto.chars()` to inspect each UTF-8 character.".to_string(),
                "Lowercase it with `ch.to_ascii_lowercase()` before matching.".to_string(),
                "Check `matches!(c, 'a' | 'e' | 'i' | 'o' | 'u')` and increment a `mut` counter.".to_string(),
                "Solution: `texto.chars().filter(|c| matches!(c.to_ascii_lowercase(), 'a' | 'e' | 'i' | 'o' | 'u')).count()`".to_string(),
            ];
        }
        "filtrar-positivos" => {
            ex.title = "Filter positives".to_string();
            ex.statement = "Implement `filtrar_positivos(nums: &[i32]) -> Vec<i32>` returning a new `Vec` keeping only values strictly greater than zero in their original order. Zero and negative values must be skipped.".to_string();
            ex.skeleton = "pub fn filtrar_positivos(nums: &[i32]) -> Vec<i32> {\n    todo!(\"return only values > 0\")\n}\n".to_string();
            ex.hints = vec![
                "Create `let mut out = Vec::new();` or chain iterators over `nums`.".to_string(),
                "Iterate `for &n in nums` and check `if n > 0` (zero is excluded!).".to_string(),
                "Push each positive value via `out.push(n);`.".to_string(),
                "Solution: `nums.iter().copied().filter(|&n| n > 0).collect()`".to_string(),
            ];
        }
        "primeira-palavra" => {
            ex.title = "First word".to_string();
            ex.statement = "Implement `primeira_palavra(frase: &str) -> &str` returning a borrowed slice (`&str`) containing the first whitespace-delimited word (ignoring leading spaces). If `frase` is empty or whitespace-only, return `\"\"`.".to_string();
            ex.skeleton = "pub fn primeira_palavra(frase: &str) -> &str {\n    todo!(\"return the first word without allocating String\")\n}\n".to_string();
            ex.hints = vec![
                "No `String` allocation needed: `&str` methods return `&str` slices directly."
                    .to_string(),
                "`.trim_start()` skips leading spaces; `.split_whitespace()` splits words."
                    .to_string(),
                "`frase.split_whitespace().next()` returns `Option<&str>` with the first word."
                    .to_string(),
                "Solution: `frase.split_whitespace().next().unwrap_or(\"\")`".to_string(),
            ];
        }
        "retangulo-area" => {
            ex.title = "Methods on struct Retangulo".to_string();
            ex.statement = "Implement methods `area(&self) -> u32` and `eh_quadrado(&self) -> bool` on public struct `Retangulo` (`largura: u32`, `altura: u32`). `area` multiplies width by height; `eh_quadrado` returns `true` only when `largura > 0 && largura == altura`.".to_string();
            ex.skeleton = "pub struct Retangulo {\n    pub largura: u32,\n    pub altura: u32,\n}\n\nimpl Retangulo {\n    pub fn area(&self) -> u32 {\n        todo!(\"largura * altura\")\n    }\n\n    pub fn eh_quadrado(&self) -> bool {\n        todo!(\"check positive and equal sides\")\n    }\n}\n".to_string();
            ex.hints = vec![
                "Access struct fields inside `impl` via `self.largura` and `self.altura`.".to_string(),
                "In `area(&self)`, return `self.largura * self.altura`.".to_string(),
                "In `eh_quadrado(&self)`, verify `self.largura > 0 && self.largura == self.altura`.".to_string(),
                "Solution: `area` returns `self.largura * self.altura` and `eh_quadrado` returns `self.largura > 0 && self.largura == self.altura`.".to_string(),
            ];
        }
        "divisao-segura" => {
            ex.title = "Safe division with Option".to_string();
            ex.statement = "Implement `divisao_segura(a: i32, b: i32) -> Option<i32>`. When `b == 0`, return `None` instead of dividing by zero; otherwise return `Some(a / b)` (integer division).".to_string();
            ex.skeleton = "pub fn divisao_segura(a: i32, b: i32) -> Option<i32> {\n    todo!(\"avoid dividing by zero using Option\")\n}\n".to_string();
            ex.hints = vec![
                "Test `b == 0` first with `if b == 0`.".to_string(),
                "When `b == 0`, explicitly return `None`.".to_string(),
                "Otherwise wrap `a / b` in `Some(...)` (or call `a.checked_div(b)`).".to_string(),
                "Solution: `if b == 0 { None } else { Some(a / b) }`".to_string(),
            ];
        }
        "soma-quadrados-pares" => {
            ex.title = "Sum of even squares".to_string();
            ex.statement = "Implement `soma_quadrados_pares(nums: &[i32]) -> i32` computing the sum of squares of even numbers only in `nums`. Example: `&[1, 2, 3, 4]` -> `2² + 4² = 20`.".to_string();
            ex.skeleton = "pub fn soma_quadrados_pares(nums: &[i32]) -> i32 {\n    todo!(\"filter evens, square, and sum\")\n}\n".to_string();
            ex.hints = vec![
                "Use `for &n in nums` or iterator chaining via `.iter().copied()`.".to_string(),
                "An integer is even when `n % 2 == 0` (also works for negative evens and zero)."
                    .to_string(),
                "Square each value with `.map(|n| n * n)` and total with `.sum()`.".to_string(),
                "Solution: `nums.iter().copied().filter(|n| n % 2 == 0).map(|n| n * n).sum()`"
                    .to_string(),
            ];
        }
        _ => {}
    }
}

/// Returns `(title, learning_goal, completion_evidence)` in English (`en-US`) for any of the 213 V3 blueprint IDs.
pub fn v3_entry_en(id: &str) -> Option<(&'static str, &'static str, &'static str)> {
    Some(match id {
        // Basic (14)
        "B-T01" => (
            "What is programming?",
            "Understand instructions, execution order, and real-world software examples.",
            "Explain input -> processing -> output in your own words and answer the quiz.",
        ),
        "B-T02" => (
            "How computers run instructions",
            "Distinguish source code, the compiler, and an executable binary.",
            "Connect source, compilation, and execution in the visual demo and quiz.",
        ),
        "B-T03" => (
            "Terminal without fear",
            "Recognize commands, folders, file paths, and keyboard focus.",
            "Identify the project folder and compiler output panel.",
        ),
        "B-T04" => (
            "RustLab, Rust, and Cargo",
            "Understand the roles of rustc, rustup, and Cargo across the ecosystem.",
            "Locate run, build, and help controls without running arbitrary commands.",
        ),
        "B-T05" => (
            "Files and a Rust project",
            "Recognize Cargo.toml, src/main.rs, and target/.",
            "Identify the main source file and describe the Cargo manifest.",
        ),
        "B-P01" => (
            "My first Hello, world!",
            "Edit and execute a real main() function in Rust.",
            "Real `cargo run` prints your own custom message.",
        ),
        "B-T06" => (
            "Reading a program line by line",
            "Recognize fn main, braces, expressions, and semicolons.",
            "Identify each syntax element in an example program.",
        ),
        "B-T07" => (
            "Values and variables with let",
            "Understand `let` bindings and reading stored values.",
            "Predict output from a program with simple variables.",
        ),
        "B-T08" => (
            "Primitive types made clear",
            "Recognize integers, floats, booleans, char, and text.",
            "Classify examples with matching value and type.",
        ),
        "B-T09" => (
            "Immutability, mut, and const",
            "Distinguish default immutable bindings, mutation, and constants.",
            "Predict which code change compiles cleanly.",
        ),
        "B-T10" => (
            "println! and interpolation",
            "Understand console printing and value formatting.",
            "Explain expected stdout for a small snippet.",
        ),
        "B-T11" => (
            "Normal beginner errors",
            "Recognize missing punctuation, type mismatches, and rustc diagnostics.",
            "Spot where to look inside a `rustc` error message.",
        ),
        "B-T12" => (
            "How to experiment and study",
            "Use small edits, tests, careful error reading, and reflection.",
            "Describe a debugging loop and lesson workflow.",
        ),
        "B-P02" => (
            "My first variable",
            "Write your own variables in Rust and print a meaningful message.",
            "Custom values printed via real `cargo run`.",
        ),

        // Middle M01-M12 (48)
        "M01-T01" => (
            "Numeric types and inference",
            "Identify signed/unsigned integers, floats, and type inference.",
            "Predict inferred types and avoid hidden conversions.",
        ),
        "M01-T02" => (
            "Operators and precedence",
            "Read arithmetic, comparison, and boolean logic expressions.",
            "Evaluate expressions safely and accurately.",
        ),
        "M01-T03" => (
            "Scope and shadowing",
            "Distinguish block scope, reassignment, and shadowing.",
            "Explain variable lifetime across nested blocks.",
        ),
        "M01-P01" => (
            "Profile card",
            "Combine numeric types, arithmetic, and formatting in a small program.",
            "Tested output checks derived score and formatting.",
        ),
        "M02-T01" => (
            "if/else expressions",
            "Write conditional branches that return values.",
            "Predict which branch runs for each boundary.",
        ),
        "M02-T02" => (
            "Reading user text input",
            "Use `stdin` alongside guided `Result` handling.",
            "Trace the input flow and why reading input can fail.",
        ),
        "M02-T03" => (
            "Parsing user input safely",
            "Convert strings to numbers while handling malformed input.",
            "Separate valid numeric input from parse failures.",
        ),
        "M02-P01" => (
            "Interactive choice calculator",
            "Build a calculator handling operations and branch decisions.",
            "Operations, edge cases, and division by zero pass tests.",
        ),
        "M03-T01" => (
            "loop, while, and for",
            "Choose the right loop construct for each repetition task.",
            "Explain termination conditions and bounds.",
        ),
        "M03-T02" => (
            "Ranges and break/continue",
            "Use inclusive/exclusive ranges and loop flow controls.",
            "Predict exact iteration counts.",
        ),
        "M03-T03" => (
            "match and basic patterns",
            "Handle multiple alternatives exhaustively with `match`.",
            "Recognize exhaustiveness across simple patterns.",
        ),
        "M03-P01" => (
            "Guess the number",
            "Create a CLI guessing loop powered by loops and `match`.",
            "Multiple guesses, invalid input, and edge ranges tested.",
        ),
        "M04-T01" => (
            "Functions and signatures",
            "Understand parameter lists, return types, and local scope.",
            "Read function signatures and call sites clearly.",
        ),
        "M04-T02" => (
            "Return expressions",
            "Understand semicolon-free tail expressions and early returns.",
            "Predict what value a function returns.",
        ),
        "M04-T03" => (
            "Separating responsibilities",
            "Break repeated tasks into small focused functions.",
            "Explain clean boundaries between responsibilities.",
        ),
        "M04-P01" => (
            "Unit converter",
            "Implement multiple conversions across cohesive functions.",
            "Tests cover conversions, bounds, and edge inputs.",
        ),
        "M05-T01" => (
            "Moved values and ownership",
            "Understand moving ownership from one binding to another.",
            "Explain which variable owns the value after a move.",
        ),
        "M05-T02" => (
            "Copy and Clone",
            "Distinguish implicit primitive `Copy` from explicit `Clone`.",
            "Contrast stack-copied values with heap-owned allocations.",
        ),
        "M05-T03" => (
            "Scope and intro to Drop",
            "Understand lexical lifetimes and deterministic cleanup.",
            "Predict exactly when a resource is dropped.",
        ),
        "M05-P01" => (
            "Ownership transfer challenge",
            "Fix ownership errors cleanly without blind `.clone()` calls.",
            "Former E0382 code compiles and passes unit tests.",
        ),
        "M06-T01" => (
            "Shared references",
            "Read borrowed values via immutable references `&T`.",
            "Explain read access without moving ownership.",
        ),
        "M06-T02" => (
            "Mutable references",
            "Understand exclusive access guarantees of `&mut T`.",
            "Identify overlapping borrow conflicts.",
        ),
        "M06-T03" => (
            "Slices and lifetime intuition",
            "Use slice views and reason about reference validity.",
            "Predict slice bounds and borrow duration.",
        ),
        "M06-P01" => (
            "Borrowed inventory",
            "Implement borrowed readers alongside scoped mutations.",
            "Unit tests verify return values and borrow semantics.",
        ),
        "M07-T01" => (
            "String, &str, and bytes",
            "Contrast owned `String` buffers against borrowed `&str` slices.",
            "Identify UTF-8 byte vs character implications.",
        ),
        "M07-T02" => (
            "Chars, slices, and Unicode",
            "See why indexing raw UTF-8 byte offsets can panic.",
            "Choose safe iteration via `.chars()`.",
        ),
        "M07-T03" => (
            "Text formatting and conversion",
            "Format and transform text idiomatically.",
            "Explain handling whitespace or malformed text.",
        ),
        "M07-P01" => (
            "Message analyzer",
            "Inspect and format accented UTF-8 messages cleanly.",
            "Unicode inputs handled without byte-boundary panics.",
        ),
        "M08-T01" => (
            "Arrays, slices, and Vec",
            "Choose between fixed-size arrays and growable `Vec`.",
            "Contrast checked `.get()` indexing against panicking `[]`.",
        ),
        "M08-T02" => (
            "HashMap and sets",
            "Understand key-value maps and uniqueness sets.",
            "Predict lookup, insert, and update behavior.",
        ),
        "M08-T03" => (
            "Iteration by value vs reference",
            "Pick `into_iter` vs `iter` vs `iter_mut` purposefully.",
            "Explain moved collections vs borrowed iteration.",
        ),
        "M08-P01" => (
            "Shopping list",
            "Build dynamic slice/vector logic backed by collections.",
            "Add, inspect, bounds, and duplicates pass unit tests.",
        ),
        "M09-T01" => (
            "Structs and fields",
            "Group related state into domain types.",
            "Pick appropriate structs to represent data cleanly.",
        ),
        "M09-T02" => (
            "Methods and impl blocks",
            "Attach behavior directly to data types.",
            "Contrast `&self`, `&mut self`, and `self` receivers.",
        ),
        "M09-T03" => (
            "Enums and associated patterns",
            "Model mutually exclusive domain states cleanly.",
            "Cover every variant exhaustively in `match`.",
        ),
        "M09-P01" => (
            "Contacts catalog",
            "Combine structs and enums for application state.",
            "Tests cover creation, lookups, updates, and invalid cases.",
        ),
        "M10-T01" => (
            "Option and absent values",
            "Represent legitimate absence safely without null pointers.",
            "Explain `Some` vs `None` handling.",
        ),
        "M10-T02" => (
            "Result and the ? operator",
            "Handle and propagate recoverable errors explicitly.",
            "Trace `Ok`/`Err` flow across a function.",
        ),
        "M10-T03" => (
            "No panics for expected errors",
            "Prefer explicit error handling over `.unwrap()`.",
            "Distinguish valid bug assertions from expected failures.",
        ),
        "M10-P01" => (
            "Signup validator",
            "Validate user input returning descriptive `Result`/`Option`.",
            "Missing, invalid, and valid cases verified by tests.",
        ),
        "M11-T01" => (
            "Iterator, map, and filter",
            "Transform data pipelines lazily with iterator adapters.",
            "Reason about lazy evaluation and consumers.",
        ),
        "M11-T02" => (
            "Closures and basic captures",
            "Pass behavior cleanly as inline closures.",
            "Identify how closures capture outer variables.",
        ),
        "M11-T03" => (
            "Patterns and if let",
            "Simplify single-case matching with `if let` / `let else`.",
            "Choose the clearest pattern-matching construct.",
        ),
        "M11-P01" => (
            "Score report",
            "Compute summaries using iterators and helper functions.",
            "Tests cover empty lists, ties, and aggregations.",
        ),
        "M12-T01" => (
            "Modules and basic visibility",
            "Split a program cleanly into multiple modules/files.",
            "Understand `pub` visibility and module paths.",
        ),
        "M12-T02" => (
            "Intro to unit tests",
            "Write `#[test]` functions and meaningful assertions.",
            "Explain why a test passes or catches a regression.",
        ),
        "M12-T03" => (
            "Cargo, crates, and docs",
            "Read `Cargo.toml` manifests and look up docs quickly.",
            "Identify compatible crate versions and dependencies.",
        ),
        "M12-P01" => (
            "Task manager",
            "Build and verify modular multi-step Rust code.",
            "Structured modules, tests, and docs pass verification.",
        ),

        // Hard / Deep Dive D-REV + D01-D12 (61)
        "D-REV-001" => (
            "Independent review — your first integrated program",
            "Combine types, collections, ownership, input, and tests without starter spoilers.",
            "Behavior tests and self-explanation pass cleanly.",
        ),
        "D01-T01" => (
            "Non-lexical lifetimes (NLL)",
            "Understand NLL and when borrows actually end.",
            "Explain the exact point each borrow ends.",
        ),
        "D01-T02" => (
            "Lifetimes in functions",
            "Read and write explicit lifetime annotations `'a`.",
            "Connect output reference lifetimes to input borrows.",
        ),
        "D01-T03" => (
            "Iteration, moves, and ownership",
            "Contrast consuming vs non-consuming iterator APIs.",
            "Predict moves and borrows across loops.",
        ),
        "D01-P01" => (
            "Borrow checker workshop",
            "Resolve real borrow conflicts without unnecessary `.clone()`.",
            "Formerly rejected signatures compile and pass tests.",
        ),
        "D01-P02" => (
            "Zero-copy refactoring",
            "Separate read borrows from mutations across collections.",
            "Behavior preserved with clean ownership flow.",
        ),
        "D02-T01" => (
            "Data-carrying enums and patterns",
            "Model rich application states inside enum variants.",
            "Write exhaustive state transitions.",
        ),
        "D02-T02" => (
            "Derive traits and conversions",
            "Use `Debug`, `Display`, `From`, and `TryFrom` effectively.",
            "Describe fallible conversions with `TryFrom`.",
        ),
        "D02-T03" => (
            "Foundational generic types",
            "Design parameterized containers and generic functions.",
            "Write and read clean generic signatures.",
        ),
        "D02-P01" => (
            "State machine",
            "Implement explicit compile-checked state transitions.",
            "Invalid transitions rejected cleanly in tests.",
        ),
        "D02-P02" => (
            "Typed parser",
            "Parse raw text into strongly typed domain structs/enums.",
            "Valid and malformed inputs both tested.",
        ),
        "D03-T01" => (
            "Trait bounds and impl Trait",
            "Constrain generic behavior with trait bounds.",
            "Explain monomorphized static dispatch.",
        ),
        "D03-T02" => (
            "Trait objects and dyn",
            "Use dynamic dispatch for heterogeneous collections.",
            "Compare `dyn Trait` trade-offs against generics.",
        ),
        "D03-T03" => (
            "Associated types and defaults",
            "Design cleaner traits using associated types and default methods.",
            "Pick expressive trait signatures.",
        ),
        "D03-P01" => (
            "Shapes and summaries trait lab",
            "Implement shared contracts across types via traits.",
            "Multiple trait implementations pass tests.",
        ),
        "D03-P02" => (
            "Extensible command system",
            "Dispatch commands cleanly with generics or trait objects.",
            "Commands, errors, and unit tests pass.",
        ),
        "D04-T01" => (
            "Box and heap allocation",
            "Build recursive data structures using `Box<T>`.",
            "Trace heap ownership layout accurately.",
        ),
        "D04-T02" => (
            "Rc/Arc and shared ownership",
            "Apply reference counting only when shared ownership fits.",
            "Avoid overuse and spot valid use cases.",
        ),
        "D04-T03" => (
            "RefCell, Weak, and Drop",
            "Understand interior mutability and reference-cycle prevention.",
            "Explain runtime borrow panics and weak cycles.",
        ),
        "D04-P01" => (
            "Element tree / pure test suite",
            "Implement tested pure helpers and recursive logic cleanly.",
            "Traversals and edge cases pass tests.",
        ),
        "D04-P02" => (
            "Shared cache",
            "Combine `Rc`/`Arc`/`RefCell` responsibly.",
            "Correctness and cycle-free cleanup tested.",
        ),
        "D05-T01" => (
            "Domain error enums",
            "Design informative domain error enums.",
            "Distinguish recoverable domain errors from bugs/panics.",
        ),
        "D05-T02" => (
            "Robust propagation across layers",
            "Compose multi-layer errors cleanly via `Result`.",
            "Trace failure propagation across module boundaries.",
        ),
        "D05-T03" => (
            "Logs, diagnostics, and Clippy",
            "Read Clippy lints and emit actionable diagnostics.",
            "Reproduce and identify root cause of bugs.",
        ),
        "D05-P01" => (
            "Resilient CLI",
            "Handle missing files and invalid arguments without panicking.",
            "Failure paths verified by tests.",
        ),
        "D05-P02" => (
            "Documented bug hunt",
            "Investigate and fix an intentionally buggy starter codebase.",
            "Regression tests pass with documented root cause.",
        ),
        "D06-T01" => (
            "Workspace layout",
            "Separate library and binary crates in a Cargo workspace.",
            "Identify crate dependency boundaries.",
        ),
        "D06-T02" => (
            "Integration tests and docs",
            "Test public APIs cleanly including `///` doctests.",
            "Test suite and doctests run green.",
        ),
        "D06-T03" => (
            "Cargo features and SemVer",
            "Understand feature flags, SemVer, build profiles, and `Cargo.lock`.",
            "Explain dependency compatibility and reproducible builds.",
        ),
        "D06-P01" => (
            "Library-backed workspace",
            "Build clean iterator/library logic verified by `cargo test`.",
            "Unit and workspace tests pass.",
        ),
        "D06-P02" => (
            "Reproducible packaging",
            "Pin dependencies and document build/release steps.",
            "Verified clean release check.",
        ),
        "D07-T01" => (
            "Threads and join handles",
            "Spawn and synchronize worker OS threads.",
            "Reason about join guarantees and ordering.",
        ),
        "D07-T02" => (
            "Channels and message passing",
            "Send owned messages safely across threads.",
            "Design reliable producer/consumer communication.",
        ),
        "D07-T03" => (
            "Scoped threads",
            "Use `std::thread::scope` to borrow stack data safely.",
            "Contrast scoped threads with `move` closures.",
        ),
        "D07-P01" => (
            "Parallel processor",
            "Write pure boundary/physics logic ready for safe parallel evaluation.",
            "Deterministic checks match serial baseline.",
        ),
        "D07-P02" => (
            "Channel pipeline",
            "Build clean deterministic state transformations across boundaries.",
            "Boundary and physics tests pass without races.",
        ),
        "D08-T01" => (
            "Mutex, Arc, and shared state",
            "Protect shared mutable data with mutual exclusion.",
            "Explain lock poisoning and keeping lock scopes short.",
        ),
        "D08-T02" => (
            "RwLock and contention",
            "Choose read-write locks for read-heavy workloads.",
            "Concurrent multi-reader access reasoned cleanly.",
        ),
        "D08-T03" => (
            "Send, Sync, and data races",
            "Understand Rust's marker auto-traits preventing data races.",
            "Catch unsafe thread-sharing attempts at compile time.",
        ),
        "D08-P01" => (
            "Reliable shared counter",
            "Implement deterministic state checks free of race assumptions.",
            "Collision/state checks pass reliably.",
        ),
        "D08-P02" => (
            "Bounded worker queue",
            "Manage clean bounds and graceful shutdown invariants.",
            "Boundary collisions and clean termination pass tests.",
        ),
        "D09-T01" => (
            "stdin, stdout, and stderr",
            "Separate standard I/O streams for interactive CLIs.",
            "Route output vs errors appropriately.",
        ),
        "D09-T02" => (
            "Keyboard events and PTY",
            "Understand terminal raw mode and polling input events.",
            "Drive a simple keyboard input state loop.",
        ),
        "D09-T03" => (
            "UI state and incremental redraw",
            "Manage screens, redraws, and clean terminal teardown.",
            "Design safe terminal cleanup on exit/panic.",
        ),
        "D09-P01" => (
            "Navigable terminal menu",
            "Read stdin commands interactively and exit cleanly.",
            "Interactive menu responds to options `2` and `0` cleanly.",
        ),
        "D09-P02" => (
            "File viewer",
            "Browse and display files safely in the terminal.",
            "Unicode, missing files, and errors tested cleanly.",
        ),
        "D10-T01" => (
            "Coordinates and layout",
            "Model rows, columns, and game viewport bounds.",
            "Predict on-screen element coordinates.",
        ),
        "D10-T02" => (
            "Update loop and fixed tick",
            "Run deterministic game ticks independent of keypress rate.",
            "Reason about frame delta and tick timing.",
        ),
        "D10-T03" => (
            "Input, collision, and separation",
            "Separate physical game simulation from terminal rendering.",
            "Read input state safely without stalling.",
        ),
        "D10-P01" => (
            "Move the paddle up and down",
            "Update positions and filter active state cleanly.",
            "Boundary/filter unit tests pass cleanly.",
        ),
        "D10-P02" => (
            "Complete Pong",
            "Implement ball wall-bounce physics and deterministic state tests.",
            "Bounce/boundary tests verify physics.",
        ),
        "D11-T01" => (
            "Grid, direction, and rules",
            "Model a game grid and prevent 180-degree self-reversal.",
            "Explain safe direction updates.",
        ),
        "D11-T02" => (
            "Seeded randomness",
            "Use deterministic seeded PRNGs for testable game spawns.",
            "Reproducible food spawning verified.",
        ),
        "D11-T03" => (
            "Growth, collisions, and game state",
            "Update snake segments and detect head-to-body collisions.",
            "Predict collision cases accurately.",
        ),
        "D11-P01" => (
            "Playable basic Snake",
            "Implement core state tracking and movement logic.",
            "Grid tests pass cleanly.",
        ),
        "D11-P02" => (
            "Polished full Snake",
            "Add self-collision detection and clean boundary checks.",
            "Edge and body-collision tests pass.",
        ),
        "D12-T01" => (
            "Buffer and cursor",
            "Design UTF-8-safe text buffers and cursor navigation.",
            "Prevent slicing mid-codepoint on Unicode text.",
        ),
        "D12-T02" => (
            "Saving, opening, and data safety",
            "Handle file I/O errors and atomic save workflows.",
            "Track dirty/unsaved buffer changes cleanly.",
        ),
        "D12-T03" => (
            "Undo/redo and UI events",
            "Model reversible editing commands in history stacks.",
            "Describe undo/redo command invariants.",
        ),
        "D12-P01" => (
            "Mini editor — editing",
            "Handle insertion, deletion, and line boundaries cleanly.",
            "Unicode and boundary tests pass.",
        ),
        "D12-P02" => (
            "Mini editor — complete app",
            "Summarize line counts and byte lengths accurately.",
            "Line/byte summary and edge cases pass tests.",
        ),

        // End Game F01-F10 (60)
        "F01-T01" => (
            "What async solves",
            "Contrast parallelism, concurrency, and blocking I/O.",
            "Compare async tasks against OS threads.",
        ),
        "F01-T02" => (
            "async/await and Future",
            "Understand `Future`, `Poll::Pending`, and waking.",
            "Trace execution of an `async` call.",
        ),
        "F01-T03" => (
            "Executors and Tokio",
            "Understand runtime task scheduling and async I/O.",
            "Spot executor-blocking pitfalls.",
        ),
        "F01-T04" => (
            "Cancellation, streams, and backpressure",
            "Use `select!`, timeouts, and bounded channels.",
            "Design graceful task cancellation.",
        ),
        "F01-P01" => (
            "Async client with timeout",
            "Format clean HTTP/1.1 request lines and validate inputs.",
            "Request formatting tests pass.",
        ),
        "F01-P02" => (
            "Cancelable async worker",
            "Implement clean bounded request formatting and lifecycle control.",
            "Request and timeout behavior verified.",
        ),
        "F02-T01" => (
            "TCP and application protocols",
            "Read TCP byte streams and frame messages cleanly.",
            "Explain framing boundaries and retries.",
        ),
        "F02-T02" => (
            "HTTP and JSON",
            "Understand methods, status codes, headers, and serialization.",
            "Catch malformed payloads and client errors.",
        ),
        "F02-T03" => (
            "Resilient API client",
            "Consume external APIs validating status, JSON, and timeouts.",
            "Explain backoff and strict validation.",
        ),
        "F02-T04" => (
            "Web servers and routing",
            "Structure route handlers, shared state, and tests.",
            "Trace the full request/response lifecycle.",
        ),
        "F02-P01" => (
            "Documented HTTP client",
            "Format, validate, and deserialize API data cleanly.",
            "Mock/payload tests pass.",
        ),
        "F02-P02" => (
            "Minimal web service",
            "Expose simple routes returning accurate error statuses.",
            "Integration checks cover invalid requests.",
        ),
        "F03-T01" => (
            "Advanced generics and monomorphization",
            "Read expressive trait bounds and understand code size tradeoffs.",
            "Compare static vs dynamic dispatch tradeoffs.",
        ),
        "F03-T02" => (
            "Associated types, GATs, and HRTBs",
            "Recognize advanced type and lifetime constraints.",
            "Read advanced generic library APIs.",
        ),
        "F03-T03" => (
            "Pin, Unpin, and self-references",
            "Understand memory pinning required by self-referential futures.",
            "Explain why moving self-references violates invariants.",
        ),
        "F03-T04" => (
            "Trait objects and API patterns",
            "Apply `dyn`, newtypes, supertraits, and dyn-compatibility.",
            "Choose ergonomic trait designs.",
        ),
        "F03-P01" => (
            "Generic reusable library",
            "Design clean strongly-typed generic APIs.",
            "Generic bounds and behavior tests pass.",
        ),
        "F03-P02" => (
            "Dyn and lifetime abstraction",
            "Build extensible trait-based component systems.",
            "Tests cover polymorphism and lifetimes.",
        ),
        "F04-T01" => (
            "Declarative macros",
            "Learn token-tree matching with `macro_rules!`.",
            "Distinguish macro expansion time from runtime.",
        ),
        "F04-T02" => (
            "Hygiene and expansion",
            "Understand macro identifier hygiene and scope isolation.",
            "Catch variable capture pitfalls inside macros.",
        ),
        "F04-T03" => (
            "Derive and procedural macros",
            "Understand proc-macros operating on `TokenStream`.",
            "Inspect code generated by custom derives.",
        ),
        "F04-T04" => (
            "When NOT to write macros",
            "Prefer functions, generics, and traits before reaching for macros.",
            "Justify when a macro truly pays for its complexity.",
        ),
        "F04-P01" => (
            "Small config/validation parser",
            "Parse key=value CLI entries while trimming and validating input.",
            "Parser handles edge cases, quotes, and missing keys.",
        ),
        "F04-P02" => (
            "Educational minimal derive",
            "Implement clean delimited key/value parsing helpers.",
            "Parser fixtures pass reliably.",
        ),
        "F05-T01" => (
            "Rules and invariants of unsafe",
            "Learn what `unsafe` allows and what safety rules NEVER relax.",
            "Explain caller vs implementer soundness obligations.",
        ),
        "F05-T02" => (
            "Raw pointers and layout",
            "Understand `*const`/`*mut`, `#[repr]`, and alignment.",
            "Spot invalid pointer dereferences.",
        ),
        "F05-T03" => (
            "FFI and the C ABI",
            "Interact safely across a minimal C-compatible ABI boundary.",
            "Explain ownership transfer across FFI boundaries.",
        ),
        "F05-T04" => (
            "Encapsulating unsafe",
            "Build safe wrappers backed by `// SAFETY:` invariants.",
            "Recognize Undefined Behavior risks.",
        ),
        "F05-P01" => (
            "Safe FFI wrapper",
            "Encapsulate a well-reviewed minimal boundary safely.",
            "Safety invariants and clean build verified.",
        ),
        "F05-P02" => (
            "Unsafe audit",
            "Fix broken invariants in an unsafe snippet.",
            "Tests pass cleanly without UB.",
        ),
        "F06-T01" => (
            "Git and collaboration",
            "Practice focused commits, clean branches, and code reviews.",
            "Explain clean commit history.",
        ),
        "F06-T02" => (
            "Advanced Cargo workflows",
            "Master workspaces, feature unification, profiles, and lockfiles.",
            "Configure release and target settings cleanly.",
        ),
        "F06-T03" => (
            "CI, Clippy, fmt, and supply chain",
            "Automate checks and audit crate dependencies.",
            "Design a trustworthy CI pipeline.",
        ),
        "F06-T04" => (
            "Publishing and SemVer",
            "Understand `docs.rs`, `crates.io`, and Semantic Versioning.",
            "Explain breaking vs compatible API changes.",
        ),
        "F06-P01" => (
            "Publishable library crate",
            "Write a documented, tested crate ready for packaging.",
            "`cargo package --no-verify` / tests pass cleanly.",
        ),
        "F06-P02" => (
            "Cross-platform CI pipeline",
            "Verify Linux and Windows workflows cleanly.",
            "Honest verification status documented.",
        ),
        "F07-T01" => (
            "Complexity and real cost",
            "Estimate algorithmic complexity before micro-optimizing.",
            "Spot accidental quadratic loops.",
        ),
        "F07-T02" => (
            "Allocations and efficient ownership",
            "Measure unnecessary `.clone()`, `Vec`, and `String` allocations.",
            "Explain measured memory behavior.",
        ),
        "F07-T03" => (
            "Benchmarks and profiling",
            "Measure real baselines and hotspots instead of guessing.",
            "Interpret benchmark variance properly.",
        ),
        "F07-T04" => (
            "Data-oriented design",
            "Apply cache locality and batch processing.",
            "Justify optimizations with measurements.",
        ),
        "F07-P01" => (
            "Evidence-backed optimization",
            "Benchmark a slow implementation and optimize without losing correctness.",
            "Baseline and reproducible improvement verified.",
        ),
        "F07-P02" => (
            "Load testing and regressions",
            "Add bounds and guardrails against performance regressions.",
            "All speed claims backed by measurements.",
        ),
        "F08-T01" => (
            "Serde and structured files",
            "Serialize versioned structs cleanly to JSON/TOML.",
            "Round-trip fixtures pass cleanly.",
        ),
        "F08-T02" => (
            "SQLite and transactions",
            "Use schemas, parameterized queries, and atomic transactions.",
            "Prevent injection and malformed input issues.",
        ),
        "F08-T03" => (
            "Credentials, privileges, and paths",
            "Protect secrets, permissions, and guard against path traversal.",
            "Spot unsafe filesystem path joins.",
        ),
        "F08-T04" => (
            "Migrations and recovery",
            "Preserve backward compatibility and recover from corruption.",
            "Old schemas and corrupt inputs tested cleanly.",
        ),
        "F08-P01" => (
            "SQLite-backed app",
            "Persist, update, and query structured records.",
            "Transaction and migration tests pass.",
        ),
        "F08-P02" => (
            "Safe file importer",
            "Validate untrusted structured inputs strictly.",
            "Malformed payloads and path traversal rejected.",
        ),
        "F09-T01" => (
            "Event loop and UI state",
            "Understand native GUI window event loops.",
            "Trace state updates per frame/event.",
        ),
        "F09-T02" => (
            "Logical canvas and viewport",
            "Decouple logical artwork dimensions from window resolution.",
            "Explain why window resizing preserves pixels.",
        ),
        "F09-T03" => (
            "Mouse input, tools, and rendering",
            "Map pointer events into logical canvas drawing commands.",
            "Test coordinate translation cleanly.",
        ),
        "F09-T04" => (
            "Responsive layouts and export",
            "Reflow panels on resize and export native canvas pixels.",
            "Artwork data remains intact after resizing.",
        ),
        "F09-P01" => (
            "Native GUI canvas window",
            "Open a native GUI window alongside clean `CanvasSize` validation.",
            "Window behavior and size constraints verified.",
        ),
        "F09-P02" => (
            "Drawing with resize",
            "Keep a persistent logical buffer under a responsive viewport.",
            "Automated coordinate/bounds tests pass cleanly.",
        ),
        "F10-T01" => (
            "Larger projects and modules",
            "Define clean modules and clear boundaries in large codebases.",
            "Evaluate module coupling cleanly.",
        ),
        "F10-T02" => (
            "Robust and property testing",
            "Combine unit, property-based, and integration tests.",
            "Catch edge-case counterexamples cleanly.",
        ),
        "F10-T03" => (
            "Observability, errors, and recovery",
            "Design actionable error reports and graceful recovery.",
            "Reproduce defects directly from logs.",
        ),
        "F10-T04" => (
            "Packaging and maintenance",
            "Package, document, and sustain software releases.",
            "Complete a clean release checklist.",
        ),
        "F10-P01" => (
            "Real-world refactoring",
            "Improve architecture without breaking existing behavior.",
            "Regression tests remain green with clear rationale.",
        ),
        "F10-P02" => (
            "Verifiable release delivery",
            "Prepare Linux/Windows release builds honestly.",
            "Real verified artifacts or explicit status documented.",
        ),

        // Mini Paint Capstone (12)
        "PAINT-01" => (
            "Define scope and structure",
            "Plan application state, tools, canvas storage, and UI panels.",
            "Architecture checklist and valid `CanvasSize` checks pass.",
        ),
        "PAINT-02" => (
            "Native window and event loop",
            "Launch a responsive native window handling close and resize.",
            "Window checks and canvas size bounds pass.",
        ),
        "PAINT-03" => (
            "Persistent logical canvas",
            "Model canvas dimensions/pixels independent of screen resolution.",
            "Canvas model unit tests pass cleanly.",
        ),
        "PAINT-04" => (
            "Coordinate system",
            "Map mouse pixels cleanly into logical canvas buffer indices.",
            "Out-of-bounds and pixel-index mapping tests pass.",
        ),
        "PAINT-05" => (
            "Brush and continuous strokes",
            "Draw smooth continuous strokes across fast mouse movements.",
            "Continuous pixel indexing tests pass without gaps.",
        ),
        "PAINT-06" => (
            "Palette, colors, and tools",
            "Support color selection, brush, and eraser safely on buffer bounds.",
            "Bounds and tool transitions pass tests.",
        ),
        "PAINT-07" => (
            "Responsive UI & interpolation",
            "Compute maximum delta step count for gap-free brush lines across resizes.",
            "Line-step interpolation and resize invariants pass.",
        ),
        "PAINT-08" => (
            "Undo, redo, and safe clear",
            "Implement reversible undo/redo history and safe canvas clearing.",
            "Line/history tests pass cleanly.",
        ),
        "PAINT-09" => (
            "Save and reopen projects",
            "Persist editable artwork files cleanly in a documented format.",
            "Stroke/persistence invariants verified.",
        ),
        "PAINT-10" => (
            "Export image (PPM / PNG)",
            "Generate a valid P6 PPM header preserving exact logical canvas dimensions.",
            "Exported header and byte layout verified by tests.",
        ),
        "PAINT-11" => (
            "Package for Linux and Windows",
            "Produce reproducible builds and document OS dependencies.",
            "Valid export format and release build verified.",
        ),
        "PAINT-12" => (
            "Independent final tool",
            "Add your own custom drawing tool, fix edge cases, and document.",
            "Final checklist and export/tool tests pass.",
        ),

        // CaveGame Capstone (18)
        "CAVE-01" => (
            "Set up Bevy and first window",
            "Prepare a tested Bevy project and map world coordinates to chunks.",
            "Chunk coordinate mapping and initial build pass.",
        ),
        "CAVE-02" => (
            "ECS — entities and components",
            "Model game entities, chunk grids, and ECS components with tests.",
            "Coordinate and ECS unit tests pass.",
        ),
        "CAVE-03" => (
            "Resources, systems, and schedule",
            "Use Bevy Resources, schedules, and game states cleanly.",
            "State transition and chunk index tests pass.",
        ),
        "CAVE-04" => (
            "3D camera and coordinates",
            "Configure 3D projection, camera orientation, and chunk coordinates.",
            "Negative-coordinate chunk math verified.",
        ),
        "CAVE-05" => (
            "The first 3D block & culling",
            "Render lit cubes and decide which voxel block faces are visible.",
            "Hidden-face culling logic passes unit tests.",
        ),
        "CAVE-06" => (
            "First-person controls",
            "Implement WASD + mouse look with cursor capture alongside voxel checks.",
            "Input and face-visibility tests pass.",
        ),
        "CAVE-07" => (
            "Jump, gravity, and collision",
            "Add gravity, jumping, and block collision detection.",
            "Deterministic visibility/physics tests pass.",
        ),
        "CAVE-08" => (
            "Voxel world representation",
            "Model block types, air, and 24x24 chunk worlds efficiently.",
            "Air vs solid block face culling passes tests.",
        ),
        "CAVE-09" => (
            "Block raycast interaction",
            "Implement crosshair raycasting to place and break blocks.",
            "Block placement/culling rules pass unit tests.",
        ),
        "CAVE-10" => (
            "Chunk organization",
            "Generate deterministic world height across seeded chunks.",
            "Chunk height and boundary tests pass.",
        ),
        "CAVE-11" => (
            "Procedural seeded terrain",
            "Generate reproducible voxel terrain heights from `(x, z, seed)`.",
            "Same seed produces identical terrain; range stays valid.",
        ),
        "CAVE-12" => (
            "Meshes and visible faces",
            "Strip hidden internal faces and build chunk meshes efficiently.",
            "Height/mesh data tests pass cleanly.",
        ),
        "CAVE-13" => (
            "Chunk streaming and lifecycle",
            "Load and unload chunks dynamically as the player moves.",
            "Deterministic chunk height/streaming passes tests.",
        ),
        "CAVE-14" => (
            "Creative inventory & hotbar",
            "Add block palette, hotbar selection, crosshair, and HUD.",
            "Selection and terrain invariants verified.",
        ),
        "CAVE-15" => (
            "Save, load, and version worlds",
            "Persist world seed and player edits with versioned format.",
            "Seeded round-trip determinism verified.",
        ),
        "CAVE-16" => (
            "Optimization and profiling",
            "Measure frame times, memory, and chunk rebuild costs.",
            "Determinism and baseline checks pass.",
        ),
        "CAVE-17" => (
            "Polish and distribution",
            "Add settings, pause/help overlays, and release packaging.",
            "Smoke tests and packaging checklist pass.",
        ),
        "CAVE-18" => (
            "Independent final challenge",
            "Implement your own original gameplay feature inside the voxel world.",
            "Feature tests and manual playtest verified.",
        ),
        _ => return None,
    })
}

/// Translates any `PracticeSpec` into English (`en-US`) while keeping all `tests`-checked identifiers intact.
pub fn translate_practice_spec_to_en(spec: &mut crate::learn::PracticeSpec) {
    if let Some((title, goal, _)) = v3_entry_en(spec.lesson_id)
        && spec.id != "B-P01"
        && spec.id != "B-P02"
        && spec.id != "D09-P01"
    {
        spec.title = title;
        spec.prompt = goal;
    }

    match spec.id {
        "B-T01-P" => {
            spec.title = "Practice B-T01-P · Precise steps in Rust";
            spec.prompt = "Replace `\"...\"` inside `println!` with a sentence describing input, processing, and output, then run with `F5`.";
            spec.starter = "fn main() {\n    // Example: Input (name) -> Processing (greeting) -> Output\n    println!(\"...\");\n}\n";
            spec.hints = [
                "Level 1: Programs transform inputs into outputs step by step.",
                "Level 2: Replace the ellipsis inside `println!(\"...\");` with a real sentence.",
                "Level 3: Keep both double quotes and the trailing `;` intact.",
                "Level 4: Write at least 8 characters describing your flow and press `F5`.",
            ];
        }
        "B-T02-P" => {
            spec.title = "Practice B-T02-P · From source to executable";
            spec.prompt = "Print a line mentioning how the `compiler` produces an `executable`, then run with `F5`.";
            spec.starter = "fn main() {\n    println!(\"...\");\n}\n";
            spec.hints = [
                "Level 1: Remember the stages: source code, compiler, executable.",
                "Level 2: Use `println!` to print a sentence containing `compiler` or `executable`.",
                "Level 3: Do not leave `...` inside the quotes.",
                "Level 4: Print a sentence explaining what the compiler does and run `F5`.",
            ];
        }
        "B-T04-P" => {
            spec.title = "Practice B-T04-P · rustc and Cargo tooling";
            spec.prompt =
                "Print a `println!` sentence mentioning `cargo` or `rustc`, then run with `F5`.";
            spec.starter = "fn main() {\n    println!(\"...\");\n}\n";
            spec.hints = [
                "Level 1: Rust's compiler is `rustc`, and its build/package tool is `cargo`.",
                "Level 2: Put a short sentence inside `println!` containing `cargo` or `rustc`.",
                "Level 3: Check that quotes and `;` are balanced.",
                "Level 4: Replace `...` with a sentence mentioning Cargo and press `F5`.",
            ];
        }
        "B-P01" => {
            spec.title = "Practice B-P01 · First println! with Cargo";
            spec.prompt =
                "Replace `\"...\"` with your own message in `println!`, save, and run with `F5`.";
            spec.starter = "fn main() {\n    // Write your message below and press F5 to compile and run\n    println!(\"...\");\n}\n";
            spec.hints = [
                "Level 1 (Direction): `println!(\"...\");` prints a full line to standard output.",
                "Level 2 (Structure): Keep `fn main() { ... }` and edit only the text inside `println!`.",
                "Level 3 (Pitfall): Verify both double quotes and the semicolon `;` at the end of the line.",
                "Level 4 (Outline): Put a real greeting different from `...` inside `println!`, then press `Ctrl+S` and `F5`.",
            ];
        }
        "B-T06-P" => {
            spec.title = "Practice B-T06-P · Two statements in main";
            spec.prompt = "Add a second `println!` call inside `fn main()` so it prints at least two lines (`F5`).";
            spec.starter = "fn main() {\n    println!(\"line 1\");\n}\n";
            spec.hints = [
                "Level 1: Each sequential statement inside `fn main()` ends with `;`.",
                "Level 2: Add another `println!(\"...\");` line right below the first one.",
                "Level 3: Make sure the second call stays before the closing `}` brace.",
                "Level 4: Print two non-empty lines when running `F5`.",
            ];
        }
        "B-P02" => {
            spec.title = "Practice B-P02 · Variables and mutability with Cargo";
            spec.prompt = "Update the mutable variable `pontos` to a non-zero value before `println!`, then run with `F5`.";
            spec.starter = "fn main() {\n    let curso = \"RustLab\";\n    let mut pontos = 0;\n    // Update `pontos` before printing\n    println!(\"{curso}: {pontos}\");\n}\n";
            spec.hints = [
                "Level 1 (Direction): In Rust, `let mut` opts a variable into reassignment.",
                "Level 2 (Structure): Add a line before `println!` adding to or assigning `pontos`.",
                "Level 3 (Pitfall): Do not shadow with `let pontos = ...` without updating if practicing mutation.",
                "Level 4 (Outline): Between the binding and `println!`, change `pontos` away from `0` and press `F5`.",
            ];
        }
        "B-T09-P" => {
            spec.title = "Practice B-T09-P · Controlled mutability (cargo test)";
            spec.prompt = "Implement `incrementar(mut valor: i32, passo: i32) -> i32` by adding `passo` into `valor` and returning the total (`F5`).";
            spec.starter = "pub fn incrementar(mut valor: i32, passo: i32) -> i32 {\n    let _ = (&mut valor, passo);\n    todo!(\"add passo to valor and return\")\n}\n";
            spec.hints = [
                "Level 1: Parameters marked `mut` can be modified locally inside the function.",
                "Level 2: Add `passo` to `valor` with `valor += passo;` and return `valor`.",
                "Level 3: Omit `;` on the final tail return expression.",
                "Level 4: Remove `todo!` and return `valor` after adding `passo`.",
            ];
        }
        "M01-T01-P" => {
            spec.title = "Practice M01-T01-P · f64 floating-point";
            spec.prompt = "Implement `dobro_decimal(x: f64) -> f64` returning twice `x` (`F5`).";
            spec.starter = "pub fn dobro_decimal(x: f64) -> f64 {\n    let _ = x;\n    todo!(\"multiply x by 2.0\")\n}\n";
            spec.hints = [
                "Level 1: Rust never silently coerces between `i32` and `f64`.",
                "Level 2: Use float literal `2.0` when operating with `f64`.",
                "Level 3: Avoid integer `2`; write `x * 2.0` without `;`.",
                "Level 4: One tail expression returning `x * 2.0` solves it cleanly.",
            ];
        }
        "M01-P01" => {
            spec.title = "Practice M01-P01 · Operator precedence";
            spec.prompt = "Implement `cartao_pontos(base: i32, bonus: i32) -> i32` computing `(base + bonus) * 2` (`F5`).";
            spec.starter = "pub fn cartao_pontos(base: i32, bonus: i32) -> i32 {\n    let _ = (base, bonus);\n    todo!(\"return (base + bonus) * 2\")\n}\n";
            spec.hints = [
                "Level 1: Multiplication has higher precedence than addition.",
                "Level 2: Explicit parentheses force addition to happen first.",
                "Level 3: Without parentheses, `base + bonus * 2` yields the wrong answer.",
                "Level 4: Write `(base + bonus) * 2` as your return expression.",
            ];
        }
        "M02-T01-P" => {
            spec.title = "Practice M02-T01-P · if/else expressions";
            spec.prompt = "Implement `classificar_temperatura(graus: i32) -> &'static str`: `< 15 => \"frio\"`, `15..=25 => \"ameno\"`, `> 25 => \"quente\"` (`F5`).";
            spec.starter = "pub fn classificar_temperatura(graus: i32) -> &'static str {\n    let _ = graus;\n    todo!(\"return \\\"frio\\\", \\\"ameno\\\", or \\\"quente\\\"\")\n}\n";
            spec.hints = [
                "Level 1: `if / else if / else` in Rust is an expression returning a value.",
                "Level 2: Check `< 15` first, then `<= 25`, leaving `else` for `\"quente\"`.",
                "Level 3: All branches must evaluate to `&'static str`.",
                "Level 4: Check boundary values 15 and 25 carefully so both yield `\"ameno\"`.",
            ];
        }
        "M07-P01" => {
            spec.title = "Practice M07-P01 · UTF-8 Strings & slices";
            spec.prompt = "Implement `saudar_usuario(nome: &str) -> String`: `.trim()` spaces; if empty return `\"Olá, visitante!\"`, else `\"Olá, <nome>!\"` (`F5`).";
            spec.starter = "pub fn saudar_usuario(nome: &str) -> String {\n    let _ = nome;\n    todo!(\"handle empty name and format greeting\")\n}\n";
            spec.hints = [
                "Level 1: Call `.trim()` on `&str` to strip whitespace without allocating.",
                "Level 2: Check `clean.is_empty()` before formatting.",
                "Level 3: Return an owned `String` (`format!(...)` or `.to_string()`).",
                "Level 4: Return `\"Olá, visitante!\".to_string()` when empty or `format!(\"Olá, {clean}!\")`.",
            ];
        }
        "M08-P01" => {
            spec.title = "Practice M08-P01 · Slice to tuple (min, max)";
            spec.prompt = "Implement `extremos_array(valores: &[i32]) -> Option<(i32, i32)>` returning `None` when empty or `Some((min, max))` (`F5`).";
            spec.starter = "pub fn extremos_array(valores: &[i32]) -> Option<(i32, i32)> {\n    let _ = valores;\n    todo!(\"return Some((min, max)) or None\")\n}\n";
            spec.hints = [
                "Level 1: Never index `valores[0]` without confirming the slice is non-empty.",
                "Level 2: Use `let (&first, rest) = valores.split_first()?;` or check `is_empty()`.",
                "Level 3: Update `min = min.min(v)` and `max = max.max(v)` in the loop.",
                "Level 4: Wrap `(min, max)` inside `Some((min, max))`.",
            ];
        }
        "M09-P01" => {
            spec.title = "Practice M09-P01 · Selective iteration sum";
            spec.prompt = "Implement `somar_positivos(lista: &[i32]) -> i32` summing only numbers strictly greater than zero (`F5`).";
            spec.starter = "pub fn somar_positivos(lista: &[i32]) -> i32 {\n    let _ = lista;\n    todo!(\"filter > 0 and sum\")\n}\n";
            spec.hints = [
                "Level 1: Iterate via `lista.iter().copied()` or `for &n in lista`.",
                "Level 2: Include only `n > 0`; zero and negatives are excluded.",
                "Level 3: Iterator form: `lista.iter().copied().filter(|&n| n > 0).sum()`.",
                "Level 4: Empty or all-non-positive slices return `0`.",
            ];
        }
        "M05-P01" => {
            spec.title = "Practice M05-P01 · Moving and returning String";
            spec.prompt = "Implement `anexar_sufixo(mut texto: String, sufixo: &str) -> String` by appending `sufixo` and returning ownership of `texto` (`F5`).";
            spec.starter = "pub fn anexar_sufixo(mut texto: String, sufixo: &str) -> String {\n    let _ = (&mut texto, sufixo);\n    todo!(\"append sufixo to texto and return texto\")\n}\n";
            spec.hints = [
                "Level 1: The function owns `texto` and borrows `sufixo`.",
                "Level 2: `texto.push_str(sufixo);` mutates in place without cloning.",
                "Level 3: Return `texto` without `;` to transfer ownership back.",
                "Level 4: Replace `todo!` after `texto.push_str(sufixo);` with `texto`.",
            ];
        }
        "M06-P01" => {
            spec.title = "Practice M06-P01 · Borrowed Unicode character count";
            spec.prompt = "Implement `contar_caracteres(texto: &str) -> usize` counting Unicode scalar chars (not bytes!) via `&str` (`F5`).";
            spec.starter = "pub fn contar_caracteres(texto: &str) -> usize {\n    let _ = texto;\n    todo!(\"count Unicode chars in texto\")\n}\n";
            spec.hints = [
                "Level 1: `texto.len()` returns UTF-8 byte length, not character count.",
                "Level 2: `.chars()` yields each Unicode scalar character of `&str`.",
                "Level 3: Chain `.count()` onto the char iterator.",
                "Level 4: Return `texto.chars().count()` without taking ownership.",
            ];
        }
        "D01-P01" => {
            spec.title = "Practice D01-P01 · Lifetime annotation 'a";
            spec.prompt = "Implement `maior_fatia<'a>(a: &'a str, b: &'a str) -> &'a str` returning the longer byte slice (on a tie, return `a`) (`F5`).";
            spec.starter = "pub fn maior_fatia<'a>(a: &'a str, b: &'a str) -> &'a str {\n    let _ = (a, b);\n    todo!(\"compare a.len() and b.len()\")\n}\n";
            spec.hints = [
                "Level 1: Lifetime `'a` ties the returned borrow to the shorter of `a` and `b`.",
                "Level 2: Use `if a.len() >= b.len() { a } else { b }`.",
                "Level 3: Note `>=` guarantees returning `a` on ties.",
                "Level 4: No `String` allocation needed — return `a` or `b` directly.",
            ];
        }
        "M10-P01" => {
            spec.title = "Practice M10-P01 · Struct Contador & methods";
            spec.prompt = "Implement on `Contador`: `novo(inicio: i32) -> Self` and `avancar(&mut self) -> i32` which adds 1 and returns the updated value (`F5`).";
            spec.starter = "pub struct Contador {\n    atual: i32,\n}\n\nimpl Contador {\n    pub fn novo(inicio: i32) -> Self {\n        let _ = inicio;\n        todo!(\"create Contador with atual = inicio\")\n    }\n    pub fn avancar(&mut self) -> i32 {\n        todo!(\"increment self.atual and return\")\n    }\n}\n";
            spec.hints = [
                "Level 1: Constructors return `Self { field }` and omit `&self`.",
                "Level 2: Mutating methods take `&mut self`.",
                "Level 3: Inside `avancar`, write `self.atual += 1; self.atual`.",
                "Level 4: Ensure both methods are marked `pub`.",
            ];
        }
        "M11-P01" => {
            spec.title = "Practice M11-P01 · Enum & exhaustive match";
            spec.prompt = "Implement `executar(cmd: Comando) -> i32` handling `Somar(a, b)`, `Negar(x)`, and `Zero` using `match` (`F5`).";
            spec.starter = "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub enum Comando {\n    Somar(i32, i32),\n    Negar(i32),\n    Zero,\n}\n\npub fn executar(cmd: Comando) -> i32 {\n    let _ = cmd;\n    todo!(\"match on Comando\")\n}\n";
            spec.hints = [
                "Level 1: Rust `match` checks that every enum variant is covered.",
                "Level 2: Destructure `Comando::Somar(a, b) => a + b`.",
                "Level 3: Return `-x` for `Comando::Negar(x)` and `0` for `Comando::Zero`.",
                "Level 4: Name all three variants explicitly without needing `_`.",
            ];
        }
        "D03-P01" => {
            spec.title = "Practice D03-P01 · Trait Resumo";
            spec.prompt =
                "Implement `Resumo` for `Artigo` returning `\"<titulo> — <autor>\"` (`F5`).";
            spec.starter = "pub trait Resumo {\n    fn resumir(&self) -> String;\n}\n\npub struct Artigo {\n    pub titulo: String,\n    pub autor: String,\n}\n\nimpl Resumo for Artigo {\n    fn resumir(&self) -> String {\n        todo!(\"format titulo and autor separated by ' — '\")\n    }\n}\n";
            spec.hints = [
                "Level 1: `impl Trait for Struct` fulfills the trait contract.",
                "Level 2: Use `self.titulo` and `self.autor` inside `format!`.",
                "Level 3: Pay attention to the spaces around the em-dash ` — `.",
                "Level 4: Write `format!(\"{} — {}\", self.titulo, self.autor)`.",
            ];
        }
        "M12-T01-P" => {
            spec.title = "Practice M12-T01-P · Safe absence with Option";
            spec.prompt = "Implement `primeiro_par(nums: &[i32]) -> Option<i32>` returning the first even number or `None` (`F5`).";
            spec.starter = "pub fn primeiro_par(nums: &[i32]) -> Option<i32> {\n    let _ = nums;\n    todo!(\"find the first even number\")\n}\n";
            spec.hints = [
                "Level 1: Rust replaces bare nulls with `Option<T>` (`Some(v)` / `None`).",
                "Level 2: An integer is even when `n % 2 == 0`.",
                "Level 3: Use `nums.iter().copied().find(|n| n % 2 == 0)`.",
                "Level 4: Return the result of `.find(...)` directly.",
            ];
        }
        "M12-P01" => {
            spec.title = "Practice M12-P01 · Typed errors with Result";
            spec.prompt = "Implement `parse_positivo(texto: &str) -> Result<i32, String>`: reject non-numeric strings or values `<= 0` with `Err(...)` (`F5`).";
            spec.starter = "pub fn parse_positivo(texto: &str) -> Result<i32, String> {\n    let _ = texto;\n    todo!(\"parse integer and ensure > 0\")\n}\n";
            spec.hints = [
                "Level 1: Call `.trim().parse::<i32>()`, map error via `.map_err(...)`, and propagate with `?`.",
                "Level 2: After parsing succeeds, verify `n > 0`.",
                "Level 3: Zero and negative integers must return `Err(String)`.",
                "Level 4: When `n > 0`, return `Ok(n)`.",
            ];
        }
        "D04-P01" => {
            spec.title = "Practice D04-P01 · Pure testable function (palindrome)";
            spec.prompt = "Implement `eh_palindromo(s: &str) -> bool` ignoring spaces and ASCII letter case (`F5`).";
            spec.starter = "pub fn eh_palindromo(s: &str) -> bool {\n    let _ = s;\n    todo!(\"check if s is a palindrome\")\n}\n";
            spec.hints = [
                "Level 1: Filter out spaces and lowercase each char into a `Vec<char>`.",
                "Level 2: Compare the normalized vector forward vs reversed.",
                "Level 3: `c.to_ascii_lowercase()` normalizes ASCII letters cleanly.",
                "Level 4: Empty or whitespace-only strings are palindromes (`true`).",
            ];
        }
        "D05-P01" => {
            spec.title = "Practice D05-P01 · Generics & Fn closures";
            spec.prompt = "Implement `aplicar_duas_vezes<F: Fn(i32) -> i32>(x: i32, f: F) -> i32` applying `f` twice to `x` (`F5`).";
            spec.starter = "pub fn aplicar_duas_vezes<F: Fn(i32) -> i32>(x: i32, f: F) -> i32 {\n    let _ = (x, f);\n    todo!(\"apply f to f(x)\")\n}\n";
            spec.hints = [
                "Level 1: A generic parameter bounded by `Fn(i32) -> i32` is callable directly.",
                "Level 2: Compute the first step `let first = f(x);`.",
                "Level 3: Pass `first` back into `f`.",
                "Level 4: The solution reduces cleanly to `f(f(x))`.",
            ];
        }
        "D06-P01" => {
            spec.title = "Practice D06-P01 · Iterator pipeline";
            spec.prompt = "Implement `quadrados_impares(limite: i32) -> Vec<i32>` returning squares of odd numbers across `1..=limite` (`F5`).";
            spec.starter = "pub fn quadrados_impares(limite: i32) -> Vec<i32> {\n    let _ = limite;\n    todo!(\"return squares of odd numbers in 1..=limite\")\n}\n";
            spec.hints = [
                "Level 1: When `limite < 1`, range `1..=limite` is naturally empty.",
                "Level 2: Filter odds via `.filter(|n| n % 2 != 0)` over `1..=limite`.",
                "Level 3: Square each item with `.map(|n| n * n)`.",
                "Level 4: End with `.collect()` to gather into `Vec<i32>`.",
            ];
        }
        "F04-P01" => {
            spec.title = "Practice F04-P01 · CLI key=value parser";
            spec.prompt = "Implement `parse_chave_valor(linha: &str) -> Option<(&str, &str)>` splitting at the first `=`, trimming whitespace, and rejecting empty keys (`F5`).";
            spec.starter = "pub fn parse_chave_valor(linha: &str) -> Option<(&str, &str)> {\n    let _ = linha;\n    todo!(\"split linha at = and trim whitespace\")\n}\n";
            spec.hints = [
                "Level 1: Call `.split_once('=')` to split strictly on the first `=`.",
                "Level 2: Call `.trim()` on both key and value.",
                "Level 3: Return `None` if the trimmed key is empty.",
                "Level 4: Values containing additional `=` stay preserved with `split_once`.",
            ];
        }
        "F01-P01" => {
            spec.title = "Practice F01-P01 · Network HTTP request line";
            spec.prompt = "Implement `formatar_requisicao(metodo: &str, rota: &str) -> String` producing `\"<UPPER_METHOD> <route> HTTP/1.1\"` (`F5`).";
            spec.starter = "pub fn formatar_requisicao(metodo: &str, rota: &str) -> String {\n    let _ = (metodo, rota);\n    todo!(\"format HTTP/1.1 request line\")\n}\n";
            spec.hints = [
                "Level 1: Trim surrounding whitespace via `.trim()` first.",
                "Level 2: Uppercase the HTTP method with `.to_uppercase()`.",
                "Level 3: Format `format!(\"{method} {route} HTTP/1.1\")`.",
                "Level 4: Keep the exact suffix `HTTP/1.1`.",
            ];
        }
        "M02-P01" => {
            spec.title = "Practice M02-P01 · Safe calculator core";
            spec.prompt = "Implement `calcular(a: i32, op: char, b: i32) -> Option<i32>` for `+`, `-`, `*`, `/`, returning `None` on division by 0 or unknown operators (`F5`).";
            spec.starter = "pub fn calcular(a: i32, op: char, b: i32) -> Option<i32> {\n    let _ = (a, op, b);\n    todo!(\"handle +, -, *, and / without panicking\")\n}\n";
            spec.hints = [
                "Level 1: Use `match op` to handle the four operators.",
                "Level 2: Use `a.checked_div(b)` or guard `if b == 0 { return None; }` on division.",
                "Level 3: Unknown operators should fall into `_ => None`.",
                "Level 4: `checked_add`/`checked_sub`/`checked_mul` protect against integer overflow too.",
            ];
        }
        "D10-P01" => {
            spec.title = "Practice D10-P01 · Filter completed tasks";
            spec.prompt = "Implement `filtrar_concluidas(tarefas: &[(&str, bool)]) -> Vec<String>` returning only titles marked `true` (`F5`).";
            spec.starter = "pub fn filtrar_concluidas(tarefas: &[(&str, bool)]) -> Vec<String> {\n    let _ = tarefas;\n    todo!(\"return titles of completed tasks\")\n}\n";
            spec.hints = [
                "Level 1: Each slice element is a tuple `(title, completed)`.",
                "Level 2: Keep items where the boolean flag is `true`.",
                "Level 3: Convert each `&str` title with `.to_string()`.",
                "Level 4: Preserve original task ordering.",
            ];
        }
        "PAINT-01-P" => {
            spec.title = "Practice PAINT-01-P · Mini Paint logical size";
            spec.prompt = "Implement `CanvasSize::new(width: u16, height: u16) -> Option<Self>` accepting only dimensions in `1..=2048` (`F5`).";
            spec.starter = "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct CanvasSize {\n    pub width: u16,\n    pub height: u16,\n}\n\nimpl CanvasSize {\n    pub fn new(width: u16, height: u16) -> Option<Self> {\n        let _ = (width, height);\n        todo!(\"validate dimensions > 0 and <= 2048\")\n    }\n}\n";
            spec.hints = [
                "Level 1: A logical canvas never permits zero width/height or huge allocations.",
                "Level 2: Check `(1..=2048).contains(&width) && (1..=2048).contains(&height)`.",
                "Level 3: Return `None` when either dimension is out of range.",
                "Level 4: When valid, return `Some(Self { width, height })`.",
            ];
        }
        "CAVE-01-P" => {
            spec.title = "Practice CAVE-01-P · 16x16 chunk coordinates (CaveGame)";
            spec.prompt = "Implement `chunk_coords(x: i32, z: i32) -> (i32, i32)` mapping world blocks to `16x16` chunks (including negatives!) (`F5`).";
            spec.starter = "pub fn chunk_coords(x: i32, z: i32) -> (i32, i32) {\n    let _ = (x, z);\n    todo!(\"use Euclidean division by 16\")\n}\n";
            spec.hints = [
                "Level 1: Standard `/` integer division truncates toward zero near `-1` (`-1 / 16 == 0`).",
                "Level 2: In voxel engines, block `-1` belongs to chunk `-1`.",
                "Level 3: Rust has `i32::div_euclid(16)` built into `std`!",
                "Level 4: Return `(x.div_euclid(16), z.div_euclid(16))`.",
            ];
        }
        "D09-P01" => {
            spec.title = "Practice D09-P01 · Interactive terminal menu (stdin)";
            spec.prompt = "Read commands from `stdin` in a loop: `\"1\"` prints `Help`, `\"2\"` prints `Points: 10`, and `\"0\"` prints `Exiting` and breaks (`F5`).";
            spec.starter = "use std::io::{self, BufRead};\n\nfn main() {\n    for line in io::stdin().lock().lines() {\n        let op = line.unwrap_or_default();\n        match op.trim() {\n            // Complete branches \"1\", \"2\", and \"0\"\n            _ => break,\n        }\n    }\n}\n";
            spec.hints = [
                "Level 1: Match on `op.trim()` inside the loop over `io::stdin().lock().lines()`.",
                "Level 2: For `\"1\"`, print `println!(\"Help\");`; for `\"2\"`, print `println!(\"Points: 10\");`.",
                "Level 3: For `\"0\"`, print `println!(\"Exiting\");` and execute `break;`.",
                "Level 4: On `F5`, RustLab pipes simulated user lines via `stdin` and closes with EOF.",
            ];
        }
        // V3 custom specs
        "M03-P01" => {
            spec.starter = "pub fn somar_ate(n: i32) -> i32 {\n    let _ = n;\n    todo!(\"sum integers 1..=n, returning 0 if n <= 0\")\n}\n";
            spec.prompt = "Implement `somar_ate(n: i32) -> i32` returning the sum of `1..=n` (and `0` when `n <= 0`) (`F5`).";
            spec.hints = [
                "Level 1 (Direction): Handle `n <= 0` by returning `0` so the range never goes backwards.",
                "Level 2 (Structure): Loop `(1..=n)` accumulating into `let mut total = 0`.",
                "Level 3 (Pitfall): `1..=n` is inclusive of `n`, whereas `1..n` omits `n`.",
                "Level 4 (Study): `if n <= 0 { 0 } else { (1..=n).sum() }`.",
            ];
        }
        "M04-P01" => {
            spec.starter = "pub fn area_retangulo(largura: u32, altura: u32) -> u32 {\n    let _ = (largura, altura);\n    todo!(\"compute largura * altura as a tail expression\")\n}\n";
            spec.prompt = "Implement `area_retangulo(largura: u32, altura: u32) -> u32` using a semicolon-free return expression (`F5`).";
            spec.hints = [
                "Level 1 (Direction): In Rust, the final line without `;` is returned automatically.",
                "Level 2 (Structure): Multiply `largura * altura` to yield `u32`.",
                "Level 3 (Pitfall): Adding `;` at the end turns it into a `()` statement.",
                "Level 4 (Study): `largura.saturating_mul(altura)` also guards against overflow.",
            ];
        }
        "D-REV-001" => {
            spec.starter = "pub fn contar_palavras(texto: &str) -> usize {\n    let _ = texto;\n    todo!(\"count whitespace-separated words without allocating extra Strings\")\n}\n";
            spec.prompt = "Implement `contar_palavras(texto: &str) -> usize` counting whitespace-delimited words without allocations (`F5`).";
            spec.hints = [
                "Level 1 (Direction): `&str` provides `.split_whitespace()` in the standard library.",
                "Level 2 (Structure): Chain `.count()` on the word iterator.",
                "Level 3 (Pitfall): `split(' ')` produces empty segments on double spaces; prefer `split_whitespace()`.",
                "Level 4 (Study): Return `texto.split_whitespace().count()`.",
            ];
        }
        "D07-P01" | "D07-P02" | "D10-P02" => {
            spec.starter = "pub fn rebater_eixo(pos: i32, vel: i32, limite: i32) -> i32 {\n    let _ = (pos, vel, limite);\n    todo!(\"invert vel when pos <= 0 or pos >= limite\")\n}\n";
            spec.prompt = "Implement `rebater_eixo(pos: i32, vel: i32, limite: i32) -> i32`: negate `vel` when `pos <= 0 || pos >= limite` (`F5`).";
            spec.hints = [
                "Level 1 (Direction): In games like Pong, hitting wall bounds reverses velocity sign.",
                "Level 2 (Structure): Check `if pos <= 0 || pos >= limite { -vel } else { vel }`.",
                "Level 3 (Pitfall): Use `<= 0` so sprites at boundary `0` bounce cleanly.",
                "Level 4 (Study): `if pos <= 0 || pos >= limite { -vel } else { vel }`.",
            ];
        }
        "D08-P01" | "D08-P02" | "D11-P02" => {
            spec.starter = "pub fn colisao_corpo(cabeca: (i32, i32), corpo: &[(i32, i32)]) -> bool {\n    let _ = (cabeca, corpo);\n    todo!(\"return true if cabeca matches any segment in corpo\")\n}\n";
            spec.prompt = "Implement `colisao_corpo(cabeca: (i32, i32), corpo: &[(i32, i32)]) -> bool` checking whether `cabeca` overlaps any coordinate in `corpo` (`F5`).";
            spec.hints = [
                "Level 1 (Direction): In Snake, collision occurs when head coordinates match any body segment.",
                "Level 2 (Structure): Use `corpo.contains(&cabeca)` or `.iter().any(|&seg| seg == cabeca)`.",
                "Level 3 (Pitfall): Ensure `(i32, i32)` tuples compare by full coordinate equality.",
                "Level 4 (Study): `corpo.contains(&cabeca)` is concise and idiomatic.",
            ];
        }
        "D12-P02" => {
            spec.starter = "pub fn resumo_buffer(texto: &str) -> (usize, usize) {\n    let _ = texto;\n    todo!(\"return (lines, bytes) of the mini editor buffer\")\n}\n";
            spec.prompt = "Implement `resumo_buffer(texto: &str) -> (usize, usize)` returning `(line_count, byte_len)` for `texto` (`F5`).";
            spec.hints = [
                "Level 1 (Direction): `.lines().count()` counts logical lines regardless of trailing `\\n`.",
                "Level 2 (Structure): `.len()` returns exact UTF-8 byte count.",
                "Level 3 (Pitfall): On `\"\"`, `.lines().count()` yields `0`.",
                "Level 4 (Study): Return `(texto.lines().count(), texto.len())`.",
            ];
        }
        id if id.starts_with("PAINT-01")
            || id.starts_with("PAINT-02")
            || id.starts_with("PAINT-03") =>
        {
            spec.starter = "#[derive(Debug, Clone, Copy, PartialEq, Eq)]\npub struct CanvasSize {\n    pub width: u16,\n    pub height: u16,\n}\n\nimpl CanvasSize {\n    pub fn new(width: u16, height: u16) -> Option<Self> {\n        let _ = (width, height);\n        todo!(\"accept dimensions 1..=2048\")\n    }\n}\n";
            spec.prompt = "Implement `CanvasSize::new(width: u16, height: u16) -> Option<Self>` accepting dimensions `1..=2048` (`F5`).";
            spec.hints = [
                "Level 1 (Direction): A logical canvas never accepts 0 on either axis.",
                "Level 2 (Structure): Check `(1..=2048).contains(&width) && (1..=2048).contains(&height)`.",
                "Level 3 (Pitfall): Mini Paint's logical canvas size stays intact across window resizes.",
                "Level 4 (Study): Return `Some(Self { width, height })` if valid, otherwise `None`.",
            ];
        }
        id if id.starts_with("PAINT-04")
            || id.starts_with("PAINT-05")
            || id.starts_with("PAINT-06") =>
        {
            spec.starter = "pub fn indice_pixel(x: usize, y: usize, largura: usize, altura: usize) -> Option<usize> {\n    let _ = (x, y, largura, altura);\n    todo!(\"return Some(y * largura + x) strictly within bounds\")\n}\n";
            spec.prompt = "Implement `indice_pixel(x: usize, y: usize, largura: usize, altura: usize) -> Option<usize>` computing `Some(y * largura + x)` only when in bounds (`F5`).";
            spec.hints = [
                "Level 1 (Direction): Never index the pixel buffer without verifying `x < largura && y < altura`.",
                "Level 2 (Structure): Use explicit bounds checking or checked arithmetic before returning.",
                "Level 3 (Pitfall): Out-of-bounds clicks must return `None`, never panic.",
                "Level 4 (Study): `if x < largura && y < altura { Some(y * largura + x) } else { None }`.",
            ];
        }
        id if id.starts_with("PAINT-07")
            || id.starts_with("PAINT-08")
            || id.starts_with("PAINT-09") =>
        {
            spec.starter = "pub fn passos_linha(dx: i32, dy: i32) -> i32 {\n    let _ = (dx, dy);\n    todo!(\"return max absolute delta between dx and dy to interpolate stroke\")\n}\n";
            spec.prompt = "Implement `passos_linha(dx: i32, dy: i32) -> i32` returning the largest absolute movement across `dx` and `dy` (`F5`).";
            spec.hints = [
                "Level 1 (Direction): Fast cursor drag skips pixels; interpolation prevents stroke gaps.",
                "Level 2 (Structure): Compare `dx.abs()` and `dy.abs()` via `.max(...)`.",
                "Level 3 (Pitfall): Deltas can be negative when dragging up or left!",
                "Level 4 (Study): `dx.abs().max(dy.abs())`.",
            ];
        }
        id if id.starts_with("PAINT-") => {
            spec.starter = "pub fn cabecalho_ppm(largura: u16, altura: u16) -> String {\n    let _ = (largura, altura);\n    todo!(\"return P6 header `P6\\n<largura> <altura>\\n255\\n` for PPM export\")\n}\n";
            spec.prompt = "Implement `cabecalho_ppm(largura: u16, altura: u16) -> String` formatting `\"P6\\n{largura} {altura}\\n255\\n\"` (`F5`).";
            spec.hints = [
                "Level 1 (Direction): Binary P6 PPM images begin with the ASCII magic header `P6`.",
                "Level 2 (Structure): Format `format!(\"P6\\n{largura} {altura}\\n255\\n\")`.",
                "Level 3 (Pitfall): Include `255` and the trailing `\\n` before raw RGB pixel bytes.",
                "Level 4 (Study): `format!(\"P6\\n{largura} {altura}\\n255\\n\")`.",
            ];
        }
        id if id.starts_with("CAVE-01")
            || id.starts_with("CAVE-02")
            || id.starts_with("CAVE-03")
            || id.starts_with("CAVE-04") =>
        {
            spec.starter = "pub fn chunk_coords(x: i32, z: i32) -> (i32, i32) {\n    let _ = (x, z);\n    todo!(\"map world coords into 16x16 chunk indices with div_euclid\")\n}\n";
            spec.prompt = "Implement `chunk_coords(x: i32, z: i32) -> (i32, i32)` mapping world coords into `16x16` chunks using `div_euclid` (`F5`).";
            spec.hints = [
                "Level 1 (Direction): Standard `/` truncates negatives toward zero and breaks at `-1`.",
                "Level 2 (Structure): Apply `.div_euclid(16)` to both `x` and `z`.",
                "Level 3 (Pitfall): `(-1i32).div_euclid(16)` is `-1`, preserving 16 blocks per chunk.",
                "Level 4 (Study): `(x.div_euclid(16), z.div_euclid(16))`.",
            ];
        }
        id if id.starts_with("CAVE-05")
            || id.starts_with("CAVE-06")
            || id.starts_with("CAVE-07")
            || id.starts_with("CAVE-08")
            || id.starts_with("CAVE-09") =>
        {
            spec.starter = "pub fn face_visivel(bloco_atual: u8, vizinho: u8) -> bool {\n    let _ = (bloco_atual, vizinho);\n    todo!(\"return true iff bloco_atual != 0 (solid) and vizinho == 0 (air)\")\n}\n";
            spec.prompt = "Implement `face_visivel(bloco_atual: u8, vizinho: u8) -> bool` returning `true` only when `bloco_atual != 0 && vizinho == 0` (`F5`).";
            spec.hints = [
                "Level 1 (Direction): Face culling skips interior faces between two solid blocks.",
                "Level 2 (Structure): Treat `0` as Air and non-zero IDs as solid blocks.",
                "Level 3 (Pitfall): Air (`0`) never needs to emit exterior block faces.",
                "Level 4 (Study): `bloco_atual != 0 && vizinho == 0`.",
            ];
        }
        id if id.starts_with("CAVE-") => {
            spec.starter = "pub fn altura_terreno(x: i32, z: i32, seed: u64) -> i32 {\n    let _ = (x, z, seed);\n    todo!(\"generate deterministic height in 2..=12 from (x, z, seed)\")\n}\n";
            spec.prompt = "Implement `altura_terreno(x: i32, z: i32, seed: u64) -> i32` producing a deterministic height in `2..=12` (`F5`).";
            spec.hints = [
                "Level 1 (Direction): Reproducible voxel worlds generate identical terrain for a fixed seed.",
                "Level 2 (Structure): Combine `(x, z, seed)` deterministically without unseeded randomness.",
                "Level 3 (Pitfall): Clamp range to `2..=12` using Euclidean remainder `.rem_euclid(11)`.",
                "Level 4 (Study): `let h = (x as i64 * 31 + z as i64 * 17 + seed as i64).rem_euclid(11) as i32; 2 + h`.",
            ];
        }
        _ => {}
    }
}
