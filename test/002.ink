` eval test `

Newline := char(10)
log := x => out(string(x) + char(10))

first := 1 * 2 * 3
second := (4 + 5 + 6)
log(first + second + first + second)

add3 := (n) => (
	x := 1 + 2 + n
	x
)
log('should say 6:')
log(add3(3))

log('hello,' + ' world!')

sum := (a, b, c, d) => a + b + c + d
log(sum(10, 20, 30, 40))
log(1 * 2 + 3 / 4)
log(~2 < 4)
log(2 > 3)

` raw variable references `

a := a => a
b := b => b
c := c => c
d := d => d

log('Say 42:')
log(a(b(c(d(42)))))
