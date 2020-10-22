` closures that leak heap objects, Comp mutability `

Newline := char(10)
log := x => out(string(x) + Newline)

` object definition `

Node := val => (
	obj := {
		val: val
		next: ()
		setNext: node => obj.next := node
		toString: () => (
			nextStr := (obj.next :: {
				() -> ''
				_ -> ' -> ' + (obj.next.toString)()
			})
			'Node(' + string(obj.val) + nextStr + ')'
		)
	}
)

printNode := node => log((node.toString)())

` construct linked list `

a := Node('a')
b := Node('b')
(a.setNext)(b)
(b.setNext)(Node('c'))
b.val := 'B'

` print list nodes `

printNode(a)
printNode(b)
printNode(b.next)

` Mutating comps from parent scope,
	closed over and passed as argument `

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
