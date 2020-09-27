` list functions `

Newline := char(10)
log := x => out(string(x) + Newline)

map := (list, f) => (sub := (acc, i) => i :: {
	len(list) -> acc
	_ -> (
		acc.len(acc) := f(list.(i))
		sub(acc, i + 1)
	)
})([], 0)

nats := [1, 2, 3, 4, 5]
sq := x => x * x

printList := () => (
	out(string(nats.0)), out(', ')
	out(string(nats.1)), out(', ')
	out(string(nats.2)), out(', ')
	out(string(nats.3)), out(', ')
	log(nats.4)
)

printList()
nats := map(nats, sq)
printList()

