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
