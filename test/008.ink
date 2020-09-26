` objects and lists `

Newline := char(10)
log := x => out(string(x) + Newline)

list := [10, 20, 30]

obj := {
	a: 'A'
	'b': 'B'
	('c' + ''): 'C'
}

` test list get `
log(list.0)
log(list.1)
log(list.2)
log(list.3)

` test map get `
log(obj.a)
log(obj.('b'))
log(obj.('' + 'c'))
