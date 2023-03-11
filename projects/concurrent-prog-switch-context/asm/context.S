#ifdef __APPLE__

#define GET_CONTEXT _get_context
#define JUMP_TO_FUNC _jump_to_func
#define RESTORE_OLD_CONTEXT _restore_old_context
#define SWITCH_CONTEXT _switch_context

#else

#define GET_CONTEXT get_context
#define JUMP_TO_FUNC jump_to_func
#define RESTORE_OLD_CONTEXT restore_old_context
#define SWITCH_CONTEXT switch_context

#endif

.global GET_CONTEXT
.global JUMP_TO_FUNC
.global RESTORE_OLD_CONTEXT
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

    # xorps   %xmm0, %xmm0
    # movaps  %xmm0, -192(%rbp) # SEGV
    # movq    %xmm0, -192(%rbp) # No SEGV

    ret

.text
.align 4

# jump to a new context of a function, (%rsp)%16==8 after the context switch
# rdi: first arg `Registers`, rsi: second arg
JUMP_TO_FUNC:
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
    movq    %rdx, %rsp  # configure rsp with the memory in heapj

    /* new stack, (%rsp)%16==0 */

    # pad to achieve 16 bytes alignment in the jumped function
    addq    $0x8, %rsp

    # movq    56(%rdi), %rdx
    # jmpq    *%rdx   # jump to entry_point()
    pushq   56(%rdi)
    ret

# restore old context, (%rsp)%16==0 after the context switch
# rdi: first arg `Registers`, rsi: second arg
RESTORE_OLD_CONTEXT:
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
    movq    %rdx, %rsp  # configure rsp with the memory in heapj

    /* new stack, (%rsp)%16==0 */

    # No padding because original context should be keeping 16 bytes alignment
    # addq    $0x8, %rsp

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

    /* new stack, (%rsp)%16==0 */

    addq    $0x8, %rsp # pad to achieve 16 bytes alignment

    pushq   56(%rdi) # jump to foo2()
    ret
