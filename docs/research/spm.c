/* Model of an NxN = N serial-parallel multiplier */
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>

#define NBITS 24

/* Multiplier bit cells */
struct spmcell {
    int XY;                     /* Multiplier (x) AND multiplicand (y) */
    int Si;                     /* Partial product sum in */
    int So;                     /* Partial product sum out */
    int Ci;                     /* Carry in (last out) */
    int Co;                     /* Carry out (next in) */
};

/* 3,2 counter / full adder truth table */
struct ttval {
    int carry;
    int sum;
};
static struct ttval fa32[8] = { /* XY Si Ci  | Co So */
    { 0, 0 },                   /*  0  0  0  |  0  0 */
    { 0, 1 },                   /*  0  0  1  |  0  1 */
    { 0, 1 },                   /*  0  1  0  |  0  1 */
    { 1, 0 },                   /*  0  1  1  |  1  0 */
    { 0, 1 },                   /*  1  0  0  |  0  1 */
    { 1, 0 },                   /*  1  0  1  |  1  0 */
    { 1, 0 },                   /*  1  1  0  |  1  0 */
    { 1, 1 }                    /*  1  1  1  |  1  1 */
};

/* Serial-parallel multiplier */
uint32_t product1(x, y)
uint32_t x, y;
{
    int i, j, k;
    uint32_t result;
    struct spmcell cell[NBITS];

    /* Reset output flip-flops on all cells, clear result */
    i = 0;
    while (i < NBITS) {
        cell[i].So = cell[i].Co = 0;
        ++i;
    }
    result = 0;

    /* Multiply in N steps */
    cell[NBITS - 1].Si = cell[NBITS - 1].Ci = 0;
    j = 0;
    while (j < NBITS) {

        /* First, move inputs into place for each cell */
        i = 0;
        while (i < NBITS) {
            if (!(i == (NBITS - 1))) {
                cell[i].Si = cell[i + 1].So;
                cell[i].Ci = cell[i].Co;
            }
            cell[i].XY = (x & (1 << j)) && (y & (1 << i));
            ++i;
        }

        /* Now, compute and register outputs from adder truth table */
        i = 0;
        while (i < NBITS) {
            k = cell[i].XY*4 + cell[i].Si*2 + cell[i].Ci;
            cell[i].Co = fa32[k].carry;
            cell[i].So = fa32[k].sum;
            ++i;
        }

        /* Set result bit from low cell */
        result |= cell[0].So << j;

        ++j;
    }

    return result;
}

/* Serial-parallel multiplier, half-width */
uint32_t hsprod(x, y)
uint32_t x, y;
{
    int i, j, k;
    uint32_t result;
    struct spmcell cell[NBITS/2];

    /* Reset output flip-flops on all cells, clear result */
    i = 0;
    while (i < NBITS/2) {
        cell[i].So = cell[i].Co = 0;
        ++i;
    }
    result = 0;

    /* Multiply in N (NBITS/2) steps */
    cell[NBITS/2 - 1].Si = cell[NBITS/2 - 1].Ci = 0;
    j = 0;
    while (j < NBITS/2) {

        /* First, move inputs into place for each cell */
        i = 0;
        while (i < NBITS/2) {
            if (!(i == (NBITS/2 - 1))) {
                cell[i].Si = cell[i + 1].So;
                cell[i].Ci = cell[i].Co;
            }
            cell[i].XY = (x & (1 << j)) && (y & (1 << i));
            ++i;
        }

        /* Now, compute and register outputs from adder truth table */
        i = 0;
        while (i < NBITS/2) {
            k = cell[i].XY*4 + cell[i].Si*2 + cell[i].Ci;
            cell[i].Co = fa32[k].carry;
            cell[i].So = fa32[k].sum;
            ++i;
        }

        /* Set result bit from low cell */
        result |= cell[0].So << j;

        ++j;
    }

    return result;
}

/* Serial-parallel multiplier, when just the multiplier is half the width */
uint32_t hsxprod(x, y)
uint32_t x, y;
{
    int i, j, k;
    uint32_t result, sum, carry;
    struct spmcell cell[NBITS];

    /* Reset output flip-flops on all cells, clear result */
    i = 0;
    while (i < NBITS) {
        cell[i].So = cell[i].Co = 0;
        ++i;
    }
    result = 0;

    /* First N/2 steps, use low multiplier (x) */
    cell[NBITS - 1].Si = cell[NBITS - 1].Ci = 0;
    j = 0;
    while (j < NBITS/2) {

        /* First, move inputs into place for each cell */
        i = 0;
        while (i < NBITS) {
            if (!(i == (NBITS - 1))) {
                cell[i].Si = cell[i + 1].So;
                cell[i].Ci = cell[i].Co;
            }
            cell[i].XY = (x & (1 << j)) && (y & (1 << i));
            ++i;
        }

        /* Now, compute and register outputs from adder truth table */
        i = 0;
        while (i < NBITS) {
            k = cell[i].XY*4 + cell[i].Si*2 + cell[i].Ci;
            cell[i].Co = fa32[k].carry;
            cell[i].So = fa32[k].sum;
            ++i;
        }

        /* Set result bit from low cell */
        result |= cell[0].So << j;

        ++j;
    }

    /* Complete last step, move inputs into place for low cells */
    i = 0;
    while (i < NBITS/2) {
        if (!(i == (NBITS - 1))) {
            cell[i].Si = cell[i + 1].So;
            cell[i].Ci = cell[i].Co;
        }
        ++i;
    }

    /* Add remaining sums and carries */
    carry = sum = 0;
    i = 0;
    while (i < NBITS/2) {
        sum |= cell[i].Si << i;
        carry |= cell[i].Ci << i;
        ++i;
    }
    result |= ((sum + carry) << NBITS/2);

    return result;
}

/* Sum partial products from upper and lower halves of multiplier */
uint32_t product2(x, y)
uint32_t x, y;
{
    uint32_t sum, p0, p1;

    p0 = hsxprod(x & ((1 << NBITS/2) - 1), y);
    p1 = (hsprod(x >> NBITS/2, y)) << NBITS/2;
    sum = (p0 + p1) & (((uint32_t)1 << NBITS) - 1);

    return sum;
}

int main(argc, argv)
int argc;
char *argv[];
{
    int32_t x, y;

    if (!(argc == 3) || \
        !(sscanf(argv[1], "%d", &x) == 1) || \
        !(sscanf(argv[2], "%d", &y) == 1)) {
        printf("%s expects two integer arguments\n", argv[0]);
        exit(1);
    }

    printf("x: %d y: %d, product1: %d\n", x, y, product1(x, y));
    printf("x: %d y: %d, product2: %d\n", x, y, product2(x, y));

    return 0;
}
