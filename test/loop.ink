` long loops, for microbenchmarking `

` as of 2020/9/27:
	Schrift is about 4.5x slower than Ink on this program
  as of 2020/11/4:
	Schrift is about 1.7x slower than Ink on this program `

Newline := char(10)
log := x => out(string(x) + Newline)

Max := 200000
run := () => (sub := i => i :: {
	Max -> log('done!')
	_ -> sub(i + 1)
})(0)

run()
