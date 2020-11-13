` obscure binary operations `

Newline := char(10)
log := x => out(string(x) + Newline)

` bitwise ops on byte strings `

a := 'ABCDEFG'
b := 'abcdEFg'

log('ABCDEFG: ' + (a & b))
log('-> length of ' + string(len(a & b)))
log('abcdEFg: ' + (a | b))
log('-> length of ' + string(len(a | b)))
log('    00 : ' + (a ^ b))
log('-> length of ' + string(len(a ^ b)))

