use alloc::string::String;
use alloc::vec::Vec;
use alloc::format;
use crate::{print, println};

const MAX_INPUT_LENGTH: usize = 256;

pub struct Shell {
    input_buffer: String,
    command_history: Vec<String>,
    history_index: usize,
}

impl Shell {
    pub fn new() -> Self {
        Shell {
            input_buffer: String::new(),
            command_history: Vec::new(),
            history_index: 0,
        }
    }

    pub fn handle_key(&mut self, c: char) {
        match c {
            '\n' => {
                println!();
                self.execute_command();
                self.input_buffer.clear();
                self.print_prompt();
            }
            '\x08' => {
                // Backspace
                if !self.input_buffer.is_empty() {
                    self.input_buffer.pop();
                    print!("\x08 \x08"); // Erase character on screen
                }
            }
            c if c.is_ascii_graphic() || c == ' ' => {
                if self.input_buffer.len() < MAX_INPUT_LENGTH {
                    self.input_buffer.push(c);
                    print!("{}", c);
                }
            }
            _ => {} // Ignore other characters
        }
    }

    fn execute_command(&mut self) {
        let cmd = self.input_buffer.trim();

        if cmd.is_empty() {
            return;
        }

        // Add to history
        self.command_history.push(String::from(cmd));

        // Parse command and arguments
        let parts: Vec<&str> = cmd.split_whitespace().collect();
        if parts.is_empty() {
            return;
        }

        let command = parts[0];
        let args = &parts[1..];

        // Execute built-in commands
        match command {
            "help" => self.cmd_help(args),
            "clear" => self.cmd_clear(args),
            "echo" => self.cmd_echo(args),
            "sysinfo" => self.cmd_sysinfo(args),
            "history" => self.cmd_history(args),
            "uptime" => self.cmd_uptime(args),
            "colors" => self.cmd_colors(args),
            "panic" => self.cmd_panic(args),
            "about" => self.cmd_about(args),
            "run" => self.cmd_run(args),
            "exec" => self.cmd_exec(args),
            "app" => self.cmd_app(args),
            "examples" => self.cmd_examples(args),
            // File system commands
            "ls" => self.cmd_ls(args),
            "cd" => self.cmd_cd(args),
            "pwd" => self.cmd_pwd(args),
            "cat" => self.cmd_cat(args),
            "mkdir" => self.cmd_mkdir(args),
            "touch" => self.cmd_touch(args),
            "rm" => self.cmd_rm(args),
            "write" => self.cmd_write(args),
            // AI command
            "ai" => self.cmd_ai(args),
            // Task/Scheduler commands
            "ps" => self.cmd_ps(args),
            "spawn" => self.cmd_spawn(args),
            "kill" => self.cmd_kill(args),
            "sched" => self.cmd_sched(args),
            "switch" => self.cmd_switch(args),
            "" => {},
            _ => {
                println!("Unknown command: '{}'. Type 'help' for available commands.", command);
            }
        }
    }

    pub fn print_prompt(&self) {
        print!("> ");
    }

    // Built-in commands

    fn cmd_help(&self, _args: &[&str]) {
        println!("Available commands:");
        println!();
        println!("System:");
        println!("  help            - Show this help message");
        println!("  about           - About this operating system");
        println!("  clear           - Clear the screen");
        println!("  sysinfo         - Display system information");
        println!("  uptime          - Show system uptime");
        println!("  colors          - Display color test");
        println!();
        println!("File System:");
        println!("  ls [path]       - List directory contents");
        println!("  cd <path>       - Change directory");
        println!("  pwd             - Print working directory");
        println!("  cat <file>      - Display file contents");
        println!("  mkdir <dir>     - Create directory");
        println!("  touch <file>    - Create empty file");
        println!("  rm <file>       - Remove file or directory");
        println!("  write <file> <text> - Write text to file");
        println!();
        println!("HAL Script:");
        println!("  run <code>      - Run HAL Script code");
        println!("  exec <file> [args...] - Execute .hal script file with arguments");
        println!("  examples        - Show HAL Script examples");
        println!();
        println!("Applications:");
        println!("  app list        - List installed applications");
        println!("  app run <name> [args...] - Run an application");
        println!();
        println!("AI Commands:");
        println!("  ai <request>    - Natural language programming");
        println!();
        println!("Task Management:");
        println!("  ps              - List all tasks");
        println!("  spawn <name> <priority> - Create a new task");
        println!("  kill <task_id>  - Terminate a task");
        println!("  sched           - Show scheduler status");
        println!("  switch <id>     - Switch to task (testing)");
        println!();
        println!("Other:");
        println!("  echo <text>     - Print text to the screen");
        println!("  history         - Show command history");
        println!("  panic           - Trigger a kernel panic (for testing)");
    }

