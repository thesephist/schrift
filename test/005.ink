` callback test `

Newline := char(10)
log := x => out(string(x) + Newline)

sq3 := a => b => (c => (a + b + c) * (c + b + a))

logWithCallback := (msg, cb) => (
	log(msg)
	cb()
)

log((a => b => c => (a + b + c))(2)(3)(5))
log((a => b => c => (a + b + c) + 0)(2)(3)(5))

logWithCallback('Computing something', () => log(sq3(2)(3)(5)))
