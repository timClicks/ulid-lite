#include <stdarg.h>
#include <stdbool.h>
#include <stdint.h>
#include <stdlib.h>

typedef uint8_t UlidArray[16];

/**
 * Initialize the random number generator from the system's clock
 */
void ulid_init(void);

/**
 * Seed the random number generator with `s`
 */
void ulid_seed(uint32_t s);

/**
 * Create a new ULID
 *
 * Note: Callers should ensure that `ulid_init()` or `ulid_seed()`
 *       has been called before this function.
 */
UlidArray *ulid_new(void);

/**
 * Create a new ULID and encodes it as a Crockford Base32 string.
 *
 * Note: Callers should ensure that `ulid_init()` or `ulid_seed()`
 *       has been called before this function.
 *
 * Note: This function allocates memory. Callers are required to free
 *       the return value when is no longer useful.
 */
char *ulid_new_string(void);

/**
 * Create a new ULID and write it to `buf`.
 *
 * Note: Callers should ensure that `ulid_init()` or `ulid_seed()`
 * has been called before this function.
 *
 * Warning: callers must ensure that `buf` is (at least) 26 bytes.
 */
void ulid_write_new(char *buf);
