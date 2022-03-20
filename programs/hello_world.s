;xa65 assembler
;compile with xa program.s -o program

stdout    = $0200
*         = $0300

len:    .byte 12
msg:    .asc  "Hello, 6502!"

start:
loop:
        lda msg,x           ;copy the message byte after byte to the output buffer
        sta stdout,x
        inx
        cpx len             ;while (x < len)
        bne loop
        lda #$0A            ;line break
        sta stdout+$ff      ;writing to $02ff triggers the output
spin:
        jmp spin            ;there is no way to exit the program


tail:
        * = $FFFA
        .dsb (*-tail)

        * = $FFFA
        .word $0000         ;nmi vector
        .word start         ;reset vector
        .word $0000         ;irq vector
