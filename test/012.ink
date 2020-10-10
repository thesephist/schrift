` deep value equalities `

Newline := char(10)
log := x => out(string(x) + Newline)

a := {
	0: 'first'
	1: false
	2: {
		key: 'third'
		k2: 0.12
	}
}
b := [
	()
	false
	{
		key: 'third'
		k2: 0.12
	}
]
b.0 := 'first'

log('Should be true:')
log((a = b) & (a.2 = b.2))
