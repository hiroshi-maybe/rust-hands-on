#ifdef __APPLE__

#define SET_CONTEXT _set_context
#define SWITCH_CONTEXT _switch_context

#else

#define SET_CONTEXT set_context
#define SWITCH_CONTEXT switch_context

#endif

.global SET_CONTEXT
.global SWITCH_CONTEXT

# store callee-saved registers
# rdi: first arg `Registers`
SET_CONTEXT:
    mov %r15, 0x00(%rdi)
    mov %r14, 0x08(%rdi)
    mov %r13, 0x10(%rdi)
    mov %r12, 0x18(%rdi)
    mov %rbx, 0x20(%rdi)
    mov %rbp, 0x28(%rdi)

    # store stack pointer (rsp) of the caller
    lea 8(%rsp), %rax
    mov %rax, 0x30(%rdi)

    # store instruction pointer (rip) before context switch
    mov (%rsp), %rax
    mov %rax, 0x38(%rdi)

    mov $0, %rax
    ret

# restore calee-saved registers
# rdi: first arg `Registers`, rsi: second arg
SWITCH_CONTEXT:

    # push to the stack so that %rip is updated upon `ret` call
    pushq 0x38(%rdi)

    mov 0x00(%rdi), %r15
    mov 0x08(%rdi), %r14
    mov 0x10(%rdi), %r13
    mov 0x18(%rdi), %r12
    mov 0x20(%rdi), %rbx
    mov 0x28(%rdi), %rbp
    mov 0x30(%rdi), %rsp

    mov $0, %rax
    ret
