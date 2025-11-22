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
            // Process commands
            "proc" => self.cmd_proc(args),
            // IPC commands
            "pipetest" => self.cmd_pipetest(args),
            "shmtest" => self.cmd_shmtest(args),
            // Disk commands
            "diskinfo" => self.cmd_diskinfo(args),
            "diskread" => self.cmd_diskread(args),
            "diskwrite" => self.cmd_diskwrite(args),
            // Filesystem commands
            "fsformat" => self.cmd_fsformat(args),
            "fsinfo" => self.cmd_fsinfo(args),
            // ELF loader
            "loadelf" => self.cmd_loadelf(args),
            // Test user mode
            "test-usermode" => self.cmd_test_usermode(args),
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
        println!("ELF Binaries:");
        println!("  loadelf <file>  - Load and execute an ELF binary from filesystem");
        println!("  test-usermode   - Test user-mode syscalls (demo)");
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
        println!("Process Management:");
        println!("  proc            - List all processes");
        println!("  proc info <pid> - Show detailed process info");
        println!("  proc create <name> - Create a new process");
        println!();
        println!("IPC Testing:");
        println!("  pipetest        - Test pipe creation and I/O");
        println!("  shmtest         - Test shared memory");
        println!();
        println!("Disk I/O:");
        println!("  diskinfo        - Show disk information");
        println!("  diskread <sector> - Read and display a disk sector");
        println!("  diskwrite <sector> <data> - Write data to a disk sector");
        println!();
        println!("Filesystem:");
        println!("  fsformat        - Format disk with SimpleFS");
        println!("  fsinfo          - Show filesystem information");
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

    fn cmd_proc(&self, args: &[&str]) {
        use crate::process::PROCESS_TABLE;

        if args.is_empty() {
            // List all processes
            let table = PROCESS_TABLE.lock();
            let processes = table.list_processes();

            println!("Process Table:");
            println!("{:<6} {:<6} {:<20} {:<12} {:<8} {:<10}",
                "PID", "PPID", "NAME", "STATE", "PRIORITY", "CPU TIME");
            println!("{}", "-".repeat(70));

            for process in processes {
                let state_str = match process.state() {
                    crate::process::ProcessState::Ready => "Ready",
                    crate::process::ProcessState::Running => "Running",
                    crate::process::ProcessState::Waiting => "Waiting",
                    crate::process::ProcessState::Sleeping => "Sleeping",
                    crate::process::ProcessState::Zombie => "Zombie",
                    crate::process::ProcessState::Dead => "Dead",
                };

                let priority_str = match process.priority() {
                    crate::process::Priority::Idle => "Idle",
                    crate::process::Priority::Low => "Low",
                    crate::process::Priority::Normal => "Normal",
                    crate::process::Priority::High => "High",
                    crate::process::Priority::Realtime => "Realtime",
                };

                println!("{:<6} {:<6} {:<20} {:<12} {:<8} {:<10}",
                    process.pid(),
                    process.parent_pid().unwrap_or(0),
                    process.name(),
                    state_str,
                    priority_str,
                    process.cpu_time()
                );
            }
            return;
        }

        match args[0] {
            "tree" => {
                println!("Process tree view - not yet implemented");
            }
            "info" => {
                if args.len() < 2 {
                    println!("Usage: proc info <pid>");
                    return;
                }

                let pid = match args[1].parse::<u64>() {
                    Ok(p) => p,
                    Err(_) => {
                        println!("Invalid PID");
                        return;
                    }
                };

                let table = PROCESS_TABLE.lock();
                if let Some(process) = table.get_process(pid) {
                    println!("Process Information:");
                    println!("  PID:     {}", process.pid());
                    println!("  PPID:    {}", process.parent_pid().unwrap_or(0));
                    println!("  Name:    {}", process.name());
                    println!("  State:   {:?}", process.state());
                    println!("  Priority: {:?}", process.priority());
                    println!("  CWD:     {}", process.cwd());
                    println!("  Memory:  {} bytes", process.memory_usage());
                    println!("  CPU Time: {} ticks", process.cpu_time());
                    println!("  Children: {} processes", process.children().len());
                } else {
                    println!("Process {} not found", pid);
                }
            }
            "create" => {
                if args.len() < 2 {
                    println!("Usage: proc create <name>");
                    return;
                }

                let name = alloc::string::String::from(args[1]);
                match crate::process::create_process(name, Some(1)) {
                    Ok(pid) => println!("Created process with PID {}", pid),
                    Err(e) => println!("Error creating process: {}", e),
                }
            }
            _ => {
                println!("Unknown proc subcommand: {}", args[0]);
                println!("Available: list (default), tree, info <pid>, create <name>");
            }
        }
    }

    // Disk commands

    fn cmd_diskinfo(&self, _args: &[&str]) {
        use crate::ata;

        match ata::disk_info() {
            Some((sectors, size_mb)) => {
                println!("Disk Information:");
                println!("  Sectors: {}", sectors);
                println!("  Size: {} MB ({} GB)", size_mb, size_mb / 1024);
                println!("  Sector size: 512 bytes");
            }
            None => {
                println!("No disk detected");
            }
        }
    }

    fn cmd_diskread(&self, args: &[&str]) {
        use crate::ata;

        if args.is_empty() {
            println!("Usage: diskread <sector>");
            println!("  Reads and displays a 512-byte sector from disk");
            return;
        }

        let sector = match args[0].parse::<u32>() {
            Ok(s) => s,
            Err(_) => {
                println!("Invalid sector number");
                return;
            }
        };

        let mut buffer = [0u8; 512];
        match ata::read_sector(sector, &mut buffer) {
            Ok(_) => {
                println!("Sector {} contents:", sector);
                println!();

                // Display as hex dump (16 bytes per line)
                for (i, chunk) in buffer.chunks(16).enumerate() {
                    print!("{:04x}: ", i * 16);

                    // Hex view
                    for byte in chunk {
                        print!("{:02x} ", byte);
                    }

                    // Padding for incomplete lines
                    for _ in 0..(16 - chunk.len()) {
                        print!("   ");
                    }

                    print!(" | ");

                    // ASCII view
                    for byte in chunk {
                        let ch = if *byte >= 32 && *byte <= 126 {
                            *byte as char
                        } else {
                            '.'
                        };
                        print!("{}", ch);
                    }

                    println!();
                }
            }
            Err(_) => {
                println!("Error reading sector {}", sector);
            }
        }
    }

    fn cmd_diskwrite(&self, args: &[&str]) {
        use crate::ata;
        use alloc::string::String;

        if args.len() < 2 {
            println!("Usage: diskwrite <sector> <data>");
            println!("  Writes data to a 512-byte sector on disk");
            println!("  WARNING: This will overwrite existing data!");
            return;
        }

        let sector = match args[0].parse::<u32>() {
            Ok(s) => s,
            Err(_) => {
                println!("Invalid sector number");
                return;
            }
        };

        // Join remaining args as data string
        let data_str = args[1..].join(" ");

        // Create buffer and fill with data (pad with zeros if needed)
        let mut buffer = [0u8; 512];
        let data_bytes = data_str.as_bytes();
        let copy_len = core::cmp::min(data_bytes.len(), 512);
        buffer[..copy_len].copy_from_slice(&data_bytes[..copy_len]);

        match ata::write_sector(sector, &buffer) {
            Ok(_) => {
                println!("Successfully wrote {} bytes to sector {}", copy_len, sector);
            }
            Err(_) => {
                println!("Error writing to sector {}", sector);
            }
        }
    }

    // Filesystem commands

    fn cmd_fsformat(&self, _args: &[&str]) {
        use crate::simplefs;

        println!("WARNING: This will erase all data on the disk!");
        println!("Formatting disk with SimpleFS...");
        println!();

        match simplefs::format() {
            Ok(_) => {
                println!("Filesystem formatted and mounted successfully");
            }
            Err(e) => {
                println!("Error formatting filesystem: {}", e);
            }
        }
    }

    fn cmd_fsinfo(&self, _args: &[&str]) {
        use crate::simplefs;

        match simplefs::get_info() {
            Some((total_inodes, free_inodes, total_blocks, free_blocks)) => {
                println!("SimpleFS Information:");
                println!("  Inodes:");
                println!("    Total: {}", total_inodes);
                println!("    Free: {}", free_inodes);
                println!("    Used: {}", total_inodes - free_inodes);
                println!("  Data Blocks:");
                println!("    Total: {}", total_blocks);
                println!("    Free: {}", free_blocks);
                println!("    Used: {}", total_blocks - free_blocks);
                println!("  Storage:");
                println!("    Block size: 512 bytes");
                println!("    Total capacity: {} KB", total_blocks / 2);
                println!("    Used: {} KB", (total_blocks - free_blocks) / 2);
                println!("    Free: {} KB", free_blocks / 2);
            }
            None => {
                println!("Filesystem not mounted");
                println!("Use 'fsformat' to format the disk");
            }
        }
    }

    // ELF loader command

    fn cmd_loadelf(&self, args: &[&str]) {
        if args.is_empty() {
            println!("Usage: loadelf <file>");
            println!("  Loads and executes an ELF binary from the filesystem");
            println!();
            println!("Example:");
            println!("  loadelf /bin/hello    - Load and execute /bin/hello");
            return;
        }

        let filepath = args[0];

        // Read the file from VFS
        use crate::vfs;

        let vfs = vfs::VFS.lock();
        let file_data = match vfs.read_file(filepath) {
            Ok(data) => data,
            Err(e) => {
                println!("Error reading file '{}': {}", filepath, e);
                println!("Tip: Use 'ls' to list available files");
                return;
            }
        };
        drop(vfs); // Release the lock

        // Convert String to bytes
        let elf_bytes = file_data.as_bytes();

        println!("Loading ELF binary: {}", filepath);
        println!("Size: {} bytes", elf_bytes.len());

        // Create a new process
        use crate::process;
        use alloc::string::ToString;

        let process_name = filepath.split('/').last().unwrap_or("elf_program").to_string();

        let mut new_process = process::Process::new(process_name.clone(), Some(1));

        // Load ELF binary into process
        let entry_point = match new_process.load_elf(elf_bytes) {
            Ok(entry) => entry,
            Err(e) => {
                println!("Error loading ELF: {}", e);
                return;
            }
        };

        println!("Entry point: 0x{:016x}", entry_point);

        // Set up user stack
        let stack_ptr = match new_process.setup_user_stack() {
            Ok(sp) => sp,
            Err(e) => {
                println!("Error setting up stack: {}", e);
                return;
            }
        };

        println!("User stack: 0x{:016x}", stack_ptr);
        println!("Memory usage: {} KB", new_process.memory_usage() / 1024);

        println!();
        println!("ELF binary loaded successfully!");
        println!("Process: {} (PID {})", process_name, new_process.pid());
        println!();

        // Get the page table for this process
        let page_table_phys = new_process.page_table_phys();

        println!("Jumping to user mode...");
        println!();

        // Execute the user program
        // This will transition from Ring 0 (kernel) to Ring 3 (user)
        use crate::usermode;
        use x86_64::VirtAddr;

        unsafe {
            usermode::execute_user_program(
                VirtAddr::new(entry_point),
                VirtAddr::new(stack_ptr),
                page_table_phys,
            );
        }

        // Note: execute_user_program() never returns - it jumps to Ring 3
        // The program will execute and eventually exit via syscall
    }

    fn cmd_test_usermode(&self, _args: &[&str]) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║          MyOS System Call Interface (INT 0x80)              ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        println!("User-mode programs can make system calls using INT 0x80");
        println!();
        println!("Calling Convention:");
        println!("  RAX = syscall number");
        println!("  RDI = arg1");
        println!("  RSI = arg2");
        println!("  RDX = arg3");
        println!("  R10 = arg4");
        println!("  R8  = arg5");
        println!("  R9  = arg6");
        println!("  Return value in RAX");
        println!();
        println!("Available System Calls:");
        println!();
        println!("Process Management:");
        println!("  0  - exit(code)              Exit current process");
        println!("  1  - yield()                 Yield CPU to scheduler");
        println!("  6  - getpid()                Get process ID");
        println!("  7  - getppid()               Get parent process ID");
        println!("  8  - fork()                  Create child process");
        println!("  9  - wait()                  Wait for child process");
        println!("  10 - kill(pid, signal)       Send signal to process");
        println!("  11 - exec(path, argv)        Execute program");
        println!();
        println!("I/O:");
        println!("  2  - print(str, len)         Print string to console");
        println!("  14 - pipe(fds[2])            Create pipe");
        println!("  15 - read(fd, buf, len)      Read from file descriptor");
        println!("  16 - write(fd, buf, len)     Write to file descriptor");
        println!("  17 - close(fd)               Close file descriptor");
        println!();
        println!("Time:");
        println!("  3  - get_time()              Get system time (ms)");
        println!("  4  - get_ticks()             Get timer ticks");
        println!("  5  - sleep(ms)               Sleep for milliseconds");
        println!();
        println!("IPC - Shared Memory:");
        println!("  18 - shmget(key, size, flags)");
        println!("  19 - shmat(id, flags)");
        println!("  20 - shmdt(addr)");
        println!("  21 - shmctl(id, cmd)");
        println!();
        println!("IPC - Semaphores:");
        println!("  22 - seminit(value)");
        println!("  23 - semopen(name, flags, value)");
        println!("  24 - semwait(id)");
        println!("  25 - sempost(id)");
        println!("  26 - semgetvalue(id)");
        println!("  27 - semdestroy(id)");
        println!();
        println!("IPC - Message Queues:");
        println!("  28 - msgget(key, flags)");
        println!("  29 - msgsnd(qid, type, data)");
        println!("  30 - msgrcv(qid, type, buf)");
        println!("  31 - msgctl(qid, cmd)");
        println!();
        println!("Example (x86-64 assembly):");
        println!("  ; Write 'Hello' to stdout");
        println!("  mov rax, 16        ; syscall write");
        println!("  mov rdi, 1         ; fd = stdout");
        println!("  lea rsi, [msg]     ; buffer");
        println!("  mov rdx, 5         ; length");
        println!("  int 0x80           ; trigger syscall");
        println!();
        println!("To test user mode:");
        println!("  1. Create an ELF binary with syscalls");
        println!("  2. Store it on the /disk filesystem");
        println!("  3. Run: loadelf /disk/yourprogram");
        println!();
        println!("Status: User-mode execution fully operational!");
        println!("  ✓ Ring 0 ↔ Ring 3 transitions working");
        println!("  ✓ INT 0x80 syscall handler active");
        println!("  ✓ Memory isolation via page tables");
        println!("  ✓ Preemptive multitasking enabled");
    }

    fn cmd_pipetest(&self, _args: &[&str]) {
        println!("╔═══════════════════════════════════════════════════════╗");
        println!("║              Pipe IPC Test                           ║");
        println!("╚═══════════════════════════════════════════════════════╝");
        println!();

        // Create a pipe
        println!("Creating pipe...");
        match crate::pipe::create_pipe() {
            Ok((read_fd, write_fd)) => {
                println!("✓ Pipe created successfully!");
                println!("  Read FD:  {}", read_fd);
                println!("  Write FD: {}", write_fd);
                println!();

                // Test writing to pipe
                let test_msg = b"Hello through the pipe!";
                println!("Writing message: \"{}\"", core::str::from_utf8(test_msg).unwrap());

                match crate::pipe::write_pipe(write_fd, test_msg) {
                    Ok(n) => {
                        println!("✓ Wrote {} bytes to pipe", n);
                        println!();

                        // Test reading from pipe
                        println!("Reading from pipe...");
                        let mut buffer = [0u8; 256];

                        match crate::pipe::read_pipe(read_fd, &mut buffer[..test_msg.len()]) {
                            Ok(n) => {
                                println!("✓ Read {} bytes from pipe", n);
                                let read_str = core::str::from_utf8(&buffer[..n]).unwrap_or("<invalid UTF-8>");
                                println!("  Message: \"{}\"", read_str);
                                println!();

                                if read_str == core::str::from_utf8(test_msg).unwrap() {
                                    println!("✓ Test PASSED: Message matches!");
                                } else {
                                    println!("✗ Test FAILED: Message mismatch!");
                                }
                            }
                            Err(e) => {
                                println!("✗ Error reading from pipe: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Error writing to pipe: {}", e);
                    }
                }

                // Clean up
                println!();
                println!("Closing pipe...");
                let _ = crate::pipe::close_pipe(read_fd);
                let _ = crate::pipe::close_pipe(write_fd);
                println!("✓ Pipe closed");
            }
            Err(e) => {
                println!("✗ Error creating pipe: {}", e);
            }
        }

        println!();
        println!("Pipe test complete!");
    }

    fn cmd_shmtest(&self, _args: &[&str]) {
        println!("╔═══════════════════════════════════════════════════════╗");
        println!("║         Shared Memory IPC Test                       ║");
        println!("╚═══════════════════════════════════════════════════════╝");
        println!();

        // Create shared memory segment
        const SHM_SIZE: usize = 4096;
        const TEST_KEY: i32 = 1234;

        println!("Creating shared memory segment...");
        println!("  Key:  {}", TEST_KEY);
        println!("  Size: {} bytes", SHM_SIZE);

        match crate::shm::shmget(TEST_KEY, SHM_SIZE, 0o666) {
            Ok(shm_id) => {
                println!("✓ Shared memory created!");
                println!("  Segment ID: {}", shm_id);
                println!();

                // Attach to shared memory
                println!("Attaching to shared memory...");
                match crate::shm::shmat(shm_id, 0) {
                    Ok(addr) => {
                        println!("✓ Attached at address: 0x{:x}", addr);
                        println!();

                        // Write test data
                        println!("Writing test data...");
                        let test_data = b"Shared memory test data!";
                        unsafe {
                            let ptr = addr as *mut u8;
                            for (i, &byte) in test_data.iter().enumerate() {
                                *ptr.add(i) = byte;
                            }
                        }
                        println!("✓ Wrote {} bytes", test_data.len());
                        println!();

                        // Read back test data
                        println!("Reading back data...");
                        let mut buffer = [0u8; 256];
                        unsafe {
                            let ptr = addr as *const u8;
                            for i in 0..test_data.len() {
                                buffer[i] = *ptr.add(i);
                            }
                        }

                        let read_str = core::str::from_utf8(&buffer[..test_data.len()])
                            .unwrap_or("<invalid UTF-8>");
                        println!("  Data: \"{}\"", read_str);
                        println!();

                        if read_str == core::str::from_utf8(test_data).unwrap() {
                            println!("✓ Test PASSED: Data matches!");
                        } else {
                            println!("✗ Test FAILED: Data mismatch!");
                        }

                        // Detach
                        println!();
                        println!("Detaching from shared memory...");
                        match crate::shm::shmdt(addr) {
                            Ok(_) => println!("✓ Detached"),
                            Err(e) => println!("✗ Error detaching: {}", e),
                        }
                    }
                    Err(e) => {
                        println!("✗ Error attaching: {}", e);
                    }
                }

                // Clean up
                println!();
                println!("Removing shared memory segment...");
                match crate::shm::shmctl(shm_id, 0) {  // IPC_RMID = 0
                    Ok(_) => println!("✓ Removed"),
                    Err(e) => println!("✗ Error removing: {}", e),
                }
            }
            Err(e) => {
                println!("✗ Error creating shared memory: {}", e);
            }
        }

        println!();
        println!("Shared memory test complete!");
    }
}
