#ifdef __APPLE__

#define SET_CONTEXT _set_context
#define SWITCH_CONTEXT _switch_context

#else

#define SET_CONTEXT set_context
#define SWITCH_CONTEXT switch_context

#endif

.global SET_CONTEXT
.global SWITCH_CONTEXT

.text
.align 4

# store callee-saved registers
# rdi: first arg `Registers`
SET_CONTEXT:
    pop %rbp
    xor %eax, %eax

    movq %r15, 0x00(%rdi)
    movq %r14, 0x08(%rdi)
    movq %r13, 0x10(%rdi)
    movq %r12, 0x18(%rdi)
    movq %rbx, 0x20(%rdi)
    movq %rbp, 0x28(%rdi)

    # store stack pointer (rsp) of the caller
    lea 8(%rsp), %rdx
    movq %rdx, 0x30(%rdi)
    push %rbp

    # store instruction pointer (rip) before context switch
    movq (%rsp), %rdx
    movq %rdx, 0x38(%rdi)
    ret

.text
.align 4

# restore calee-saved registers
# rdi: first arg `Registers`, rsi: second arg
SWITCH_CONTEXT:
    xor %eax, %eax
    inc %eax

    pop %rsi
    movq 0x00(%rdi), %r15
    movq 0x08(%rdi), %r14
    movq 0x10(%rdi), %r13
    movq 0x18(%rdi), %r12
    movq 0x20(%rdi), %rbx
    movq 0x28(%rdi), %rbp
    movq 0x30(%rdi), %rdx
    movq %rsp, %rdx
    addq $0x8 %rsp
    push %rbp
    push %rsi
    movq 0x38(%rdi), %rdx
    jmpq *%rdx
