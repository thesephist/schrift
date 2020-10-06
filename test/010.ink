` test byte string mutation `

Newline := char(10)
log := x => out(string(x) + Newline)

str := 'hello'
log(str)

(
	str.len(str) := ' world'
)
log(str)

log(str.0 := 'goodbye')
log(str)

log(str.0)
log(str.2)
log(str.20)

` join list of strings `

letters := ['alpha', 'beta', 'gamma', 'delta']
characters := ['harry potter', 'ron weasley', 'hermione granger']

` fast join implementation for lists with len >= 1 `
join := (pieces, pad) => (sub := (i, acc) => (
	i :: {
		len(pieces) -> acc
		_ -> sub(
			i + 1
			acc.len(acc) := pad + pieces.(i)
		)
	}
))(1, pieces.0)

log(join(letters, ' '))
log(join(characters, ', & '))
