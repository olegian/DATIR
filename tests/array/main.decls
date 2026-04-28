decl-version 2.0
input-language rust
var-comparability implicit

ppt tests/array/main.rs::bar:::ENTER
ppt-type enter
variable arr
  var-kind variable
  dec-type [usize]
  rep-type hashcode
  comparability -1
variable arr.length
  var-kind field length
  dec-type usize
  rep-type int
  enclosing-var arr
  comparability -1
variable arr[..]
  var-kind array
  dec-type usize
  rep-type int[]
  enclosing-var arr
  array 1
  comparability -1
variable unused
  var-kind variable
  dec-type usize
  rep-type int
  comparability -1
variable y
  var-kind variable
  dec-type usize
  rep-type int
  comparability -1
variable z
  var-kind variable
  dec-type usize
  rep-type int
  comparability -1

ppt tests/array/main.rs::bar:::EXIT
ppt-type exit
parent enter-exit tests/array/main.rs::bar:::ENTER 4
variable arr
  var-kind variable
  dec-type [usize]
  rep-type hashcode
  comparability -1
variable arr.length
  var-kind field length
  dec-type usize
  rep-type int
  enclosing-var arr
  comparability -1
variable arr[..]
  var-kind array
  dec-type usize
  rep-type int[]
  enclosing-var arr
  array 1
  comparability -1
variable return
  var-kind return
  dec-type usize
  rep-type int
  comparability -1
variable unused
  var-kind variable
  dec-type usize
  rep-type int
  comparability -1
variable y
  var-kind variable
  dec-type usize
  rep-type int
  comparability -1
variable z
  var-kind variable
  dec-type usize
  rep-type int
  comparability -1

ppt tests/array/main.rs::bar:::EXIT20
ppt-type subexit
parent exit-exitnn tests/array/main.rs::bar:::EXIT 5
variable arr
  var-kind variable
  dec-type [usize]
  rep-type hashcode
  comparability -1
variable arr.length
  var-kind field length
  dec-type usize
  rep-type int
  enclosing-var arr
  comparability -1
variable arr[..]
  var-kind array
  dec-type usize
  rep-type int[]
  enclosing-var arr
  array 1
  comparability -1
variable return
  var-kind return
  dec-type usize
  rep-type int
  comparability -1
variable unused
  var-kind variable
  dec-type usize
  rep-type int
  comparability -1
variable y
  var-kind variable
  dec-type usize
  rep-type int
  comparability -1
variable z
  var-kind variable
  dec-type usize
  rep-type int
  comparability -1

ppt tests/array/main.rs::foo:::ENTER
ppt-type enter
variable arr
  var-kind variable
  dec-type [u32; 3]
  rep-type hashcode
  comparability -1
variable arr.length
  var-kind field length
  dec-type usize
  rep-type int
  enclosing-var arr
  comparability -1
variable arr[..]
  var-kind array
  dec-type u32
  rep-type int[]
  enclosing-var arr
  array 1
  comparability -1
variable unused
  var-kind variable
  dec-type u32
  rep-type int
  comparability -1
variable x
  var-kind variable
  dec-type u32
  rep-type int
  comparability -1
variable y
  var-kind variable
  dec-type u32
  rep-type int
  comparability -1

ppt tests/array/main.rs::foo:::EXIT
ppt-type exit
parent enter-exit tests/array/main.rs::foo:::ENTER 2
variable arr
  var-kind variable
  dec-type [u32; 3]
  rep-type hashcode
  comparability -1
variable arr.length
  var-kind field length
  dec-type usize
  rep-type int
  enclosing-var arr
  comparability -1
variable arr[..]
  var-kind array
  dec-type u32
  rep-type int[]
  enclosing-var arr
  array 1
  comparability -1
variable return
  var-kind return
  dec-type u32
  rep-type int
  comparability -1
variable unused
  var-kind variable
  dec-type u32
  rep-type int
  comparability -1
variable x
  var-kind variable
  dec-type u32
  rep-type int
  comparability -1
variable y
  var-kind variable
  dec-type u32
  rep-type int
  comparability -1

ppt tests/array/main.rs::foo:::EXIT13
ppt-type subexit
parent exit-exitnn tests/array/main.rs::foo:::EXIT 3
variable arr
  var-kind variable
  dec-type [u32; 3]
  rep-type hashcode
  comparability -1
variable arr.length
  var-kind field length
  dec-type usize
  rep-type int
  enclosing-var arr
  comparability -1
variable arr[..]
  var-kind array
  dec-type u32
  rep-type int[]
  enclosing-var arr
  array 1
  comparability -1
variable return
  var-kind return
  dec-type u32
  rep-type int
  comparability -1
variable unused
  var-kind variable
  dec-type u32
  rep-type int
  comparability -1
variable x
  var-kind variable
  dec-type u32
  rep-type int
  comparability -1
variable y
  var-kind variable
  dec-type u32
  rep-type int
  comparability -1

ppt tests/array/main.rs::main:::ENTER
ppt-type enter

ppt tests/array/main.rs::main:::EXIT
ppt-type exit
parent enter-exit tests/array/main.rs::main:::ENTER 0

ppt tests/array/main.rs::main:::EXIT1
ppt-type subexit
parent exit-exitnn tests/array/main.rs::main:::EXIT 1


