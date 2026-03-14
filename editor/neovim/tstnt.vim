if exists("b:current_syntax")
  finish
endif

syn keyword tstntKeyword do let mut return if else while loop match use struct impl test null repeat in break continue try catch throw async await thread true false unit enum interface
syn keyword tstntBuiltin print len str int float bool type_of assert assert_eq assert_ne panic range map filter reduce apply zip flatten sort unique sum min max abs is_null input
syn keyword tstntType int float str bool void any char

syn match tstntComment "#.*$"
syn match tstntNumber "\b[0-9]\+\(\.[0-9]\+\)\?\b"
syn match tstntOperator "->\\|\.\.\\.\\|?\.\\|==\\|!=\\|<=\\|>=\\|&&\\|||\\|+=\\|-=\\|*=\\|/="
syn match tstntFunction "\b[a-z_][a-zA-Z0-9_]*\ze("
syn match tstntStruct "\b[A-Z][a-zA-Z0-9_]*\b"

syn region tstntString start='"' end='"' skip='\\"' contains=tstntInterp
syn match tstntInterp "{[^}]*}" contained

hi def link tstntKeyword   Keyword
hi def link tstntBuiltin   Function
hi def link tstntType      Type
hi def link tstntComment   Comment
hi def link tstntNumber    Number
hi def link tstntOperator  Operator
hi def link tstntFunction  Function
hi def link tstntStruct    Type
hi def link tstntString    String
hi def link tstntInterp    Special

let b:current_syntax = "tstnt"
