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
        println!("  examples        - Show HAL Script examples");
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

        use crate::halscript::{lexer::Lexer, parser::Parser, Interpreter};

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

        // Execute
        let mut interpreter = Interpreter::new();
        if let Err(e) = interpreter.run(ast) {
            println!("Runtime error: {}", e);
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
}
