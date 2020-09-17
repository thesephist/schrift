`
	Bytecode specification for Schrift

	- Every ExprList is compiled to its Block.
		- This means anything that should be a Block should be an ExprList
		  prior to compilation

	CONSIDERATIONS
	- should ops be allowed to take const(x) as arguments?  this needs a more
	  complex instruction decoding pipeline but will dramatically reduce
	  stack/register usage.
`

a := 1
b := 2
a + b + 3

Block {
	slots: 	5
	consts:	[3]
	binds:	[]
	code: [
		@1	LOAD_ARG 0
		@2	LOAD_ARG 1
		@3	ADD @1 @2
		@4	LOAD_CONST 0
		@5	ADD @3 @4
	]
}

===

c := 5
f := (a, b) => a + b + c + 2
g := f(2, 3)

Block {
	slots:	5
	consts: [5, 2, 3, #1]
	binds:	[]
	code: [
		@1	LOAD_CONST 0
		@1	ESCAPE ` escape to heap, puts Val::HeapPtr(Arc<Val>) in @15 `
		@3	LOAD_CONST 3
		@4	LOAD_CONST 1
		@5	LOAD_CONST 2
		@6	CALL @3 [@4, @5]
	]
}
#1 -> Block {
	slots:	7
	consts:	[2] ` @15 is a Val::HeapPtr(Arc<Val>) to the heap `
	binds:	[@1]
	code: [
		@7	LOAD_ARG 0 ` argument slots are filled automatically `
		@8	LOAD_ARG 1 ` by the virtual machine, here for illustrative purposes `
		@9	ADD @7 @8
		@10	LOAD_ESC 0 ` load escaped value from the heap `
		@11	ADD @9 @10
		@12	LOAD_CONST 0
		@13	ADD @11 @12
	]
}

===

a := [1]
a.0 := 10
b := a.0

Block {
	slots:	6
	consts:	[1, 0, 10]
	binds:	[]
	code: [
		@1	MAKE_COMP
		@2	LOAD_CONST 0
		@3	LOAD_CONST 1
			SET_COMP @1 @2 @3
		@4	LOAD_CONST 2
		@5	SET_COMP @1 @3 @4
		@6	GET_COMP @1 @3
	]
}

===

2 :: {
	2 -> true
	_ -> false
}

Block {
	slots:	3
	consts:	[2, true, false]
	binds:	[]
	code: [
		@1	LOAD_CONST 0
		@2	LOAD_CONST 0
		` arguments here are:
			- block to call
			- eq arguments (1 and 2)
			- number of extra instructions
				to jump if called `
		@3	CALL_IF_EQ @4 @1 @2 1
		@3	CALL_IF_EQ @5 @1 @2 0
	]
}
@4 -> Block {
	slots:	1
	consts:	[true]
	code: [
		@6	LOAD_CONST 0
	]
}
@5 -> Block {
	slots:	1
	consts:	[false]
	code: [
		@7	LOAD_CONST 0
	]
}

===

` exprlist as inline closure `

n := 5
( out(n) )

Block {
	slots:	2
	consts:	[5, #1]
	binds:	[]
	code: [
		@1	LOAD_CONST 0
		@1	ESCAPE
		@4	LOAD_CONST 1
		@2	CALL @4 []
	]
}
#1 -> Block {
	slots:	2
	consts:	[builtin_out]
	binds:	[@1]
	code: [
		@5	LOAD_ESC 0
		@7	LOAD_CONST 0
		@6	CALL @7 [@5]
	]
}

===

a := native1
f := () => a()
a := native2

Block {
	slots:	0
	consts:	[native1, native2, #1]
	binds:	[]
	code: [
		@1	LOAD_CONST 0
		@1	ESCAPE
		@2	LOAD_CONST 2
		@1	LOAD_CONST 1
	]
}
#1 -> Block {
	slots:	1
	consts:	[]
	binds:	[@1]
	code: [
		@11	LOAD_ESC 0
		@10	CALL @11 []
	]
}
