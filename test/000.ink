` scratch file `

Newline := char(10)
log := x => out(string(x) + Newline)

msg := 'hello'
(
	(
		log('message:')
		log(msg)
	)
)
