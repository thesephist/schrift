` scratch file `

Newline := char(10)
log := x => out(string(x) + Newline)

log('Expected: hi, hello, hello world')

S := ['hi']
log(S.0)

(() => (
	S.0 := 'hello'
))()
log(S.0)

((S) => (
	S.0 := 'hello world'
))(S)
log(S.0)
