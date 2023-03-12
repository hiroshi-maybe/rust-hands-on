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

# https://diveintosystems.org/book/C7-x86_64/functions.html
# http://ylb.jp/Tech/x86_64ASM/x86_64_onMac.html
# https://github.com/rust-lang/rust/issues/74640
# https://graphitemaster.github.io/fibers/
# https://stackoverflow.com/questions/49391001/why-does-the-x86-64-amd64-system-v-abi-mandate-a-16-byte-stack-alignment

# store callee-saved registers
# rdi: first arg `Registers`
SET_CONTEXT:
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
# rdi: first arg `Registers`, rsi: second arg for %rsp padding
SWITCH_CONTEXT:
    xor     %eax, %eax
    inc     %eax            /* Return 1 instead */

    # popq    %rsi    # return address of switch_context()
    movq    (%rdi), %rbx
    movq    8(%rdi), %rbp
    movq    16(%rdi), %r12
    movq    24(%rdi), %r13
    movq    32(%rdi), %r14
    movq    40(%rdi), %r15
    movq    48(%rdi), %rdx # rsp in Registers
    movq    %rdx, %rsp  # move rsp to the user-space memory

    # pad to achieve 16 bytes alignment in the jumped function
    subq    %rsi, %rsp

    # pushq    %rbp
    # pushq    %rsi

    # movq    56(%rdi), %rdx
    # jmpq    *%rdx   # jump to entry_point()
    movq    56(%rdi), %rdx
    pushq   %rdx
    ret
