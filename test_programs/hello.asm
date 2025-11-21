; Minimal x86-64 ELF test program
; This program simply exits with code 42

BITS 64

section .text
global _start

_start:
    ; Exit syscall (sys_exit = 60 on x86-64 Linux, but we'll use syscall 0 for MyOS)
    ; For MyOS, use int 0x80 with syscall number in rax
    mov rax, 0          ; sys_exit syscall number (assuming 0 for MyOS)
    mov rdi, 42         ; exit code
    int 0x80            ; invoke syscall

    ; Infinite loop in case syscall returns
    jmp $
