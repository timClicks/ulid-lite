#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct ulid_ctx {
  uint32_t seed;
} ulid_ctx;

typedef uint8_t ulid_t[16];

/**
 * Generate a `ulid_ctx` and seed the random number generator (RNG)
 * provided by your system's libc implementation of the rand() family.
 *
 * Passing 0 as `seed` will seed the random number generator from the
 * system's clock.
 */
struct ulid_ctx ulid_init(uint32_t seed);

/**
 * Create a new ULID.
 */
ulid_t *ulid_new(struct ulid_ctx *ctx);

/**
 * Free a ULID created with `ulid_new()`
 */
void ulid_delete(ulid_t*);

/**
 * Create a new ULID and encodes it as a NULL-terminated string
 * encoded in Crockford's Base32 alphabet.
 *
 * Note: This function incurs a memory allocation.
 */
char *ulid_new_string(struct ulid_ctx *ctx);

/**
 * Create a new ULID and write it to `buf`.
 *
 * Note: Callers should ensure that `ulid_init()` or `ulid_seed()`
 * has been called before this function.
 *
 * Warning: callers must ensure that `buf` is (at least) 26 bytes.
 */
void ulid_write_new(char *buf);

/**
 * Encode 128 bit ULID as a string.
 *
 * Note: callers should ensure that `dest` contains 27 bytes, e.g. 26 + NUL.
 */
void ulid_encode(ulid_t *id, char *dest);
