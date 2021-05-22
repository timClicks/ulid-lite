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
 * Write a new 128-bit ULID in `dest`.
 *
 * If `ctx` pointer is null, the random number generator is re-seeded from
 * the system's clock.
 *
 * The destination pointer `dest` must be a valid, non-null, pointer to
 * `ulid_t`.
 */
void ulid_new(struct ulid_ctx *ctx, ulid_t *dest);

/**
 * Write a new ULID in `dest` using Crockford's Base32 alphabet.
 *
 * If `ctx` pointer is null, the random number generator is re-seeded from
 * the system's clock.
 *
 * The destination pointer `dest` must be a valid, non-null, pointer to
 * `char` buffer with at least length 26.
 *
 * No terminating null byte is written to the buffer.
 */
void ulid_new_string(struct ulid_ctx *ctx, char *dest);

/**
 * Encode the 128-bit ULID pointed by `id` as a string in `dest`.
 *
 * The destination pointer `dest` must be a valid, non-null, pointer to
 * `char` buffer with at least length 26.
 *
 * The Crockford's Base32 alphabet is used.  No terminating null byte is
 * written to the buffer.
 */
void ulid_encode(const ulid_t *id, char *dest);