    fn cmd_about(&self, _args: &[&str]) {
        println!("╔════════════════════════════════════════════════════════════╗");
        println!("║              MyOS - AI-Powered Operating System           ║");
        println!("╚════════════════════════════════════════════════════════════╝");
        println!();
        println!("Version: 0.1.0 (Development Build)");
        println!("Architecture: x86_64");
        println!("Kernel: Rust bare-metal microkernel");
        println!();
        println!("Features:");
        println!("  ✓ Hardware interrupt handling");
        println!("  ✓ Memory management (paging + heap)");
        println!("  ✓ VGA text mode driver");
        println!("  ✓ PS/2 keyboard driver");
        println!("  ✓ Interactive command shell");
        println!("  ✓ HAL Script programming language");
        println!("  ✓ Virtual file system");
        println!("  ⧗ Process scheduler (coming soon)");
        println!("  ⧗ AI integration (coming soon)");
        println!();
        println!("Built with Rust - Memory safe, blazingly fast!");
    }

    fn cmd_clear(&self, _args: &[&str]) {
        // Clear screen by printing many newlines
        for _ in 0..25 {
            println!();
        }
    }

    fn cmd_echo(&self, args: &[&str]) {
        if args.is_empty() {
            println!();
        } else {
            println!("{}", args.join(" "));
        }
    }

    fn cmd_sysinfo(&self, _args: &[&str]) {
        use crate::memory::allocator::{HEAP_START, HEAP_SIZE};

        println!("System Information:");
        println!("  OS Name:       MyOS");
        println!("  Version:       0.1.0");
        println!("  Architecture:  x86_64");
        println!("  Kernel Type:   Microkernel");
        println!();
        println!("Memory:");
        println!("  Heap Start:    0x{:x}", HEAP_START);
        println!("  Heap Size:     {} KB", HEAP_SIZE / 1024);
        println!();
        println!("Hardware:");
        println!("  Display:       VGA Text Mode (80x25)");
        println!("  Input:         PS/2 Keyboard");
        println!("  Timer:         PIT (Programmable Interval Timer)");
    }

    fn cmd_uptime(&self, _args: &[&str]) {
        // Get uptime from kernel
        let uptime_ms = crate::time::uptime_ms();
        let seconds = uptime_ms / 1000;
        let minutes = seconds / 60;
        let hours = minutes / 60;
        let days = hours / 24;

        if days > 0 {
            println!("Uptime: {} days, {} hours, {} minutes, {} seconds",
                     days, hours % 24, minutes % 60, seconds % 60);
        } else if hours > 0 {
            println!("Uptime: {} hours, {} minutes, {} seconds",
                     hours, minutes % 60, seconds % 60);
        } else if minutes > 0 {
            println!("Uptime: {} minutes, {} seconds", minutes, seconds % 60);
        } else {
            println!("Uptime: {} seconds", seconds);
        }
    }

    fn cmd_history(&self, _args: &[&str]) {
        if self.command_history.is_empty() {
            println!("No command history.");
        } else {
            println!("Command history:");
            for (i, cmd) in self.command_history.iter().enumerate() {
                println!("  {}: {}", i + 1, cmd);
            }
        }
    }

    fn cmd_colors(&self, _args: &[&str]) {
        use crate::vga_buffer::{WRITER, Color, ColorCode};
        use core::fmt::Write;

        println!("VGA Color Test:");
        println!();

        let colors = [
            ("Black", Color::Black),
            ("Blue", Color::Blue),
            ("Green", Color::Green),
            ("Cyan", Color::Cyan),
            ("Red", Color::Red),
            ("Magenta", Color::Magenta),
            ("Brown", Color::Brown),
            ("LightGray", Color::LightGray),
            ("DarkGray", Color::DarkGray),
            ("LightBlue", Color::LightBlue),
            ("LightGreen", Color::LightGreen),
            ("LightCyan", Color::LightCyan),
            ("LightRed", Color::LightRed),
            ("Pink", Color::Pink),
            ("Yellow", Color::Yellow),
            ("White", Color::White),
        ];

        for (name, color) in colors.iter() {
            let mut writer = WRITER.lock();
            let old_color = writer.color_code;
            writer.color_code = ColorCode::new(*color, Color::Black);
            let _ = write!(writer, "  ████ ");
            writer.color_code = old_color;
            drop(writer);
            println!("{}", name);
        }
        println!();
    }

