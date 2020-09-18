` testing conditional branches `

Newline := char(10)
log := x => out(string(x) + Newline)

2 :: {
	2 -> log('is two')
	_ -> log('is not two')
}
