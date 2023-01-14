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
    # pop     %rbp
    xor     %eax, %eax      /* Direct invocation returns 0 */

    movq    %rbx, (%rdi)
    movq    %rbp, 0x08(%rdi)
    movq    %r12, 0x10(%rdi)
    movq    %r13, 0x18(%rdi)
    movq    %r14, 0x20(%rdi)
    movq    %r15, 0x28(%rdi)
    lea     0x08(%rsp), %rdx
    movq    %rdx, 0x30(%rdi)
    # push    %rbp
    # movq    (%rsp), %rdx
    # movq    %rdx, 0x38(%rdi)
    ret

.text
.align 4

# restore calee-saved registers
# rdi: first arg `Registers`, rsi: second arg
SWITCH_CONTEXT:
    xor     %eax, %eax
    inc     %eax            /* Return 1 instead */

    pop     %rsi
    movq    (%rdi), %rbx
    movq    8(%rdi), %rbp
    movq    16(%rdi), %r12
    movq    24(%rdi), %r13
    movq    32(%rdi), %r14
    movq    40(%rdi), %r15
    movq    48(%rdi), %rdx
    movq    %rdx, %rsp
    addq    $0x8, %rsp
    push    %rbp
    push    %rsi
    movq    56(%rdi), %rdx
    jmpq    *%rdx
