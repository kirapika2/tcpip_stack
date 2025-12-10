#include <stdint.h>
#include <stddef.h>
#include <stdlib.h>
#include <pthread.h>
#include <time.h>

#include "platform.h"

#include "util.h"

int
platform_init(void)
{
    srandom(time(NULL));
    return 0;
}

int
platform_run(void)
{
    return 0;
}

int
platform_shutdown(void)
{
    return 0;
}

/*
 * Memory
 */

void *
memory_alloc(size_t size)
{
    return calloc(1, size);
}

void
memory_free(void *ptr)
{
    free(ptr);
}

/*
 * Lock
 */

int
lock_init(lock_t *lock)
{
    return pthread_mutex_init(lock, NULL);
}

int
lock_acquire(lock_t *lock)
{
    return pthread_mutex_lock(lock);
}

int
lock_release(lock_t *lock)
{
    return pthread_mutex_unlock(lock);
}

/*
 * Random
 */

uint16_t
random16(void)
{
    return random() % (UINT16_MAX+1);
}
