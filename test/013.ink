` long loop for TCO verification `

f := i => i :: {
	10000 -> out(string(i) + ' done, tail call optimized.' + char(10))
	_ -> f(i + 1)
}

f(0)
