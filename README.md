# jack-compiler

Compiles Jack to Hack VM commands.


### Usage
Run with cargo:
```bash
cargo run -- <file_name>.jack
# or
cargo run -- <dir> # translates <dir>/*.jack to <dir>/*.vm
```
See the `samples` directory for some sample .jack programs.
### Examples
```bash
cargo run -- samples/ComplexArrays/Main.jack
```
#### Main.jack
``` java
class Main {

    function void main() {
        var Array a, b, c;

        let a = Array.new(10);
        let b = Array.new(5);
        let c = Array.new(1);

        let a[3] = 2;
        let a[4] = 8;
        let a[5] = 4;
        let b[a[3]] = a[3] + 3;  // b[2] = 5
        let a[b[a[3]]] = a[a[5]] * b[((7 - a[3]) - Main.double(2)) + 1];  // a[5] = 8 * 5 = 40
        let c[0] = null;
        let c = c[0];

        do Output.printString("Test 1: expected result: 5; actual result: ");
        do Output.printInt(b[2]);
        do Output.println();
        do Output.printString("Test 2: expected result: 40; actual result: ");
        do Output.printInt(a[5]);
        do Output.println();
        do Output.printString("Test 3: expected result: 0; actual result: ");
        do Output.printInt(c);
        do Output.println();

        let c = null;

        if (c = null) {
            do Main.fill(a, 10);
            let c = a[3];
            let c[1] = 33;
            let c = a[7];
            let c[1] = 77;
            let b = a[3];
            let b[1] = b[1] + c[1];  // b[1] = 33 + 77 = 110;
        }

        do Output.printString("Test 4: expected result: 77; actual result: ");
        do Output.printInt(c[1]);
        do Output.println();
        do Output.printString("Test 5: expected result: 110; actual result: ");
        do Output.printInt(b[1]);
        do Output.println();
        return;
    }

    function int double(int a) {
    	return a * 2;
    }

    function void fill(Array a, int size) {
        while (size > 0) {
            let size = size - 1;
            let a[size] = Array.new(3);
        }
        return;
    }
}

```
#### Main.vm
```
function Main.main 3
push constant 10
call Array.new 1
pop local 0
push constant 5
call Array.new 1
pop local 1
push constant 1
call Array.new 1
pop local 2
push local 0
push constant 3
add
push constant 2
pop temp 0
pop pointer 1
push temp 0
pop that 0
push local 0
push constant 4
add
push constant 8
pop temp 0
pop pointer 1
push temp 0
pop that 0
push local 0
push constant 5
add
push constant 4
pop temp 0
pop pointer 1
push temp 0
pop that 0
push local 1
push local 0
push constant 3
add
pop pointer 1
push that 0
add
push local 0
push constant 3
add
pop pointer 1
push that 0
push constant 3
add
...
```

### Simulator
A VM simulator for the Hack VM is available [here](https://www.nand2tetris.org/software).

### Documentation
See Chapter 9-11 of [The Elements of Computing Systems: Building a Modern Computer from First Principles](https://www.amazon.com/Elements-Computing-Systems-Building-Principles/dp/0262640686) for the full specification of Jack
