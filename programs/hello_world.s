;XA65

out=$0200
*=$0300

loop:
    lda msg,x    ;copy the message byte after byte to the output buffer
    sta out,x
    inx
    txa
    cmp len      ;if we reached the end of the message
    beq break    ;break the loop
    jmp loop

break:
    jsr flush
    jmp exit

flush:
    sta $02ff   ;writing to $02ff triggers the output
    rts

exit:
    jmp exit    ;there is no real way to exit, so we just halt
                ;please note that this will take 100% of the host CPU

.dsb 16,$00

msg: .asc  "Hello, 6502!"
len: .byte 12
