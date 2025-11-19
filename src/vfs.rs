use alloc::string::{String, ToString};
use alloc::vec::Vec;
use alloc::collections::BTreeMap;
use alloc::format;
use spin::Mutex;
use lazy_static::lazy_static;

#[derive(Debug, Clone)]
pub enum FileType {
    File,
    Directory,
}

#[derive(Debug, Clone)]
pub struct FileNode {
    pub name: String,
    pub file_type: FileType,
    pub content: String,
    pub size: usize,
    pub created: u64,
    pub modified: u64,
}

impl FileNode {
    pub fn new_file(name: String, content: String) -> Self {
        let size = content.len();
        let time = crate::time::uptime_ms();
        FileNode {
            name,
            file_type: FileType::File,
            content,
            size,
            created: time,
            modified: time,
        }
    }

    pub fn new_directory(name: String) -> Self {
        let time = crate::time::uptime_ms();
        FileNode {
            name,
            file_type: FileType::Directory,
            content: String::new(),
            size: 0,
            created: time,
            modified: time,
        }
    }
}

pub struct VirtualFileSystem {
    root: BTreeMap<String, FileNode>,
    current_dir: String,
}

impl VirtualFileSystem {
    pub fn new() -> Self {
        let mut vfs = VirtualFileSystem {
            root: BTreeMap::new(),
            current_dir: String::from("/"),
        };

        // Create root directory structure
        vfs.root.insert(String::from("/"), FileNode::new_directory(String::from("/")));
        vfs.root.insert(String::from("/home"), FileNode::new_directory(String::from("home")));
        vfs.root.insert(String::from("/scripts"), FileNode::new_directory(String::from("scripts")));
        vfs.root.insert(String::from("/tmp"), FileNode::new_directory(String::from("tmp")));

        // Create a welcome file
        let welcome = "Welcome to MyOS!\n\nThis is a bare-metal operating system with:\n- HAL Script programming language\n- Interactive shell\n- Virtual file system\n- AI integration\n\nType 'help' for available commands.\nTry 'exec /scripts/fibonacci.hal' to run an example!\n";
        vfs.root.insert(
            String::from("/home/welcome.txt"),
            FileNode::new_file(String::from("welcome.txt"), String::from(welcome))
        );

        // Create example HAL scripts
        let fib_script = "# Fibonacci sequence generator\nfn fib(n) {\n    if n < 2 {\n        return n\n    }\n    return fib(n-1) + fib(n-2)\n}\n\nprint \"Fibonacci sequence:\"\nfor i in 0..15 {\n    print fib(i)\n}\n";
        vfs.root.insert(
            String::from("/scripts/fibonacci.hal"),
            FileNode::new_file(String::from("fibonacci.hal"), String::from(fib_script))
        );

        let primes_script = "# Prime number generator\nfn is_prime(n) {\n    if n < 2 {\n        return false\n    }\n    i = 2\n    while i * i <= n {\n        if n % i == 0 {\n            return false\n        }\n        i = i + 1\n    }\n    return true\n}\n\nprint \"Prime numbers under 100:\"\nfor num in 2..100 {\n    if is_prime(num) {\n        print num\n    }\n}\n";
        vfs.root.insert(
            String::from("/scripts/primes.hal"),
            FileNode::new_file(String::from("primes.hal"), String::from(primes_script))
        );

        let factorial_script = "# Factorial calculator\nfn factorial(n) {\n    if n <= 1 {\n        return 1\n    }\n    return n * factorial(n - 1)\n}\n\nprint \"Factorials:\"\nfor i in 0..11 {\n    result = factorial(i)\n    print str(i) + \"! = \" + str(result)\n}\n";
        vfs.root.insert(
            String::from("/scripts/factorial.hal"),
            FileNode::new_file(String::from("factorial.hal"), String::from(factorial_script))
        );

        let fizzbuzz_script = "# FizzBuzz\nprint \"FizzBuzz:\"\nfor i in 1..101 {\n    if i % 15 == 0 {\n        print \"FizzBuzz\"\n    } else if i % 3 == 0 {\n        print \"Fizz\"\n    } else if i % 5 == 0 {\n        print \"Buzz\"\n    } else {\n        print i\n    }\n}\n";
        vfs.root.insert(
            String::from("/scripts/fizzbuzz.hal"),
            FileNode::new_file(String::from("fizzbuzz.hal"), String::from(fizzbuzz_script))
        );

        let arrays_script = "# Array operations demo\narr = [5, 2, 8, 1, 9, 3]\n\nprint \"Array: \" + str(arr)\nprint \"Length: \" + str(len(arr))\nprint \"Sum: \" + str(sum(arr))\n\n# Find min and max\nmin_val = arr[0]\nmax_val = arr[0]\nfor i in 0..len(arr) {\n    min_val = min(min_val, arr[i])\n    max_val = max(max_val, arr[i])\n}\nprint \"Min: \" + str(min_val)\nprint \"Max: \" + str(max_val)\n";
        vfs.root.insert(
            String::from("/scripts/arrays.hal"),
            FileNode::new_file(String::from("arrays.hal"), String::from(arrays_script))
        );

        // Create /lib directory for library modules
        vfs.root.insert(String::from("/lib"), FileNode::new_directory(String::from("lib")));

        // Create /apps directory for user applications
        vfs.root.insert(String::from("/apps"), FileNode::new_directory(String::from("apps")));

        // Math library
        let math_lib = "# Math Library\n\nfn factorial(n) {\n    if n <= 1 {\n        return 1\n    }\n    return n * factorial(n - 1)\n}\n\nfn gcd(a, b) {\n    while b != 0 {\n        temp = b\n        b = a % b\n        a = temp\n    }\n    return abs(a)\n}\n\nfn lcm(a, b) {\n    return abs(a * b) / gcd(a, b)\n}\n\nfn is_prime(n) {\n    if n < 2 {\n        return false\n    }\n    i = 2\n    while i * i <= n {\n        if n % i == 0 {\n            return false\n        }\n        i = i + 1\n    }\n    return true\n}\n";
        vfs.root.insert(
            String::from("/lib/math.hal"),
            FileNode::new_file(String::from("math.hal"), String::from(math_lib))
        );

        // String utilities library
        let string_lib = "# String Utilities Library\n\nfn reverse_string(s) {\n    chars = split(s, \"\")\n    reversed = reverse(chars)\n    return join(reversed, \"\")\n}\n\nfn count_words(s) {\n    trimmed = trim(s)\n    if len(trimmed) == 0 {\n        return 0\n    }\n    words = split(trimmed, \" \")\n    return len(words)\n}\n\nfn title_case(s) {\n    words = split(s, \" \")\n    result = []\n    for word in words {\n        if len(word) > 0 {\n            first = upper(substring(word, 0, 1))\n            rest = lower(substring(word, 1, len(word)))\n            push(result, first + rest)\n        }\n    }\n    return join(result, \" \")\n}\n";
        vfs.root.insert(
            String::from("/lib/string.hal"),
            FileNode::new_file(String::from("string.hal"), String::from(string_lib))
        );

        // Example app using libraries
        let demo_app = "# Demo Application\nimport \"/lib/math.hal\"\nimport \"/lib/string.hal\"\n\nprint \"=== Math Library Demo ===\"\nprint \"Factorial of 5: \" + str(factorial(5))\nprint \"GCD of 48 and 18: \" + str(gcd(48, 18))\nprint \"LCM of 12 and 15: \" + str(lcm(12, 15))\nprint \"Is 17 prime? \" + str(is_prime(17))\nprint \"Is 20 prime? \" + str(is_prime(20))\n\nprint \"\"\nprint \"=== String Library Demo ===\"\ntext = \"hello world\"\nprint \"Original: \" + text\nprint \"Reversed: \" + reverse_string(text)\nprint \"Title case: \" + title_case(text)\nprint \"Word count: \" + str(count_words(text))\n";
        vfs.root.insert(
            String::from("/scripts/demo_app.hal"),
            FileNode::new_file(String::from("demo_app.hal"), String::from(demo_app))
        );

        // Calculator app
        let calc_app = "# Calculator App\n# Usage: run with arguments like: 10 + 5\n\nif len(args) < 3 {\n    print \"Calculator App v1.0\"\n    print \"Usage: <number> <operator> <number>\"\n    print \"Operators: + - * / %\"\n    print \"Example: 10 + 5\"\n} else {\n    a = num(args[0])\n    op = args[1]\n    b = num(args[2])\n    \n    if op == \"+\" {\n        result = a + b\n    } else if op == \"-\" {\n        result = a - b\n    } else if op == \"*\" {\n        result = a * b\n    } else if op == \"/\" {\n        if b == 0 {\n            print \"Error: Division by zero\"\n        } else {\n            result = a / b\n        }\n    } else if op == \"%\" {\n        result = a % b\n    } else {\n        print \"Error: Unknown operator \" + op\n    }\n    \n    if result != null {\n        print str(a) + \" \" + op + \" \" + str(b) + \" = \" + str(result)\n    }\n}\n";
        vfs.root.insert(
            String::from("/apps/calc.hal"),
            FileNode::new_file(String::from("calc.hal"), String::from(calc_app))
        );

        // Greeter app
        let greeter_app = "# Greeter App\n# Usage: run with your name as argument\n\nimport \"/lib/string.hal\"\n\nif len(args) == 0 {\n    print \"Hello, stranger!\"\n    print \"Try running with your name: greeter YourName\"\n} else {\n    name = join(args, \" \")\n    nice_name = title_case(name)\n    print \"Hello, \" + nice_name + \"!\"\n    print \"Your name has \" + str(len(name)) + \" characters.\"\n    print \"Word count: \" + str(count_words(name))\n}\n";
        vfs.root.insert(
            String::from("/apps/greeter.hal"),
            FileNode::new_file(String::from("greeter.hal"), String::from(greeter_app))
        );

        // File manager app
        let filemgr_app = "# File Manager App\n# Simple file browser and manager\n\nif len(args) == 0 {\n    print \"File Manager v1.0\"\n    print \"Commands:\"\n    print \"  list <path>     - List directory contents\"\n    print \"  read <file>     - Read file contents\"\n    print \"  create <file>   - Create empty file\"\n    print \"Example: filemgr list /scripts\"\n} else {\n    cmd = args[0]\n    \n    if cmd == \"list\" {\n        path = if len(args) > 1 { args[1] } else { \"/\" }\n        files = list_dir(path)\n        print \"Contents of \" + path + \":\"\n        for file in files {\n            print \"  \" + file\n        }\n        print \"Total: \" + str(len(files)) + \" items\"\n    } else if cmd == \"read\" {\n        if len(args) < 2 {\n            print \"Error: Please specify a file\"\n        } else {\n            path = args[1]\n            if file_exists(path) {\n                content = read_file(path)\n                print \"=== \" + path + \" ===\"\n                print content\n            } else {\n                print \"Error: File not found: \" + path\n            }\n        }\n    } else if cmd == \"create\" {\n        if len(args) < 2 {\n            print \"Error: Please specify a file\"\n        } else {\n            path = args[1]\n            write_file(path, \"\")\n            print \"Created: \" + path\n        }\n    } else {\n        print \"Error: Unknown command: \" + cmd\n    }\n}\n";
        vfs.root.insert(
            String::from("/apps/filemgr.hal"),
            FileNode::new_file(String::from("filemgr.hal"), String::from(filemgr_app))
        );

        // Prime finder app
        let prime_app = "# Prime Finder App\nimport \"/lib/math.hal\"\n\nif len(args) == 0 {\n    print \"Prime Finder v1.0\"\n    print \"Usage: <max_number>\"\n    print \"Example: primefind 100\"\n} else {\n    max_num = num(args[0])\n    print \"Prime numbers up to \" + str(max_num) + \":\"\n    \n    count = 0\n    for n in 2..max_num {\n        if is_prime(n) {\n            print n\n            count = count + 1\n        }\n    }\n    \n    print \"\"\n    print \"Found \" + str(count) + \" prime numbers\"\n}\n";
        vfs.root.insert(
            String::from("/apps/primefind.hal"),
            FileNode::new_file(String::from("primefind.hal"), String::from(prime_app))
        );

        vfs
    }

