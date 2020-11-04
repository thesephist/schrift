` long loop for TCO verification `

Newline := char(10)
log := x => out(string(x) + Newline)

` sanity check for loop correctness `
sum := max => (sub := (acc, i) => i :: {
	max -> acc + i
	_ -> sub(acc + i, i + 1)
})(0, 1)
log(sum(1000))

f := i => i :: {
	10000 -> log(string(i) + ' done, tail call optimized.')
	_ -> f(i + 1)
}
f(0)
