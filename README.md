## Overview
This is an interpreter for a custom programming language.

All components are hand-written, including the lexer and the parser. It is roughly based on the tree-walk interpreter presented in Nystrom's [Crafting Interpreters](https://craftinginterpreters.com/), but with additional support for

- arrays,
- dictionaries,
- functions (and recursion), and
- built-in functions.

It is also written in Rust, so much of the original Java logic and structure has been redesigned.

To use the interpreter, first build the project using Cargo. Then, either:

- Execute the binary without arguments. This will launch the REPL interface.
- Supply the path to the program source code as the argument. This will execute the program.

An accompanying report is available on request.
## Sample programs
You can use NEAL to...
- Calculate the factorial of a number

```
func factorial(n) {
    if (n <= 1) {
        return 1
    }
    return n * factorial(n-1)
}

print factorial(6)
```

- Compute the prime factors of a number
```
var n = to_number(input("Enter a number: "))  # `to_number` is built-in
var prime_factors = []

for (var x = 2; x < n; x = x + 1) {
    if (n % x == 0) {
        # Check if prime
        var prime = true
        for (var y = 2; y*y <= x; y = y + 1) {
            if (x % y == 0) {
                prime = false
                break
            }
        }
        if (prime) {
            append(prime_factors, x)
        }
    }
}
print sort(prime_factors)  # `sorted` is built-in
```

- Solve the Tower of Hanoi puzzle
```
func hanoi(num_disk, start_peg, end_peg) {
    if (num_disk > 0) {
        hanoi(num_disk - 1, start_peg, 6 - start_peg - end_peg)
        print 'Move disk ' + to_string(num_disk) + ': ' + to_string(start_peg) + ' -> ' + to_string(end_peg)
        hanoi(num_disk - 1, 6 - start_peg - end_peg, end_peg)
    }
}

hanoi(3, 1, 3)
```
