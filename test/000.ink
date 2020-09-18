` scratch file `

log := x => out(string(x) + char(10))

msg := 'hello'
(
	(
		log('message:')
		log(msg)
	)
)
