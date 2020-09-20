` testing conditional branches `

Newline := char(10)
log := x => out(string(x) + Newline)

2 :: {
	2 -> log('is two')
	2 -> log('this shouldn\'t print')
	_ -> log('is not two')
}

log('branches done')

log('result should be 3')
result := (1 + 2 :: {
	3 -> 3
	_ -> ~10
})
log('result: ' + string(result))
