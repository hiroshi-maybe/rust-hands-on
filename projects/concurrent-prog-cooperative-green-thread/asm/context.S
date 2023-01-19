#ifdef __APPLE__

#define SET_CONTEXT _set_context
#define SWITCH_CONTEXT _switch_context
#define SWITCH_CONTEXT2 _switch_context2

#else

#define SET_CONTEXT set_context
#define SWITCH_CONTEXT switch_context
#define SWITCH_CONTEXT2 switch_context2

#endif

.global SET_CONTEXT
.global SWITCH_CONTEXT
.global SWITCH_CONTEXT2

.text
.align 4

# https://diveintosystems.org/book/C7-x86_64/functions.html

# store callee-saved registers
# rdi: first arg `Registers`
SET_CONTEXT:
    # popq    %rbp
    # movq    (%rsp), 0x38(%rdi)
    xor     %eax, %eax      /* Direct invocation returns 0 */

    movq    %rbx, (%rdi)
    movq    %rbp, 0x08(%rdi)
    movq    %r12, 0x10(%rdi)
    movq    %r13, 0x18(%rdi)
    movq    %r14, 0x20(%rdi)
    movq    %r15, 0x28(%rdi)
    lea     8(%rsp), %rdx
    movq    %rdx, 0x30(%rdi) /* save rsp before SET_CONTEXT call */
    # pushq    %rbp
    movq    (%rsp), %rdx
    movq    %rdx, 0x38(%rdi)
    # movq    $1013, 0x38(%rdi)
    ret

.text
.align 4

# restore calee-saved registers
# rdi: first arg `Registers`, rsi: second arg
SWITCH_CONTEXT:
    xor     %eax, %eax
    inc     %eax            /* Return 1 instead */

    pop     %rsi    # return address of switch_context()
    movq    (%rdi), %rbx
    movq    8(%rdi), %rbp
    movq    16(%rdi), %r12
    movq    24(%rdi), %r13
    movq    32(%rdi), %r14
    movq    40(%rdi), %r15
    movq    48(%rdi), %rdx # rsp in Registers
    movq    %rdx, %rsp  # configure rsp with the memory in heap
    addq    $0x8, %rsp  # move stack pointer by one word
    push    %rbp    /* wat? */
    push    %rsi
    movq    56(%rdi), %rdx
    jmpq    *%rdx   # jump to entry_point()


SWITCH_CONTEXT2:
    xor     %eax, %eax
    inc     %eax            /* Return 1 instead */

    pop     %rsi    # return address of switch_context()
    movq    (%rdi), %rbx
    movq    8(%rdi), %rbp
    movq    16(%rdi), %r12
    movq    24(%rdi), %r13
    movq    32(%rdi), %r14
    movq    40(%rdi), %r15
    movq    48(%rdi), %rdx # rsp in Registers
    # movq    %rdx, %rsp  # configure rsp with the memory in heap
    # addq    $0x8, %rsp  # move stack pointer by one word
    # push    %rbp    /* wat? */
    # push    %rsi
    movq    56(%rdi), %rdx
    jmpq    *%rdx   # jump to entry_point()
