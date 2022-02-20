out=$0200
*=$0300

loop:
    lda msg,x
    sta out,x
    inx
    txa
    cmp len
    beq break
    jmp loop
break:
    jmp exit

exit:
    jmp exit

.dsb 16,$00

msg: .asc  "Hello, 6502!"
len: .byte 12