    fn cmd_panic(&self, _args: &[&str]) {
        panic!("User requested kernel panic!");
    }

    fn cmd_run(&self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: run <code>");
            println!("Example: run print 2 + 2");
            return;
        }

        let code = args.join(" ");

        use crate::halscript::{lexer::Lexer, parser::Parser};

        // Tokenize
        let mut lexer = Lexer::new(&code);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => {
                println!("Lexer error: {}", e);
                return;
            }
        };

        // Parse
        let mut parser = Parser::new(tokens);
        let ast = match parser.parse() {
            Ok(a) => a,
            Err(e) => {
                println!("Parser error: {}", e);
                return;
            }
        };

        // Execute with persistent REPL
        let mut repl = crate::HAL_REPL.lock();
        if let Err(e) = repl.run(ast) {
            println!("Runtime error: {}", e);
        }
    }

    fn cmd_exec(&self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: exec <file.hal> [args...]");
            println!("Example: exec /scripts/fibonacci.hal");
            println!("Example: exec /apps/calc.hal 10 + 5");
            return;
        }

        use crate::vfs::VFS;
        use crate::halscript::{lexer::Lexer, parser::Parser};
        use alloc::string::String;
        use alloc::vec::Vec;

        // Read the file
        let vfs = VFS.lock();
        let code: String = match vfs.read_file(args[0]) {
            Ok(c) => c,
            Err(e) => {
                println!("Error reading file: {}", e);
                return;
            }
        };
        drop(vfs);

        // Collect script arguments (everything after the filename)
        let script_args: Vec<String> = args[1..].iter()
            .map(|s| String::from(*s))
            .collect();

        println!("[Executing {}]", args[0]);

        // Tokenize
        let mut lexer = Lexer::new(&code);
        let tokens = match lexer.tokenize() {
            Ok(t) => t,
            Err(e) => {
                println!("Lexer error: {}", e);
                return;
            }
        };

        // Parse
        let mut parser = Parser::new(tokens);
        let ast = match parser.parse() {
            Ok(a) => a,
            Err(e) => {
                println!("Parser error: {}", e);
                return;
            }
        };

        // Execute with persistent REPL, setting arguments first
        let mut repl = crate::HAL_REPL.lock();
        repl.set_args(script_args);
        if let Err(e) = repl.run(ast) {
            println!("Runtime error: {}", e);
        }
    }

    fn cmd_app(&self, args: &[&str]) {
        use crate::vfs::VFS;
        use alloc::string::String;
        use alloc::vec::Vec;

        if args.is_empty() {
            println!("Application Manager");
            println!();
            println!("Usage:");
            println!("  app list           - List all installed apps");
            println!("  app run <name> [args...] - Run an application");
            println!();
            println!("Examples:");
            println!("  app list");
            println!("  app run calc 10 + 5");
            println!("  app run greeter Alice");
            println!("  app run primefind 50");
            return;
        }

        match args[0] {
            "list" => {
                let vfs = VFS.lock();
                match vfs.list_directory("/apps") {
                    Ok(entries) => {
                        println!("Installed Applications:");
                        println!();
                        for (name, file_type, _) in entries {
                            if matches!(file_type, crate::vfs::FileType::File) {
                                // Remove .hal extension for display
                                let app_name = if name.ends_with(".hal") {
                                    &name[..name.len()-4]
                                } else {
                                    &name
                                };
                                println!("  {}", app_name);
                            }
                        }
                    }
                    Err(e) => println!("Error listing apps: {}", e),
                }
            }
            "run" => {
                if args.len() < 2 {
                    println!("Usage: app run <name> [args...]");
                    println!("Example: app run calc 10 + 5");
                    return;
                }

                let app_name = args[1];
                let app_path = if app_name.ends_with(".hal") {
                    format!("/apps/{}", app_name)
                } else {
                    format!("/apps/{}.hal", app_name)
                };

                // Collect app arguments (everything after the app name)
                let app_args: Vec<&str> = if args.len() > 2 {
                    args[2..].to_vec()
                } else {
                    Vec::new()
                };

                // Build full exec args (path + app arguments)
                let mut exec_args = Vec::new();
                exec_args.push(app_path.as_str());
                exec_args.extend(app_args);

                // Use exec command to run the app
                self.cmd_exec(&exec_args);
            }
            _ => {
                println!("Unknown app command: {}", args[0]);
                println!("Try: app list   or   app run <name>");
            }
        }
    }

    fn cmd_examples(&self, _args: &[&str]) {
        println!("HAL Script Examples:");
        println!();
        println!("1. Variables and Math:");
        println!("   run x = 10");
        println!("   run y = 20");
        println!("   run print x + y");
        println!();
        println!("2. Functions:");
        println!("   run fn add(a, b) {{ return a + b }}");
        println!("   run print add(5, 3)");
        println!();
        println!("3. Fibonacci:");
        println!("   run fn fib(n) {{ if n < 2 {{ return n }} return fib(n-1) + fib(n-2) }}");
        println!("   run print fib(10)");
        println!();
        println!("4. Loops:");
        println!("   run for i in 0..5 {{ print i }}");
        println!();
        println!("5. Arrays:");
        println!("   run arr = [1, 2, 3, 4, 5]");
        println!("   run print arr[2]");
        println!();
        println!("6. Conditionals:");
        println!("   run x = 42");
        println!("   run if x > 40 {{ print \"big\" }} else {{ print \"small\" }}");
        println!();
        println!("7. Built-in functions:");
        println!("   run print uptime()");
        println!("   run print len(\"hello\")");
    }

    // File system commands

    fn cmd_ls(&self, args: &[&str]) {
        use crate::vfs::{VFS, FileType};

        let path = if args.is_empty() {
            "."
        } else {
            args[0]
        };

        let mut vfs = VFS.lock();
        let actual_path = if path == "." {
            vfs.get_current_dir()
        } else {
            path
        };

        match vfs.list_directory(actual_path) {
            Ok(entries) => {
                let entries: Vec<_> = entries;
                if entries.is_empty() {
                    println!("Empty directory");
                } else {
                    for (name, file_type, size) in entries {
                        match file_type {
                            FileType::Directory => println!("  [DIR]  {}", name),
                            FileType::File => println!("  [FILE] {} ({} bytes)", name, size),
                        }
                    }
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    fn cmd_cd(&self, args: &[&str]) {
        use crate::vfs::VFS;

        if args.is_empty() {
            println!("Usage: cd <directory>");
            return;
        }

        let mut vfs = VFS.lock();
        if let Err(e) = vfs.change_directory(args[0]) {
            println!("Error: {}", e);
        }
    }

    fn cmd_pwd(&self, _args: &[&str]) {
        use crate::vfs::VFS;

        let vfs = VFS.lock();
        println!("{}", vfs.get_current_dir());
    }

    fn cmd_cat(&self, args: &[&str]) {
        use crate::vfs::VFS;

        if args.is_empty() {
            println!("Usage: cat <file>");
            return;
        }

        let vfs = VFS.lock();
        match vfs.read_file(args[0]) {
            Ok(content) => print!("{}", content),
            Err(e) => println!("Error: {}", e),
        }
    }

    fn cmd_mkdir(&self, args: &[&str]) {
        use crate::vfs::VFS;

        if args.is_empty() {
            println!("Usage: mkdir <directory>");
            return;
        }

        let mut vfs = VFS.lock();
        if let Err(e) = vfs.create_directory(args[0]) {
            println!("Error: {}", e);
        } else {
            println!("Directory created: {}", args[0]);
        }
    }

    fn cmd_touch(&self, args: &[&str]) {
        use crate::vfs::VFS;

        if args.is_empty() {
            println!("Usage: touch <file>");
            return;
        }

        let mut vfs = VFS.lock();
        if let Err(e) = vfs.create_file(args[0], String::new()) {
            println!("Error: {}", e);
        } else {
            println!("File created: {}", args[0]);
        }
    }

    fn cmd_rm(&self, args: &[&str]) {
        use crate::vfs::VFS;

        if args.is_empty() {
            println!("Usage: rm <file>");
            return;
        }

        let mut vfs = VFS.lock();
        if let Err(e) = vfs.delete(args[0]) {
            println!("Error: {}", e);
        } else {
            println!("Removed: {}", args[0]);
        }
    }

    fn cmd_write(&self, args: &[&str]) {
        use crate::vfs::VFS;

        if args.len() < 2 {
            println!("Usage: write <file> <content>");
            return;
        }

        let filename = args[0];
        let content = args[1..].join(" ");

        let mut vfs = VFS.lock();
        if let Err(e) = vfs.write_file(filename, content) {
            println!("Error: {}", e);
        } else {
            println!("Written to: {}", filename);
        }
    }

    // AI command processor
    fn cmd_ai(&self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: ai <request>");
            println!();
            println!("Examples:");
            println!("  ai create a fibonacci function");
            println!("  ai count from 1 to 10");
            println!("  ai show prime numbers under 50");
            println!("  ai calculate 15 factorial");
            return;
        }

        let request = args.join(" ").to_lowercase();

        println!("[AI] Processing: '{}'", request);

        // Pattern matching for common requests
        let code = if request.contains("fibonacci") || request.contains("fib") {
            Some("fn fib(n) { if n < 2 { return n } return fib(n-1) + fib(n-2) }\nprint \"Fibonacci function created! Try: run print fib(10)\"")
        } else if request.contains("prime") {
            let number = Self::extract_number(&request).unwrap_or(100);
            Some(format!("fn is_prime(n) {{ if n < 2 {{ return false }} i = 2 while i * i <= n {{ if n % i == 0 {{ return false }} i = i + 1 }} return true }}\nfor num in 2..{} {{ if is_prime(num) {{ print num }} }}", number).leak() as &str)
        } else if request.contains("count") || request.contains("numbers") {
            let start = if request.contains("from") { Self::extract_number(&request).unwrap_or(1) } else { 1 };
            let end = if request.contains("to") { Self::extract_number_after_word(&request, "to").unwrap_or(10) } else { 10 };
            Some(format!("for i in {}..{} {{ print i }}", start, end + 1).leak() as &str)
        } else if request.contains("factorial") {
            let n = Self::extract_number(&request).unwrap_or(10);
            Some(format!("fn factorial(n) {{ if n <= 1 {{ return 1 }} return n * factorial(n - 1) }}\nprint factorial({})", n).leak() as &str)
        } else if request.contains("sum") || request.contains("add") {
            if request.contains("array") || request.contains("list") {
                Some("arr = [1, 2, 3, 4, 5]\nsum = 0\nfor i in 0..len(arr) { sum = sum + arr[i] }\nprint sum")
            } else {
                Some("print \"Create a sum function? Try: ai sum an array\"")
            }
        } else if request.contains("hello") || request.contains("hi") {
            Some("print \"Hello from MyOS! I'm your AI assistant.\"")
        } else if request.contains("uptime") {
            Some("print \"System uptime: \" + uptime() + \" seconds\"")
        } else if request.contains("fizzbuzz") {
            Some("for i in 1..101 { if i % 15 == 0 { print \"FizzBuzz\" } else if i % 3 == 0 { print \"Fizz\" } else if i % 5 == 0 { print \"Buzz\" } else { print i } }")
        } else if request.contains("create") && request.contains("function") {
            Some("print \"To create a function, use: run fn myfunction(param) { ... }\"")
        } else if request.contains("help") || request.contains("what can you do") {
            println!("I can help you write HAL Script code! Try:");
            println!("  - ai create a fibonacci function");
            println!("  - ai show prime numbers under 50");
            println!("  - ai count from 1 to 100");
            println!("  - ai calculate factorial");
            println!("  - ai fizzbuzz");
            return;
        } else {
            None
        };

        if let Some(generated_code) = code {
            println!("[AI] Generated code:");
            println!("{}", generated_code);
            println!();
            println!("[AI] Executing...");
            println!();

            use crate::halscript::{lexer::Lexer, parser::Parser};

            let mut lexer = Lexer::new(generated_code);
            let tokens = match lexer.tokenize() {
                Ok(t) => t,
                Err(e) => {
                    println!("[AI] Error tokenizing: {}", e);
                    return;
                }
            };

            let mut parser = Parser::new(tokens);
            let ast = match parser.parse() {
                Ok(a) => a,
                Err(e) => {
                    println!("[AI] Error parsing: {}", e);
                    return;
                }
            };

            let mut repl = crate::HAL_REPL.lock();
            if let Err(e) = repl.run(ast) {
                println!("[AI] Runtime error: {}", e);
            }
        } else {
            println!("[AI] I'm not sure how to help with that yet.");
            println!("[AI] Try 'ai help' to see what I can do!");
        }
    }

    fn extract_number(text: &str) -> Option<i64> {
        text.split_whitespace()
            .find_map(|word| word.parse::<i64>().ok())
    }

    fn extract_number_after_word(text: &str, word: &str) -> Option<i64> {
        let parts: Vec<&str> = text.split_whitespace().collect();
        for (i, &w) in parts.iter().enumerate() {
            if w == word && i + 1 < parts.len() {
                if let Ok(n) = parts[i + 1].parse::<i64>() {
                    return Some(n);
                }
            }
        }
        None
    }

    // Task Management Commands

    fn cmd_ps(&self, _args: &[&str]) {
        use crate::SCHEDULER;

        let scheduler = SCHEDULER.lock();
        let tasks = scheduler.list_tasks();

        if tasks.is_empty() {
            println!("No tasks running.");
            return;
        }

        println!("Task List:");
        println!("  ID    Name                State       Priority");
        println!("  ─────────────────────────────────────────────────");

        for task in tasks {
            let state_str = match task.state() {
                crate::task::TaskState::Ready => "Ready",
                crate::task::TaskState::Running => "Running",
                crate::task::TaskState::Waiting => "Waiting",
                crate::task::TaskState::Terminated => "Terminated",
            };
            println!("  {:<5} {:<19} {:<11} {}",
                task.id(),
                task.name(),
                state_str,
                task.priority()
            );
        }
    }

    fn cmd_spawn(&self, args: &[&str]) {
        use crate::SCHEDULER;
        use crate::task::Task;
        use alloc::string::ToString;

        if args.is_empty() {
            println!("Usage: spawn <name> [priority]");
            return;
        }

        let name = args[0].to_string();
        let priority = if args.len() > 1 {
            args[1].parse::<u8>().unwrap_or(5)
        } else {
            5  // Default priority
        };

        // Default task function for shell-spawned tasks
        extern "C" fn default_task() {
            loop {
                x86_64::instructions::hlt();
            }
        }

        let task = Task::new(name, priority, default_task);
        let task_id = task.id();

        let mut scheduler = SCHEDULER.lock();
        scheduler.add_task(task);

        println!("Task created with ID: {}", task_id);
    }

    fn cmd_kill(&self, args: &[&str]) {
        use crate::SCHEDULER;

        if args.is_empty() {
            println!("Usage: kill <task_id>");
            return;
        }

        let task_id = match args[0].parse::<u64>() {
            Ok(id) => id,
            Err(_) => {
                println!("Invalid task ID: {}", args[0]);
                return;
            }
        };

        let mut scheduler = SCHEDULER.lock();
        match scheduler.remove_task(task_id) {
            Ok(_) => println!("Task {} terminated.", task_id),
            Err(e) => println!("Error: {}", e),
        }
    }

    fn cmd_sched(&self, _args: &[&str]) {
        use crate::SCHEDULER;

        let scheduler = SCHEDULER.lock();

        println!("Scheduler Status:");
        println!("  Total tasks:  {}", scheduler.task_count());
        println!("  Ready tasks:  {}", scheduler.ready_task_count());
        println!("  Scheduler:    Round-Robin");
    }

    fn cmd_switch(&self, args: &[&str]) {
        use crate::SCHEDULER;

        if args.is_empty() {
            println!("Usage: switch <task_id>");
            println!("Manually switch to a specific task (for testing context switching)");
            return;
        }

        let task_id = match args[0].parse::<u64>() {
            Ok(id) => id,
            Err(_) => {
                println!("Error: Invalid task ID");
                return;
            }
        };

        println!("Attempting to switch to task {}...", task_id);

        let mut scheduler = SCHEDULER.lock();
        match scheduler.switch_to(task_id) {
            Ok(()) => {
                println!("Successfully switched to task {}", task_id);
            }
            Err(e) => {
                println!("Error switching to task: {}", e);
            }
        }
    }
}
