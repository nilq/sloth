## sloth
a match-oriented programming language

```
fib: (i128) = {
  |0| 0
  |1| 1
  |n| (fib n - 1) + fib n - 2
}

a: i128 = fib 100
```
