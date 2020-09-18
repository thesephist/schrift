` currying tests `

Newline := char(10)
log := x => out(string(x) + Newline)

sum2 := a => b => a + b
sum3 := a => b => c => a + b + c
sum4 := a => b => c => d => a + b + c + d

log(sum2(2)(3))
log(sum3(2)(3)(5))
log(sum4(2)(3)(5)(7))
