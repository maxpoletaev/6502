;XA65
out=$0200

*=$0300
loop:
    lda msg,x     ;copy the message byte after byte to the output buffer
    sta out,x
    inx
    cpx len       ;while (x < len)
    bne loop

    jsr flush
    jmp halt

flush:
    lda #$0A      ;newline
    sta out+$ff   ;writing to $02ff triggers the output
    rts

halt:
    jmp halt

msg: .asc  "Hello, 6502!"
len: .byte 12