    fn normalize_path(&self, path: &str) -> String {
        if path.starts_with('/') {
            String::from(path)
        } else {
            if self.current_dir == "/" {
                format!("/{}", path)
            } else {
                format!("{}/{}", self.current_dir, path)
            }
        }
    }

    pub fn create_file(&mut self, path: &str, content: String) -> Result<(), String> {
        let full_path = self.normalize_path(path);

        if self.root.contains_key(&full_path) {
            return Err(format!("File already exists: {}", path));
        }

        // Extract filename
        let filename = String::from(full_path.rsplit('/').next().unwrap_or(path));

        self.root.insert(
            full_path,
            FileNode::new_file(filename, content)
        );

        Ok(())
    }

    pub fn create_directory(&mut self, path: &str) -> Result<(), String> {
        let full_path = self.normalize_path(path);

        if self.root.contains_key(&full_path) {
            return Err(format!("Directory already exists: {}", path));
        }

        let dirname = String::from(full_path.rsplit('/').next().unwrap_or(path));

        self.root.insert(
            full_path,
            FileNode::new_directory(dirname)
        );

        Ok(())
    }

    pub fn read_file(&self, path: &str) -> Result<String, String> {
        let full_path = self.normalize_path(path);

        match self.root.get(&full_path) {
            Some(node) => match node.file_type {
                FileType::File => Ok(node.content.clone()),
                FileType::Directory => Err(format!("{} is a directory", path)),
            },
            None => Err(format!("File not found: {}", path)),
        }
    }

