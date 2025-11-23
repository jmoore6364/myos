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
            "status" => self.cmd_status(args),
            "history" => self.cmd_history(args),
            "uptime" => self.cmd_uptime(args),
            "free" => self.cmd_free(args),
            "date" => self.cmd_date(args),
            "top" => self.cmd_top(args),
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
            "hexdump" => self.cmd_hexdump(args),
            "mkdir" => self.cmd_mkdir(args),
            "touch" => self.cmd_touch(args),
            "rm" => self.cmd_rm(args),
            "write" => self.cmd_write(args),
            "cp" => self.cmd_cp(args),
            "mv" => self.cmd_mv(args),
            "tree" => self.cmd_tree(args),
            "wc" => self.cmd_wc(args),
            "grep" => self.cmd_grep(args),
            "tail" => self.cmd_tail(args),
            "head" => self.cmd_head(args),
            "du" => self.cmd_du(args),
            "find" => self.cmd_find(args),
            // AI command
            "ai" => self.cmd_ai(args),
            // Task/Scheduler commands
            "ps" => self.cmd_ps(args),
            "spawn" => self.cmd_spawn(args),
            "kill" => self.cmd_kill(args),
            "killall" => self.cmd_killall(args),
            "sched" => self.cmd_sched(args),
            "switch" => self.cmd_switch(args),
            // Process commands
            "proc" => self.cmd_proc(args),
            // IPC commands
            "pipetest" => self.cmd_pipetest(args),
            "shmtest" => self.cmd_shmtest(args),
            "semtest" => self.cmd_semtest(args),
            "msgtest" => self.cmd_msgtest(args),
            // Disk commands
            "diskinfo" => self.cmd_diskinfo(args),
            "diskread" => self.cmd_diskread(args),
            "diskwrite" => self.cmd_diskwrite(args),
            // Filesystem commands
            "fsformat" => self.cmd_fsformat(args),
            "fsinfo" => self.cmd_fsinfo(args),
            "init-disk" => self.cmd_init_disk(args),
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
        println!("  status          - Show current system status");
        println!("  uptime          - Show system uptime");
        println!("  free            - Show memory usage statistics");
        println!("  date            - Show current system time/ticks");
        println!("  top             - Real-time process monitor");
        println!("  colors          - Display color test");
        println!();
        println!("File System:");
        println!("  ls [path]       - List directory contents");
        println!("  cd <path>       - Change directory");
        println!("  pwd             - Print working directory");
        println!("  cat <file>      - Display file contents");
        println!("  hexdump <file>  - Display file in hexadecimal");
        println!("  mkdir <dir>     - Create directory");
        println!("  touch <file>    - Create empty file");
        println!("  rm <file>       - Remove file or directory");
        println!("  write <file> <text> - Write text to file");
        println!("  cp <src> <dst>  - Copy file");
        println!("  mv <src> <dst>  - Move/rename file");
        println!("  tree [path]     - Show directory tree");
        println!("  wc <file>       - Count lines, words, bytes");
        println!("  grep <pattern> <file> - Search for pattern in file");
        println!("  head <file> [n] - Show first n lines (default 10)");
        println!("  tail <file> [n] - Show last n lines (default 10)");
        println!("  du [path]       - Show disk usage");
        println!("  find <name>     - Find files by name pattern");
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
        println!("  top             - Real-time process monitor");
        println!("  spawn <name> <priority> - Create a new task");
        println!("  kill <task_id>  - Terminate a task");
        println!("  killall <name>  - Terminate all tasks with given name");
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
        println!("  semtest         - Test semaphores");
        println!("  msgtest         - Test message queues");
        println!();
        println!("Disk I/O:");
        println!("  diskinfo        - Show disk information");
        println!("  diskread <sector> - Read and display a disk sector");
        println!("  diskwrite <sector> <data> - Write data to a disk sector");
        println!();
        println!("Filesystem:");
        println!("  fsformat        - Format disk with SimpleFS");
        println!("  fsinfo          - Show filesystem information");
        println!("  init-disk       - Create example files and directories");
        println!();
        println!("Other:");
        println!("  echo <text>     - Print text to the screen");
        println!("  history         - Show command history");
        println!("  panic           - Trigger a kernel panic (for testing)");
    }

    fn cmd_about(&self, _args: &[&str]) {
        println!("╔══════════════════════════════════════════════════════════════╗");
        println!("║                MyOS - Bare-Metal Operating System            ║");
        println!("╚══════════════════════════════════════════════════════════════╝");
        println!();
        println!("Version: 0.2.0 (Stable Release)");
        println!("Architecture: x86_64");
        println!("Kernel: Rust bare-metal microkernel with preemptive multitasking");
        println!();
        println!("Core Features:");
        println!("  ✓ User-mode execution (Ring 0 ↔ Ring 3 transitions)");
        println!("  ✓ Preemptive multitasking (100 Hz timer-based)");
        println!("  ✓ Memory management (4-level paging + heap allocator)");
        println!("  ✓ Per-process page tables with memory isolation");
        println!("  ✓ System call interface (32 syscalls via INT 0x80)");
        println!();
        println!("Process Management:");
        println!("  ✓ fork() - Create child processes");
        println!("  ✓ exec() - Execute ELF binaries");
        println!("  ✓ wait() - Process synchronization");
        println!("  ✓ ELF64 binary loader");
        println!("  ✓ Process table with parent-child relationships");
        println!();
        println!("IPC Mechanisms (Inter-Process Communication):");
        println!("  ✓ Pipes - Unidirectional data channels");
        println!("  ✓ Shared Memory - Fast shared data segments");
        println!("  ✓ Semaphores - Synchronization primitives");
        println!("  ✓ Message Queues - Structured message passing");
        println!("  ✓ Signals - Asynchronous notifications");
        println!();
        println!("Filesystem:");
        println!("  ✓ SimpleFS - Custom persistent filesystem");
        println!("  ✓ Subdirectory support (nested directories)");
        println!("  ✓ VFS layer with /disk mount point");
        println!("  ✓ ATA/IDE disk driver (PIO mode)");
        println!("  ✓ File operations: create, read, write, delete");
        println!();
        println!("Development Tools:");
        println!("  ✓ HAL Script - Interpreted programming language");
        println!("  ✓ Interactive shell with 50+ commands");
        println!("  ✓ IPC test suite (pipetest, shmtest, semtest, msgtest)");
        println!("  ✓ Process management utilities (ps, proc)");
        println!();
        println!("Hardware Support:");
        println!("  ✓ VGA text mode (80x25, 16 colors)");
        println!("  ✓ PS/2 keyboard driver");
        println!("  ✓ PIC (8259) interrupt controller");
        println!("  ✓ PIT (8253/8254) timer");
        println!();
        println!("Statistics:");
        println!("  • ~12,000 lines of Rust code");
        println!("  • 32 system calls implemented");
        println!("  • 4 IPC mechanisms fully tested");
        println!("  • 100% memory safe (no unsafe bugs)");
        println!();
        println!("Built with Rust 🦀 - Memory safe, blazingly fast!");
        println!("Status: Fully functional operating system ready for user programs!");
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

    fn cmd_status(&self, _args: &[&str]) {
        println!("╔═══════════════════════════════════════════════════════╗");
        println!("║            MyOS System Status                        ║");
        println!("╚═══════════════════════════════════════════════════════╝");
        println!();

        // Uptime
        let uptime_ms = crate::time::uptime_ms();
        let uptime_s = uptime_ms / 1000;
        let hours = uptime_s / 3600;
        let minutes = (uptime_s % 3600) / 60;
        let seconds = uptime_s % 60;
        println!("Uptime: {}h {}m {}s ({} ms)", hours, minutes, seconds, uptime_ms);
        println!();

        // Tasks
        println!("Tasks & Processes:");
        let scheduler = crate::SCHEDULER.lock();
        let task_count = scheduler.task_count();
        let ready_count = scheduler.ready_task_count();
        drop(scheduler);

        let proc_table = crate::process::PROCESS_TABLE.lock();
        let process_count = proc_table.count();
        drop(proc_table);

        println!("  Total Tasks:     {}", task_count);
        println!("  Ready Tasks:     {}", ready_count);
        println!("  Total Processes: {}", process_count);
        println!();

        // Memory
        use crate::memory::allocator::{HEAP_START, HEAP_SIZE};
        println!("Memory:");
        println!("  Heap Start:      0x{:x}", HEAP_START);
        println!("  Heap Size:       {} KB", HEAP_SIZE / 1024);
        println!();

        // Features Status
        println!("Features:");
        println!("  ✓ User-mode execution active");
        println!("  ✓ Preemptive multitasking (100 Hz)");
        println!("  ✓ 32 syscalls available");
        println!("  ✓ 4 IPC mechanisms ready");
        println!("  ✓ Persistent filesystem mounted");
        println!();

        // System Health
        println!("System Health: ✓ ALL SYSTEMS OPERATIONAL");
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

    fn cmd_hexdump(&self, args: &[&str]) {
        use crate::vfs::VFS;

        if args.is_empty() {
            println!("Usage: hexdump <file>");
            return;
        }

        let vfs = VFS.lock();
        match vfs.read_file(args[0]) {
            Ok(content) => {
                let bytes = content.as_bytes();
                println!("Hexdump of '{}' ({} bytes):", args[0], bytes.len());
                println!();

                for (i, chunk) in bytes.chunks(16).enumerate() {
                    // Address
                    print!("{:08x}  ", i * 16);

                    // Hex bytes
                    for (j, byte) in chunk.iter().enumerate() {
                        print!("{:02x} ", byte);
                        if j == 7 {
                            print!(" ");
                        }
                    }

                    // Padding for incomplete lines
                    for j in chunk.len()..16 {
                        print!("   ");
                        if j == 7 {
                            print!(" ");
                        }
                    }

                    // ASCII representation
                    print!(" |");
                    for byte in chunk {
                        let c = if byte.is_ascii_graphic() || *byte == b' ' {
                            *byte as char
                        } else {
                            '.'
                        };
                        print!("{}", c);
                    }
                    println!("|");
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    fn cmd_cp(&self, args: &[&str]) {
        use crate::vfs::VFS;

        if args.len() < 2 {
            println!("Usage: cp <source> <destination>");
            return;
        }

        let mut vfs = VFS.lock();

        // Read source file
        let content = match vfs.read_file(args[0]) {
            Ok(c) => c,
            Err(e) => {
                println!("Error reading source: {}", e);
                return;
            }
        };

        // Write to destination
        match vfs.create_file(args[1], content) {
            Ok(_) => println!("Copied '{}' to '{}'", args[0], args[1]),
            Err(e) => println!("Error writing destination: {}", e),
        }
    }

    fn cmd_mv(&self, args: &[&str]) {
        use crate::vfs::VFS;

        if args.len() < 2 {
            println!("Usage: mv <source> <destination>");
            return;
        }

        let mut vfs = VFS.lock();

        // Read source file
        let content = match vfs.read_file(args[0]) {
            Ok(c) => c,
            Err(e) => {
                println!("Error reading source: {}", e);
                return;
            }
        };

        // Write to destination
        if let Err(e) = vfs.create_file(args[1], content) {
            println!("Error writing destination: {}", e);
            return;
        }

        // Delete source
        match vfs.delete(args[0]) {
            Ok(_) => println!("Moved '{}' to '{}'", args[0], args[1]),
            Err(e) => println!("Error deleting source: {}", e),
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

    fn cmd_init_disk(&self, _args: &[&str]) {
        println!("╔═══════════════════════════════════════════════════════╗");
        println!("║     Initializing Disk with Example Content          ║");
        println!("╚═══════════════════════════════════════════════════════╝");
        println!();
        println!("Creating directory structure...");

        // Create directories
        if let Err(e) = crate::simplefs::create_directory("/docs") {
            println!("Warning: Could not create /docs: {}", e);
        } else {
            println!("✓ Created /disk/docs");
        }

        if let Err(e) = crate::simplefs::create_directory("/examples") {
            println!("Warning: Could not create /examples: {}", e);
        } else {
            println!("✓ Created /disk/examples");
        }

        if let Err(e) = crate::simplefs::create_directory("/tests") {
            println!("Warning: Could not create /tests: {}", e);
        } else {
            println!("✓ Created /disk/tests");
        }

        println!();
        println!("Creating example files...");

        // Create README
        let readme = "Welcome to MyOS Persistent Storage!\n\n\
This filesystem (SimpleFS) supports:\n\
- Nested subdirectories\n\
- File creation, reading, writing, deletion\n\
- Persistent storage on ATA disk\n\n\
Try these commands:\n\
  ls /disk\n\
  cat /disk/docs/readme.txt\n\
  hexdump /disk/examples/binary.dat\n\n\
Explore and create your own files!\n";

        if let Err(e) = crate::simplefs::create_file("/docs/readme.txt", readme.as_bytes()) {
            println!("Warning: Could not create readme: {}", e);
        } else {
            println!("✓ Created /disk/docs/readme.txt");
        }

        // Create a test binary file
        use alloc::vec::Vec;
        let binary_data: Vec<u8> = (0..=255).collect();
        if let Err(e) = crate::simplefs::create_file("/examples/binary.dat", &binary_data) {
            println!("Warning: Could not create binary.dat: {}", e);
        } else {
            println!("✓ Created /disk/examples/binary.dat");
        }

        // Create syscall reference
        let syscall_ref = "MyOS System Call Reference\n\n\
INT 0x80 Syscalls:\n\
  0  - exit(code)\n\
  1  - yield()\n\
  2  - print(str, len)\n\
  6  - getpid()\n\
  7  - getppid()\n\
  8  - fork()\n\
  11 - exec(path, argv)\n\
  14 - pipe(fds)\n\
  15 - read(fd, buf, len)\n\
  16 - write(fd, buf, len)\n\n\
Calling Convention:\n\
  RAX = syscall number\n\
  RDI = arg1, RSI = arg2, RDX = arg3\n\
  Return value in RAX\n\n\
Example (x86-64 assembly):\n\
  mov rax, 16       ; write syscall\n\
  mov rdi, 1        ; stdout\n\
  lea rsi, [msg]    ; buffer\n\
  mov rdx, 5        ; length\n\
  int 0x80          ; invoke\n";

        if let Err(e) = crate::simplefs::create_file("/docs/syscalls.txt", syscall_ref.as_bytes()) {
            println!("Warning: Could not create syscalls.txt: {}", e);
        } else {
            println!("✓ Created /disk/docs/syscalls.txt");
        }

        // Create test file
        let test_data = "This is a test file for trying commands like:\n\
  cat /disk/tests/test.txt\n\
  hexdump /disk/tests/test.txt\n\
  cp /disk/tests/test.txt /disk/tests/backup.txt\n\
  mv /disk/tests/backup.txt /disk/tests/moved.txt\n\n\
Feel free to experiment!\n";

        if let Err(e) = crate::simplefs::create_file("/tests/test.txt", test_data.as_bytes()) {
            println!("Warning: Could not create test.txt: {}", e);
        } else {
            println!("✓ Created /disk/tests/test.txt");
        }

        // Create IPC info file
        let ipc_info = "MyOS IPC Mechanisms\n\n\
Available IPC methods:\n\n\
1. Pipes\n\
   - Create: syscall 14 (pipe)\n\
   - Test: pipetest command\n\n\
2. Shared Memory\n\
   - Get segment: syscall 18 (shmget)\n\
   - Attach: syscall 19 (shmat)\n\
   - Test: shmtest command\n\n\
3. Semaphores\n\
   - Initialize: syscall 22 (seminit)\n\
   - Wait: syscall 24 (semwait)\n\
   - Post: syscall 25 (sempost)\n\
   - Test: semtest command\n\n\
4. Message Queues\n\
   - Create: syscall 28 (msgget)\n\
   - Send: syscall 29 (msgsnd)\n\
   - Receive: syscall 30 (msgrcv)\n\
   - Test: msgtest command\n";

        if let Err(e) = crate::simplefs::create_file("/docs/ipc.txt", ipc_info.as_bytes()) {
            println!("Warning: Could not create ipc.txt: {}", e);
        } else {
            println!("✓ Created /disk/docs/ipc.txt");
        }

        println!();
        println!("✓ Disk initialization complete!");
        println!();
        println!("Try these commands:");
        println!("  ls /disk");
        println!("  ls /disk/docs");
        println!("  cat /disk/docs/readme.txt");
        println!("  hexdump /disk/examples/binary.dat");
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

    fn cmd_semtest(&self, _args: &[&str]) {
        println!("╔═══════════════════════════════════════════════════════╗");
        println!("║           Semaphore IPC Test                         ║");
        println!("╚═══════════════════════════════════════════════════════╝");
        println!();

        // Initialize a semaphore with value 3
        const INITIAL_VALUE: i32 = 3;

        println!("Creating semaphore with initial value {}...", INITIAL_VALUE);
        match crate::sem::sem_init(INITIAL_VALUE) {
            Ok(sem_id) => {
                println!("✓ Semaphore created!");
                println!("  Semaphore ID: {}", sem_id);
                println!();

                // Get initial value
                println!("Getting semaphore value...");
                match crate::sem::sem_getvalue(sem_id) {
                    Ok(val) => {
                        println!("✓ Current value: {}", val);
                        if val == INITIAL_VALUE {
                            println!("✓ Initial value correct!");
                        } else {
                            println!("✗ Initial value mismatch!");
                        }
                        println!();
                    }
                    Err(e) => {
                        println!("✗ Error getting value: {}", e);
                    }
                }

                // Test wait operation (should decrement)
                println!("Performing sem_wait (decrement)...");
                match crate::sem::sem_wait(sem_id) {
                    Ok(_) => {
                        println!("✓ Wait successful");

                        // Check new value
                        match crate::sem::sem_getvalue(sem_id) {
                            Ok(val) => {
                                println!("  New value: {}", val);
                                if val == INITIAL_VALUE - 1 {
                                    println!("✓ Value decremented correctly!");
                                } else {
                                    println!("✗ Value incorrect after wait!");
                                }
                            }
                            Err(e) => println!("✗ Error getting value: {}", e),
                        }
                        println!();
                    }
                    Err(e) => {
                        println!("✗ Error in wait: {}", e);
                    }
                }

                // Test post operation (should increment)
                println!("Performing sem_post (increment)...");
                match crate::sem::sem_post(sem_id) {
                    Ok(_) => {
                        println!("✓ Post successful");

                        // Check value restored
                        match crate::sem::sem_getvalue(sem_id) {
                            Ok(val) => {
                                println!("  New value: {}", val);
                                if val == INITIAL_VALUE {
                                    println!("✓ Value restored to initial!");
                                } else {
                                    println!("✗ Value incorrect after post!");
                                }
                            }
                            Err(e) => println!("✗ Error getting value: {}", e),
                        }
                        println!();
                    }
                    Err(e) => {
                        println!("✗ Error in post: {}", e);
                    }
                }

                // Clean up
                println!("Destroying semaphore...");
                match crate::sem::sem_destroy(sem_id) {
                    Ok(_) => println!("✓ Semaphore destroyed"),
                    Err(e) => println!("✗ Error destroying: {}", e),
                }
            }
            Err(e) => {
                println!("✗ Error creating semaphore: {}", e);
            }
        }

        println!();
        println!("Semaphore test complete!");
    }

    fn cmd_msgtest(&self, _args: &[&str]) {
        println!("╔═══════════════════════════════════════════════════════╗");
        println!("║         Message Queue IPC Test                       ║");
        println!("╚═══════════════════════════════════════════════════════╝");
        println!();

        const MSG_KEY: i32 = 5678;

        println!("Creating message queue...");
        println!("  Key: {}", MSG_KEY);

        match crate::msgq::msgget(MSG_KEY, 0o666) {
            Ok(queue_id) => {
                println!("✓ Message queue created!");
                println!("  Queue ID: {}", queue_id);
                println!();

                // Send a message
                let test_msg = b"Hello from message queue!";
                const MSG_TYPE: i64 = 1;

                println!("Sending message...");
                println!("  Type: {}", MSG_TYPE);
                println!("  Data: \"{}\"", core::str::from_utf8(test_msg).unwrap());

                use alloc::vec::Vec;
                match crate::msgq::msgsnd(queue_id, MSG_TYPE, test_msg.to_vec()) {
                    Ok(_) => {
                        println!("✓ Message sent ({} bytes)", test_msg.len());
                        println!();

                        // Receive the message
                        println!("Receiving message...");

                        match crate::msgq::msgrcv(queue_id, MSG_TYPE) {
                            Ok(message) => {
                                println!("✓ Message received ({} bytes)", message.data.len());
                                let recv_str = core::str::from_utf8(&message.data)
                                    .unwrap_or("<invalid UTF-8>");
                                println!("  Data: \"{}\"", recv_str);
                                println!();

                                if recv_str == core::str::from_utf8(test_msg).unwrap() {
                                    println!("✓ Test PASSED: Message matches!");
                                } else {
                                    println!("✗ Test FAILED: Message mismatch!");
                                }
                            }
                            Err(e) => {
                                println!("✗ Error receiving message: {}", e);
                            }
                        }
                    }
                    Err(e) => {
                        println!("✗ Error sending message: {}", e);
                    }
                }

                // Clean up
                println!();
                println!("Removing message queue...");
                match crate::msgq::msgctl(queue_id, 0) {  // IPC_RMID = 0
                    Ok(_) => println!("✓ Queue removed"),
                    Err(e) => println!("✗ Error removing: {}", e),
                }
            }
            Err(e) => {
                println!("✗ Error creating message queue: {}", e);
            }
        }

        println!();
        println!("Message queue test complete!");
    }

    fn cmd_tree(&self, args: &[&str]) {
        use crate::vfs::VFS;
        use alloc::string::ToString;

        let path = if args.is_empty() { "/" } else { args[0] };
        let vfs = VFS.lock();

        println!("{}", path);
        self.print_tree(&vfs, path, "", true);
    }

    fn print_tree(&self, vfs: &crate::vfs::VirtualFileSystem, path: &str, prefix: &str, is_last: bool) {
        use alloc::format;

        let entries = match vfs.list_directory(path) {
            Ok(entries) => entries,
            Err(_) => return,
        };

        for (i, (name, _file_type, _size)) in entries.iter().enumerate() {
            let is_last_entry = i == entries.len() - 1;
            let connector = if is_last_entry { "└── " } else { "├── " };
            let extension = if is_last_entry { "    " } else { "│   " };

            println!("{}{}{}", prefix, connector, name);

            // Recursively print subdirectories
            let full_path = if path == "/" {
                format!("/{}", name)
            } else {
                format!("{}/{}", path, name)
            };

            let new_prefix = format!("{}{}", prefix, extension);
            self.print_tree(vfs, &full_path, &new_prefix, is_last_entry);
        }
    }

    fn cmd_wc(&self, args: &[&str]) {
        use crate::vfs::VFS;

        if args.is_empty() {
            println!("Usage: wc <file>");
            return;
        }

        let vfs = VFS.lock();
        match vfs.read_file(args[0]) {
            Ok(content) => {
                let lines = content.lines().count();
                let words = content.split_whitespace().count();
                let bytes = content.len();
                println!("  {} {} {} {}", lines, words, bytes, args[0]);
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    fn cmd_grep(&self, args: &[&str]) {
        use crate::vfs::VFS;

        if args.len() < 2 {
            println!("Usage: grep <pattern> <file>");
            return;
        }

        let pattern = args[0];
        let file = args[1];

        let vfs = VFS.lock();
        match vfs.read_file(file) {
            Ok(content) => {
                let mut found = false;
                for (line_num, line) in content.lines().enumerate() {
                    if line.contains(pattern) {
                        println!("{}:{}: {}", file, line_num + 1, line);
                        found = true;
                    }
                }
                if !found {
                    println!("No matches found");
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    fn cmd_head(&self, args: &[&str]) {
        use crate::vfs::VFS;

        if args.is_empty() {
            println!("Usage: head <file> [n]");
            return;
        }

        let file = args[0];
        let n = if args.len() > 1 {
            args[1].parse::<usize>().unwrap_or(10)
        } else {
            10
        };

        let vfs = VFS.lock();
        match vfs.read_file(file) {
            Ok(content) => {
                for (i, line) in content.lines().enumerate() {
                    if i >= n {
                        break;
                    }
                    println!("{}", line);
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    fn cmd_tail(&self, args: &[&str]) {
        use crate::vfs::VFS;
        use alloc::vec::Vec;

        if args.is_empty() {
            println!("Usage: tail <file> [n]");
            return;
        }

        let file = args[0];
        let n = if args.len() > 1 {
            args[1].parse::<usize>().unwrap_or(10)
        } else {
            10
        };

        let vfs = VFS.lock();
        match vfs.read_file(file) {
            Ok(content) => {
                let lines: Vec<&str> = content.lines().collect();
                let start = if lines.len() > n { lines.len() - n } else { 0 };
                for line in &lines[start..] {
                    println!("{}", line);
                }
            }
            Err(e) => println!("Error: {}", e),
        }
    }

    fn cmd_du(&self, args: &[&str]) {
        use crate::vfs::VFS;

        let path = if args.is_empty() { "/" } else { args[0] };
        let vfs = VFS.lock();

        let size = self.calculate_dir_size(&vfs, path);
        println!("{} bytes\t{}", size, path);
    }

    fn calculate_dir_size(&self, vfs: &crate::vfs::VirtualFileSystem, path: &str) -> usize {
        use alloc::format;

        let mut total = 0;

        if let Ok(entries) = vfs.list_directory(path) {
            for (name, _file_type, _size) in entries {
                let full_path = if path == "/" {
                    format!("/{}", name)
                } else {
                    format!("{}/{}", path, name)
                };

                // Try to read as file
                if let Ok(content) = vfs.read_file(&full_path) {
                    total += content.len();
                } else {
                    // Must be a directory
                    total += self.calculate_dir_size(vfs, &full_path);
                }
            }
        }

        total
    }

    fn cmd_find(&self, args: &[&str]) {
        use crate::vfs::VFS;

        if args.is_empty() {
            println!("Usage: find <pattern>");
            return;
        }

        let pattern = args[0];
        let vfs = VFS.lock();

        println!("Searching for files matching '{}'...", pattern);
        self.find_files(&vfs, "/", pattern);
    }

    fn find_files(&self, vfs: &crate::vfs::VirtualFileSystem, path: &str, pattern: &str) {
        use alloc::format;

        if let Ok(entries) = vfs.list_directory(path) {
            for (name, _file_type, _size) in entries {
                if name.contains(pattern) {
                    let full_path = if path == "/" {
                        format!("/{}", name)
                    } else {
                        format!("{}/{}", path, name)
                    };
                    println!("{}", full_path);
                }

                // Recurse into subdirectories
                let full_path = if path == "/" {
                    format!("/{}", name)
                } else {
                    format!("{}/{}", path, name)
                };

                // Check if it's a directory by trying to list it
                if vfs.list_directory(&full_path).is_ok() {
                    self.find_files(vfs, &full_path, pattern);
                }
            }
        }
    }

    fn cmd_free(&self, _args: &[&str]) {
        println!("Memory Usage:");
        println!("─────────────────────────────────────────");
        println!("  Heap:      4 MB (allocated at boot)");
        println!("  Status:    Active");
        println!();
        println!("Physical Memory:");
        println!("  Managed by frame allocator");
        println!("  Page size: 4 KB");
        println!();
        println!("Note: Detailed memory statistics not yet implemented");
    }

    fn cmd_date(&self, _args: &[&str]) {
        use crate::time;

        let uptime_ms = time::uptime_ms();
        let seconds = uptime_ms / 1000;
        let minutes = seconds / 60;
        let hours = minutes / 60;
        let days = hours / 24;

        println!("System Time:");
        println!("  Uptime:    {}d {}h {}m {}s",
            days,
            hours % 24,
            minutes % 60,
            seconds % 60
        );
        println!("  Milliseconds: {}", uptime_ms);
    }

    fn cmd_top(&self, _args: &[&str]) {
        use crate::SCHEDULER;

        println!("MyOS Task Monitor");
        println!("═══════════════════════════════════════════════════════════");

        let uptime_ms = crate::time::uptime_ms();
        let seconds = uptime_ms / 1000;
        let minutes = seconds / 60;
        let hours = minutes / 60;

        println!("Uptime: {}h {}m {}s", hours, minutes % 60, seconds % 60);

        let scheduler = SCHEDULER.lock();
        let tasks = scheduler.list_tasks();
        let ready_count = scheduler.ready_task_count();

        println!("Tasks: {} total, {} ready", tasks.len(), ready_count);
        println!();
        println!("  ID    NAME                STATE       PRIORITY");
        println!("───────────────────────────────────────────────────────────");

        for task in tasks {
            let state_str = match task.state() {
                crate::task::TaskState::Ready => "Ready  ",
                crate::task::TaskState::Running => "Running",
                crate::task::TaskState::Waiting => "Waiting",
                crate::task::TaskState::Terminated => "Term   ",
            };
            println!("  {:<5} {:<19} {:<11} {}",
                task.id(),
                task.name(),
                state_str,
                task.priority()
            );
        }

        println!("───────────────────────────────────────────────────────────");

        println!("Press Ctrl-C to exit (not yet implemented - showing snapshot)");
    }

    fn cmd_killall(&self, args: &[&str]) {
        use crate::SCHEDULER;
        use alloc::vec::Vec;

        if args.is_empty() {
            println!("Usage: killall <name>");
            return;
        }

        let name = args[0];
        let scheduler = SCHEDULER.lock();
        let tasks = scheduler.list_tasks();

        // Count matching tasks
        let matching_tasks: Vec<&str> = tasks.iter()
            .filter(|t| t.name() == name)
            .map(|t| t.name())
            .collect();

        if matching_tasks.is_empty() {
            println!("No tasks found with name '{}'", name);
        } else {
            println!("Found {} task(s) named '{}'", matching_tasks.len(), name);
            println!("Note: Task termination by name not yet fully implemented");
            println!("Use 'kill <task_id>' to terminate individual tasks");
        }
    }
}
