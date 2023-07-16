16bit with 16 bit address bus
# registers
r0
r1
r2
r3

lr  link register
sp  stack pointer
pc  program counter

f  flag register => [zero , negative, carry, xxxxxx]

ma memory address, register that allows direct memory access
# instructions
--r == register
--m == memory
--i == imediate

math_single:
incr r
decr r
not r

math_double
add r, r/i
sub r, r/i
mul r, r,i

and r, r/i
or  r, r/i
xor r, r/i
cmp r, r/i

moves:
mov r, r/i
str r, [m]
load r, [m]

stack:
push r/i
pop  r

jump:
j   r/i



## sufixes
s == update flags
z == do when zero flag is set 
n == do when negative
xNZS
allow multiple suffixes
eq suffix == z
lt suffix == n
# instruction layout
MathSingles
MathDoubles
Moves
RamMoves
Stack
Jump


#### math
###### mathSingle
AAAB 0001 11CC CDDD
##### mathDouble
AAAB 000C CCCD DEEE
A: conditionals
B: update flags
C: instruction
D: register
F: Register or if all 1 then next 2xbyte as immediate
x: off = math signles, on = math doubles

#### moves
mov:
AAAB 0010 0CCD DEEE
same as above

str, load:
AAAB 0010 1CCD DEEE
same as above except for EEE as offset in either register or if 111 then next byte

#### stack
AAAB 0011 00CC 0DDD
C: instruction
D: register or if all 1 then next bytes as immediate

#### jump
AAAB 1000 0000 0000 (jump to register?)

### alu
AABBB
AA
00 -> pass A
01 -> pass B

10 -> logic unit
000 -> and
001 -> or
010 -> xor
011 -> A
100 -> nand
101 -> nor
110 -> xnor
111 -> A'

11 -> arithmetic unit
000 -> A+B
001 -> A-B
010 -> A+1
011 -> A-1
100 -> A\*B

mov nog is nachecken voor immediate erachter en of correcte path gekozen wordt