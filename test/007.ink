` fibonacci sequence generator `

Newline := char(10)
log := x => out(string(x) + Newline)

` naive implementation `
fib := n => n :: {
	0 -> 0
	1 -> 1
	_ -> fib(n - 1) + fib(n - 2)
}

out('Naive solution: '), log(fib(10))

log('Sequence, tail recursive...')

Max := 30
(sub := (a, b, n) => n :: {
	0 -> log(b)
	_ -> (
		out(string(b) + ', ')
		sub(b, a + b, n - 1)
	)
})(0, 1, Max)

