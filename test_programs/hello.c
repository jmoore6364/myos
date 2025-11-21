/* Minimal test program for MyOS ELF loader
 * This program uses the MyOS syscall interface (int 0x80)
 * Syscall 0 = exit
 * Syscall 6 = print
 */

void _start(void) {
    // sys_print (syscall 6) - print message
    const char* message = "Hello from ELF!\n";
    unsigned long len = 17;

    // Set up syscall: rax=6 (sys_print), rdi=message, rsi=length
    __asm__ volatile(
        "movq $6, %%rax\n"      // syscall number (sys_print)
        "movq %0, %%rdi\n"      // arg 1: message pointer
        "movq %1, %%rsi\n"      // arg 2: message length
        "int $0x80\n"           // invoke syscall
        :
        : "r"(message), "r"(len)
        : "rax", "rdi", "rsi"
    );

    // sys_exit (syscall 0) - exit with code 42
    __asm__ volatile(
        "movq $0, %%rax\n"      // syscall number (sys_exit)
        "movq $42, %%rdi\n"     // arg 1: exit code
        "int $0x80\n"           // invoke syscall
        :
        :
        : "rax", "rdi"
    );

    // Should never reach here, but just in case
    while(1);
}
