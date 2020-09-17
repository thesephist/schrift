` eval test `

Newline := char(10)
log := x => out(string(x) + char(10))

first := 1 + 2 + 3
second := (4 + 5 + 6)
log(first + second + first + second)

add3 := (n) => 1 + 2 + n
log(add3(3))

log('hello,' + ' world!')

sum := (a, b, c, d) => a + b + c + d
log(sum(10, 20, 30, 40))
