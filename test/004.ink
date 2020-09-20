` testing conditional branches `

Newline := char(10)
log := x => out(string(x) + Newline)

2 :: {
	2 -> log('is two')
	2 -> log('this shouldn\'t print')
	_ -> log('is not two')
}

log('branches done')
