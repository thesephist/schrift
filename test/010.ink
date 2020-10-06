` test byte string mutation `

Newline := char(10)
log := x => out(string(x) + Newline)

str := 'hello'
log(str)

str.len(str) := ' world'
log(str)

log(str.0 := 'goodbye')
log(str)

log(str.0)
log(str.2)
log(str.20)
