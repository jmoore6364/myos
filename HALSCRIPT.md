# HAL Script Language Reference

HAL Script is a simple, powerful embedded scripting language built into MyOS. It combines the simplicity of BASIC with modern programming features.

## Features

- **Dynamic typing** - No type declarations needed
- **Functions** - First-class functions with closures
- **Control flow** - if/else, for loops, while loops
- **Arrays** - Dynamic arrays with indexing
- **String operations** - Concatenation and manipulation
- **Built-in functions** - System integration

## Syntax

### Variables

Variables are dynamically typed and don't need declaration:

```halscript
x = 10
name = "MyOS"
active = true
items = [1, 2, 3, 4, 5]
```

### Data Types

- **Numbers**: 64-bit integers (`42`, `-17`, `0`)
- **Strings**: UTF-8 strings (`"hello"`, `"world"`)
- **Booleans**: `true`, `false`
- **Arrays**: `[1, 2, 3]`
- **Null**: `null`

### Operators

**Arithmetic:**
- `+` Addition (also string concatenation)
- `-` Subtraction
- `*` Multiplication
- `/` Division
- `%` Modulo

**Comparison:**
- `==` Equal
- `!=` Not equal
- `<` Less than
- `>` Greater than
- `<=` Less than or equal
- `>=` Greater than or equal

**Logical:**
- `&&` AND
- `||` OR
- `!` NOT

### Functions

Define functions with the `fn` keyword:

```halscript
fn greet(name) {
    print "Hello, " + name
}

fn add(a, b) {
    return a + b
}

fn factorial(n) {
    if n <= 1 {
        return 1
    }
    return n * factorial(n - 1)
}
```

### Conditionals

```halscript
if x > 10 {
    print "big"
} else {
    print "small"
}

if score >= 90 {
    print "A"
} else if score >= 80 {
    print "B"
} else {
    print "F"
}
```

### Loops

**For loops** with ranges:

```halscript
for i in 0..10 {
    print i
}

for x in 1..100 {
    if x % 15 == 0 {
        print "FizzBuzz"
    } else if x % 3 == 0 {
        print "Fizz"
    } else if x % 5 == 0 {
        print "Buzz"
    } else {
        print x
    }
}
```

**While loops:**

```halscript
count = 0
while count < 5 {
    print count
    count = count + 1
}
```

### Break and Continue

**Break statement** - exits the loop early:

```halscript
# Find first even number
for i in 1..100 {
    if i % 2 == 0 {
        print "First even: " + str(i)
        break
    }
}

# Infinite loop with break
count = 0
while true {
    count = count + 1
    if count >= 10 {
        break
    }
}
```

**Continue statement** - skips to next iteration:

```halscript
# Print only odd numbers
for i in 1..11 {
    if i % 2 == 0 {
        continue
    }
    print i  # Only prints odd numbers
}

# Skip multiples of 3
count = 0
while count < 10 {
    count = count + 1
    if count % 3 == 0 {
        continue
    }
    print count
}
```

### Arrays

```halscript
# Create array
arr = [1, 2, 3, 4, 5]

# Access elements
print arr[0]  # 1
print arr[2]  # 3

# Nested arrays
matrix = [[1, 2], [3, 4], [5, 6]]
print matrix[1][0]  # 3
```

### Print Statement

```halscript
print "Hello, World!"
print 42
print x + y
print "Result: " + result
```

### Comments

Use `#` for single-line comments:

```halscript
# This is a comment
x = 10  # This is also a comment
```

## Built-in Functions

HAL Script provides 13 powerful built-in functions:

### String & Type Conversion

#### `len(value)`
Returns the length of a string or array:
```halscript
print len("hello")      # 5
print len([1, 2, 3])    # 3
```

#### `str(value)`
Converts any value to a string:
```halscript
print str(42)           # "42"
print str(true)         # "true"
print str([1, 2, 3])    # "[1, 2, 3]"
```

#### `num(value)`
Parses a number from a string or converts boolean to number:
```halscript
x = num("42")           # 42
y = num(true)           # 1
z = num(false)          # 0
```

### Math Functions

#### `abs(n)`
Returns the absolute value:
```halscript
print abs(-42)          # 42
print abs(10)           # 10
```

#### `min(a, b)`
Returns the smaller of two numbers:
```halscript
print min(5, 10)        # 5
```

#### `max(a, b)`
Returns the larger of two numbers:
```halscript
print max(5, 10)        # 10
```

#### `pow(base, exp)`
Raises base to the power of exp:
```halscript
print pow(2, 10)        # 1024
print pow(3, 3)         # 27
```

#### `sqrt(n)`
Returns integer square root using Newton's method:
```halscript
print sqrt(144)         # 12
print sqrt(100)         # 10
```

### Array Functions

