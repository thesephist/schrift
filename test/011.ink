` closures that leak heap objects, Comp mutability `

Newline := char(10)
log := x => out(string(x) + Newline)

` object definition `

` TODO: blocked on two issues / bugs:
	1. Val::eq comparison impl for Val::Comp (currently always returns true)
	2. Val::Comp's HashMap should be wrapped in a shared Rc pointer, not owned/cloned `

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
