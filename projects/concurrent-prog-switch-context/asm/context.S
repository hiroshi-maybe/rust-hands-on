#ifdef __APPLE__

#define GET_CONTEXT _get_context
#define SET_CONTEXT _set_context
#define SWITCH_CONTEXT _switch_context

#else

#define GET_CONTEXT get_context
#define SET_CONTEXT set_context
#define SWITCH_CONTEXT switch_context

#endif

.global GET_CONTEXT
.global SET_CONTEXT
.global SWITCH_CONTEXT

.text
.align 4

# store callee-saved registers
# rdi: first arg `Registers`
GET_CONTEXT:
    # popq    %rbp
    xor     %eax, %eax      /* Direct invocation returns 0 */

    movq    %rbx, (%rdi)    /* rbx */
    movq    %rbp, 0x08(%rdi)    /* rbp */
    movq    %r12, 0x10(%rdi)    /* r12 */
    movq    %r13, 0x18(%rdi)    /* r13 */
    movq    %r14, 0x20(%rdi)    /* r14 */
    movq    %r15, 0x28(%rdi)    /* r15 */
    leaq    8(%rsp), %rdx
    movq    %rdx, 0x30(%rdi) /* save rsp before SET_CONTEXT call */
    # pushq    %rbp
    movq    (%rsp), %rdx
    movq    %rdx, 0x38(%rdi)    /* rdx */
    # movq    $1016, 0x38(%rdi)
    ret

.text
.align 4

# restore calee-saved registers
# rdi: first arg `Registers`, rsi: second arg
SET_CONTEXT:
    xor     %eax, %eax
    inc     %eax            /* Return 1 instead */

    # popq    %rsi
    movq    (%rdi), %rbx
    movq    8(%rdi), %rbp
    movq    16(%rdi), %r12
    movq    24(%rdi), %r13
    movq    32(%rdi), %r14
    movq    40(%rdi), %r15
    movq    48(%rdi), %rdx # rsp in Registers
    movq    %rdx, %rsp  # configure rsp with the memory in heap

    # addq    $0x8, %rsp  # move stack pointer by one word for pseudo return address
    # pushq   %rbp    # maybe saving %rbp register for subsequent function entry_point()?
    # pushq   %rsi

    # pushq    %rsi
    # movq    56(%rdi), %rdx
    # jmpq    *%rdx   # jump to entry_point()
    pushq   56(%rdi)
    ret

# restore calee-saved registers
# rdi: first arg `Registers`, rsi: second arg
SWITCH_CONTEXT:
    xor     %eax, %eax      /* Direct invocation returns 0 */

    # GET_CONTEXT
    movq    %rbx, (%rsi)    /* rbx */
    movq    %rbp, 0x08(%rsi)    /* rbp */
    movq    %r12, 0x10(%rsi)    /* r12 */
    movq    %r13, 0x18(%rsi)    /* r13 */
    movq    %r14, 0x20(%rsi)    /* r14 */
    movq    %r15, 0x28(%rsi)    /* r15 */
    leaq    8(%rsp), %rdx
    movq    %rdx, 0x30(%rsi) /* rsp */
    movq    (%rsp), %rdx
    movq    %rdx, 0x38(%rsi) /* rdx */

    # SET_CONTEXT
    movq    (%rdi), %rbx
    movq    8(%rdi), %rbp
    movq    16(%rdi), %r12
    movq    24(%rdi), %r13
    movq    32(%rdi), %r14
    movq    40(%rdi), %r15
    movq    48(%rdi), %rdx # rsp in Registers
    movq    %rdx, %rsp  # configure rsp with the memory in heap    

    pushq   56(%rdi) # jump to foo2()
    ret
