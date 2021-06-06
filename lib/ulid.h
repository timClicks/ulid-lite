#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef struct ulid_ctx {
  uint32_t seed;
} ulid_ctx;

typedef uint8_t ulid[16];

/**
 * Generate a `ulid_ctx` and seed the random number generator (RNG)
 * provided by your system's libc implementation of the rand() family.
 *
 * Passing 0 as `seed` will seed the random number generator from the
 * system's clock.
 */
struct ulid_ctx ulid_init(uint32_t seed);

/**
 * Create a new 128-bit ULID in `dest`.
 *
 * If the `ctx` pointer is null, the random number generator is re-seeded
 * from the system's clock.
 *
 * The destination `dest` must be a valid, non-null, pointer to `ulid`.
 */
void ulid_new(struct ulid_ctx *ctx, ulid *dest);

/**
 * Write a new ULID to `dest` as a string.
 *
 * Crockford's Base32 alphabet is used, and exactly 27 bytes are written,
 * including the terminating null byte.
 *
 * The destination `dest` must be a valid, non-null, pointer to a `char`
 * buffer with `size` bytes, and should have at least 27 bytes.
 *
 * If the `ctx` pointer is null, the random number generator is re-seeded
 * from the system's clock.
 *
 * Returns the number of characters printed (excluding the terminating null
 * byte) on success, or a negative error code on failure.
 */
int ulid_write_new(struct ulid_ctx *ctx, char *dest, size_t size);

/**
 * Write the 128-bit ULID pointed by `id` to `dest` as a string.
 *
 * Crockford's Base32 alphabet is used, and exactly 27 bytes are written,
 * including the terminating null byte.
 *
 * The destination `dest` must be a valid, non-null, pointer to a `char`
 * buffer with `size` bytes, and should have at least 27 bytes.
 *
 * Returns the number of characters printed (excluding the terminating null
 * byte) on success, or a negative error code on failure.
 */
int ulid_write(const ulid *id, char *dest, size_t size);
