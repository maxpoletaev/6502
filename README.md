# 🗿 6502

MOS 6502 CPU emulator made for fun.

## Current status

|||||||||||||||
|--- |--- |--- |--- |--- |--- |--- |--- |--- |--- |--- |--- |--- |--- |
|✅ADC|❌AND|❌ASL|✅BCC|✅BCS|✅BEQ|❌BIT|✅BMI|✅BNE|✅BPL|❌BRK|✅BVC|✅BVS|✅CLC|
|❌CLD|❌CLI|❌CLV|✅CMP|✅CPX|✅CPY|❌DEC|❌DEX|❌DEY|❌EOR|✅INC|✅INX|✅INY|✅JMP|
|✅JSR|✅LDA|✅LDX|✅LDY|❌LSR|✅NOP|❌ORA|✅PHA|✅PHP|✅PLA|✅PLP|❌ROL|❌ROR|❌RTI|
|✅RTS|❌SBC|❌SEC|❌SED|❌SEI|✅STA|✅STX|✅STY|✅TAX|✅TAY|✅TSX|✅TXA|✅TXS|✅TYA|

## The Virtual Machine

A CPU on its own is pretty useless. Because of that, the project comes with a
basic virtual machine to play around with. It works at 1MHz and connects the
CPU to 64K of RAM and memory-mapped stdout area that can be used to print
something to the terminal.

The overall memory layout looks like this:

```
  +---------------------+
  |   0x0000 - 0x00FF   |
  |      Zero Page      |
  +---------------------+
  |   0x0100 - 0x01FF   |
  |        Stack        | <- Stack Pointer
  +---------------------+
  |   0x0200 - 0x02FF   |
  |       Stdout        |
  +---------------------+
  |                     | <- Reset Vector
  |                     |    (your program starts here)
  |                     |
  |   0x0300 - 0xFFFF   |
  |     User memory     |
  |                     |
  |                     |
  |                     |
  +---------------------+
```

There are several examples of programs in the [programs](programs) directory
that can be compiled using [xa65](https://www.floodgap.com/retrotech/xa/)
assembler and then executed as `mos6502 hello_world`.

Unfortunateley, there are no debugging tools at the moment, so `println!` is
your friend.

## Resources

 * [Ben Eater’s Build a 65c02-based computer from scratch series](https://www.youtube.com/playlist?list=PLowKtXNTBypFbtuVMUVXNR0z1mu7dp7eH)
 * [Dave Poo’s 6502 Emulator in C++ series](https://www.youtube.com/playlist?list=PLLwK93hM93Z13TRzPx9JqTIn33feefl37)
 * [Andrew Jabobs’s 6502 Reference](https://web.archive.org/web/20210426072206/http://www.obelisk.me.uk/6502/index.html)
 * Amazing [NESDev Wiki](https://wiki.nesdev.org/w/index.php?title=CPU)
