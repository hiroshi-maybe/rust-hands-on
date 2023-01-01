#ifdef __APPLE__

#define SET_CONTEXT _set_context
#define SWITCH_CONTEXT _switch_context

#else

#define SET_CONTEXT set_context
#define SWITCH_CONTEXT switch_context

#endif

.global SET_CONTEXT
.global SWITCH_CONTEXT

# rdi: first arg `Registers`
SET_CONTEXT:
    # store callee-saved registers
    mov [rdi + 0x00], rl5
    mov [rdi + 0x08], rl4
    mov [rdi + 0x10], rl3
    mov [rdi + 0x18], rl2
    mov [rdi + 0x20], rbx
    mov [rdi + 0x28], rbp

    # store stack pointer of the caller
    lea rax, [rsp + 8]
    mov [rdi + 0x30] rax

    # store instruction pointer before context switch
    mov rax, [rsp]
    mov [rdi + 0x38] rax

    mov rax $0
    ret

# rdi: first arg `Registers`, rsi: second arg
SWITCH_CONTEXT:
    # restore calee-saved registers
    mov rl5, [rdi + 0x00]
    mov rl4, [rdi + 0x08]
    mov rl3, [rdi + 0x10]
    mov rl2, [rdi + 0x18]
    mov rbx, [rdi + 0x20]
    mov rbp, [rdi + 0x28]
    mov rsp, [rdi + 0x30]
    mov rip, [rdi + 0x38]

    mov rax $0
    ret
