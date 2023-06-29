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

math:
incr r
add r, r/i
decr r
sub r, r/i
mul r, r,i

and r, r/i
or  r, r/i
xor r, r/i
not r
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
# instruction layout
#### math
AAAB 000C CCCD DEEE
A: conditionals
B: update flags
C: instruction
D: register
F: Register or if all 1 then next 2xbyte as immediate

#### moves
mov:
AAAB 0010 0CCD DEEE
same as above

str, load:
AAAB 0010 1CCD DEEE
same as above except for EEE as offset in either register or if 111 then next byte

#### stack
AAAB 0011 CC DDDD
C: instruction
DD: register or if all 1 then next bytes as immediate
