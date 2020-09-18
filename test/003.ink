` scopes test `

Newline := char(10)
log := x => out(string(x) + Newline)

log('hi')
n := (() => (12))()
log('hi')

log(n)

(
	log(n)
)
