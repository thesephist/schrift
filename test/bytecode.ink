`
	Bytecode specification for Schrift

	- Every ExprList is compiled to its Block.
		- This means anything that should be a Block should be an ExprList prior to
		  compilation

	CONSIDERATIONS
	- should ops be allowed to take arg(x) / const(x) / bind(x) as arguments?  this
	  needs a more complex instruction decoding pipeline but will dramatically
	  reduce stack/register usage.
`

a := 1
b := 2
a + b + 3

Block {
	slots: 	5
	consts:	[3, native_add]
	binds: 	[]
	ret:	@5
	code: [
		@1	LOAD_ARG 0
		@2	LOAD_ARG 1
		@6	LOAD_CONST 1
		@3	CALL @6 @1 @2
		@4	LOAD_CONST 0
		@5	CALL @6 @3 @4
	]
}

===

c := 5
f := (a, b) => a + b + c + 2
g := f(2, 3)

Block {
	slots:	5
	consts: [5, 2, 3]
	binds:	[]
	ret:	@6
	code: [
		@1	LOAD_CONST 0
		@3	LOAD_BLOCK @2
		@4	LOAD_CONST 1
		@5	LOAD_CONST 2
		@6	CALL @3 @4 @5
	]
}
@2 -> Block {
	slots:	7
	consts:	[2, native_add]
	binds:	[@1]
	ret:	@13
	code: [
		@7	LOAD_ARG 0
		@8	LOAD_ARG 1
		@14	LOAD_CONST 1
		@9	CALL @14 @7 @8
		@10	LOAD_BIND 0
		@11	CALL @14 @9 @10
		@12	LOAD_CONST 0
		@13	CALL @14 @11 @12
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
	ret:	@6
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
	ret:	@3
	code: [
		@1	LOAD_CONST 0
		@2	LOAD_CONST 0
		@3	CALL_IF_EQ @4 @1 @2
		@3	CALL @5
	]
}
@4 -> Block {
	slots:	1
	consts:	[true]
	binds:	[]
	ret:	@6
	code: [
		@6	LOAD_CONST 0
	]
}
@5 -> Block {
	slots:	1
	consts:	[false]
	binds:	[]
	ret:	@7
	code: [
		@7	LOAD_CONST 0
	]
}

===

` exprlist as inline closure `

n := 5
( out(a) )

Block {
	slots:	2
	consts:	[5]
	binds:	[]
	ret:	@2
	code: [
		@1	LOAD_CONST 0
		@2	CALL @3
	]
}
@3 -> Block {
	slots:	2
	consts:	[out]
	binds:	[@1]
	ret:	@6
	code: [
		@5	LOAD_BIND 0
		@7	LOAD_CONST 0
		@6	CALL @7 @5
	]
}

===

a := native1
f := () => a()
a := native2

Block {
	slots:	0
	consts:	[native1, native2]
	binds:	[]
	ret:	@
	code: [
		@1	LOAD_CONST 0
		@2	LOAD_BLOCK @3
		@1	LOAD_CONST 1
	]
}
@3 -> Block {
	slots:	1
	consts:	[]
	binds:	[@1]
	ret:	@10
	code: [
		@10	CALL @1
	]
}
