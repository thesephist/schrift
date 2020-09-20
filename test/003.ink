` scopes test `

Newline := char(10)
log := x => out(string(x) + Newline)

log('hi')
n := (() => (12))()
log('hi')

log(n)

(
	log(n)
	log(n * 2)
)

` closure that closes over closed-over variables
	in an outer scope `

msg := 'hello'
(
	(
		log('message:')
		log(msg)
		log('second message:')
		log(msg + msg + msg)
	)
)

logAThing := thing => (
	(
		log('logging: ' + string(thing))
	)
)

printAThing := thing => (
	(
		log('printing: ' + string(thing))
	)
)

log('10, 12, 20, 42')
logAThing(10)
logAThing(12)
printAThing(20)
printAThing(42)
