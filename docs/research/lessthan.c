/* Model signed and unsigned less-than (<) */
#include <stdlib.h>
#include <stdio.h>

/* Compute signed A < B, as
 * C is (A - B)
 * sA is sign of A
 * sB is sign of B
 * sC is sign of C
 * (A < B) is (sC ^ overflow(A - B))
 * overflow(A - B) is ((~sA & sB & sC) | (sA & ~sB & ~sC))
 * (A < B) is (sC ^ ((~sA & sB & sC) | (sA & ~sB & ~sC)))
 * Simplified:
 * (A < B) is ((sA & sC) | (sA & ~sB) | (sC & ~sB)) */
int slt(a, b)
char a, b;
{
    char c;

    c = a - b;

    return ((unsigned char)((a & c) | (a & ~b) | (c & ~b))) >> 7;
}
    
/* Compute unsigned A < B, as
 * If (A < B), then A - B generates borrow, or !carry from (A + -B)
 * Alternatively, (A < B) is !(B <= A), or !(B < (A + 1))
 * So if !(B < (A + 1)), then !borrow, or carry, generated from (B - (A + 1)),
 * or (B + -(A + 1)) or (B + (-A - 1)) ; but (-A - 1) is (~A)
 * So carry is generated from (B + ~A), iff (A < B) 
 */
int ult(a, b)
unsigned char a, b;
{
    unsigned int c;

    c = (unsigned int)b + (unsigned int)(a ^ 0xFF);

    return c >> 8;
}
    
int main(argc, argv)
int argc;
char *argv[];
{
    char a, b;
    unsigned char c, d;
    int i, j, k;

    i = 0;
    while (i < 256) {
        j = 0;
        while (j < 256) {
            a = (char)i;
            b = (char)j;
            c = (unsigned char)i;
            d = (unsigned char)j;
            k = (a < b);
#ifdef VERBOSE
            printf("a: %d b: %d, C: %d, signed less than: %d\n",
                   a, b, k, slt(a, b));
#endif
            if (!(k == slt(a, b))) {
                printf("Problem: a: %d b: %d, C: %d, signed less than: %d\n",
                       a, b, k, slt(a, b));
                exit(1);
            }
            k = (c < d);
#ifdef VERBOSE
            printf("c: %d d: %d, C: %d, unsigned less than: %d\n",
                   c, d, k, ult(c, d));
#endif
            if (!(k == ult(c, d))) {
                printf("Problem: c: %d d: %d, C: %d, unsigned less than: %d\n",
                       c, d, k, ult(c, d));
                exit(1);
            }
            ++j;
        }
        ++i;
    }

    return 0;
}