#### `range(start, end)`
Creates an array of numbers from start to end (exclusive):
```halscript
arr = range(0, 5)       # [0, 1, 2, 3, 4]
arr = range(5, 10)      # [5, 6, 7, 8, 9]
```

#### `push(array, value)`
Returns a new array with value added:
```halscript
arr = [1, 2, 3]
arr2 = push(arr, 4)     # [1, 2, 3, 4]
```

#### `sum(array)`
Sums all numbers in an array:
```halscript
print sum([1, 2, 3, 4, 5])  # 15
```

### System Functions

#### `uptime()`
Returns system uptime in seconds:
```halscript
print uptime()          # 42 (seconds since boot)
```

## Running Scripts from Files

You can write HAL scripts to the VFS and execute them:

```bash
# Write a script
> write /home/test.hal "print \"Hello from file!\""

# Execute it
> exec /home/test.hal

# Pre-loaded examples
> ls /scripts
> exec /scripts/fibonacci.hal
> exec /scripts/primes.hal
```

## Example Programs

### Fibonacci Sequence

```halscript
fn fib(n) {
    if n < 2 {
        return n
    }
    return fib(n-1) + fib(n-2)
}

for i in 0..15 {
    print fib(i)
}
```

### FizzBuzz

```halscript
for n in 1..101 {
    if n % 15 == 0 {
        print "FizzBuzz"
    } else if n % 3 == 0 {
        print "Fizz"
    } else if n % 5 == 0 {
        print "Buzz"
    } else {
        print n
    }
}
```

### Prime Numbers

```halscript
fn is_prime(n) {
    if n < 2 {
        return false
    }
    i = 2
    while i * i <= n {
        if n % i == 0 {
            return false
        }
        i = i + 1
    }
    return true
}

for num in 2..100 {
    if is_prime(num) {
        print num
    }
}
```

### Array Operations

```halscript
# Sum of array
numbers = [1, 2, 3, 4, 5]
sum = 0
for i in 0..len(numbers) {
    sum = sum + numbers[i]
}
print "Sum: " + sum

# Find maximum
max = numbers[0]
for i in 1..len(numbers) {
    if numbers[i] > max {
        max = numbers[i]
    }
}
print "Max: " + max
```

### Greeting Program

```halscript
fn greet(name, age) {
    msg = "Hello, " + name + "!"
    print msg
    print "You are " + age + " years old."

    if age >= 18 {
        print "You are an adult."
    } else {
        print "You are a minor."
    }
}

greet("Alice", 25)
greet("Bob", 16)
```

## Using HAL Script in MyOS

### Running Code from Shell

```bash
# Direct execution
> run print "Hello, World!"

# Variables
> run x = 42
> run print x

# Functions
> run fn square(n) { return n * n }
> run print square(7)

# Show examples
> examples
```

### Interactive Examples

Try these commands in the MyOS shell:

```bash
> run print 2 + 2
> run for i in 0..5 { print i }
> run fn fib(n) { if n < 2 { return n } return fib(n-1) + fib(n-2) }
> run print fib(10)
```

## Language Design Philosophy

HAL Script is designed to be:

1. **Simple** - Easy to learn, minimal syntax
2. **Powerful** - Functions, recursion, arrays
3. **Concise** - Not verbose like Java or C++
4. **Fast** - Direct AST interpretation
5. **Embedded** - Small footprint, no dependencies

## Future Features (Planned)

- [ ] Hash maps/dictionaries
- [ ] String methods (split, trim, etc.)
- [ ] File I/O operations
- [ ] More built-in functions
- [x] Break/continue statements ✓ Implemented
- [ ] Error handling (try/catch)
- [ ] Module system
- [x] AI integration for natural language programming ✓ Implemented

## Performance

HAL Script uses a tree-walking interpreter for simplicity. Typical performance:
- Fibonacci(20): ~10ms
- Prime sieve to 1000: ~50ms
- Simple loops: ~1μs per iteration

## Comparison to Other Languages

### Python-like:
```python
# Python
def greet(name):
    print(f"Hello, {name}")
```

```halscript
# HAL Script
fn greet(name) {
    print "Hello, " + name
}
```

### JavaScript-like:
```javascript
// JavaScript
for (let i = 0; i < 10; i++) {
    console.log(i);
}
```

```halscript
# HAL Script
for i in 0..10 {
    print i
}
```

### BASIC-like:
```basic
10 PRINT "HELLO"
20 FOR I = 1 TO 10
30 PRINT I
40 NEXT I
```

```halscript
print "HELLO"
for i in 1..11 {
    print i
}
```

## Error Handling

HAL Script provides clear error messages:

```bash
> run x + y
Runtime error: Undefined variable: x

> run print 10 / 0
Runtime error: Division by zero

> run fn foo() { return x }
> run foo()
Runtime error: Undefined variable: x
```

---

**HAL Script** - Simple, powerful, embedded scripting for MyOS