    pub fn write_file(&mut self, path: &str, content: String) -> Result<(), String> {
        let full_path = self.normalize_path(path);

        match self.root.get_mut(&full_path) {
            Some(node) => match node.file_type {
                FileType::File => {
                    node.content = content.clone();
                    node.size = content.len();
                    node.modified = crate::time::uptime_ms();
                    Ok(())
                }
                FileType::Directory => Err(format!("{} is a directory", path)),
            },
            None => {
                // Create new file
                self.create_file(path, content)
            }
        }
    }

    pub fn delete(&mut self, path: &str) -> Result<(), String> {
        let full_path = self.normalize_path(path);

        if full_path == "/" {
            return Err(String::from("Cannot delete root directory"));
        }

        // Check if it's a directory with contents
        let is_dir = self.root.get(&full_path)
            .map(|n| matches!(n.file_type, FileType::Directory))
            .unwrap_or(false);

        if is_dir {
            let has_contents = self.root.keys()
                .any(|k| k.starts_with(&full_path) && k != &full_path);

            if has_contents {
                return Err(format!("Directory not empty: {}", path));
            }
        }

        self.root.remove(&full_path)
            .ok_or(format!("File not found: {}", path))?;

        Ok(())
    }

    pub fn list_directory(&self, path: &str) -> Result<Vec<(String, FileType, usize)>, String> {
        let full_path = self.normalize_path(path);

        // Check if directory exists
        match self.root.get(&full_path) {
            Some(node) => match node.file_type {
                FileType::Directory => {},
                FileType::File => return Err(format!("{} is not a directory", path)),
            },
            None => return Err(format!("Directory not found: {}", path)),
        }

        let prefix = if full_path == "/" {
            String::from("/")
        } else {
            format!("{}/", full_path)
        };

        let mut entries = Vec::new();

        for (path, node) in &self.root {
            if path == &full_path {
                continue;
            }

            if path.starts_with(&prefix) {
                let remaining = &path[prefix.len()..];
                // Only direct children (no further slashes)
                if !remaining.contains('/') && !remaining.is_empty() {
                    entries.push((
                        remaining.to_string(),
                        node.file_type.clone(),
                        node.size,
                    ));
                }
            }
        }

        Ok(entries)
    }

    pub fn get_current_dir(&self) -> &str {
        &self.current_dir
    }

    pub fn change_directory(&mut self, path: &str) -> Result<(), String> {
        let full_path = self.normalize_path(path);

        match self.root.get(&full_path) {
            Some(node) => match node.file_type {
                FileType::Directory => {
                    self.current_dir = full_path;
                    Ok(())
                }
                FileType::File => Err(format!("{} is not a directory", path)),
            },
            None => Err(format!("Directory not found: {}", path)),
        }
    }

    pub fn file_exists(&self, path: &str) -> bool {
        let full_path = self.normalize_path(path);
        self.root.contains_key(&full_path)
    }

    pub fn get_info(&self, path: &str) -> Result<&FileNode, String> {
        let full_path = self.normalize_path(path);
        self.root.get(&full_path)
            .ok_or(format!("File not found: {}", path))
    }
}

lazy_static! {
    pub static ref VFS: Mutex<VirtualFileSystem> = Mutex::new(VirtualFileSystem::new());
}
