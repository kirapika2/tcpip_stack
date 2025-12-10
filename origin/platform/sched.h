#ifndef PLATFORM_SCHED_H
#define PLATFORM_SCHED_H

#include <pthread.h>
#include <time.h>

struct sched_task {
    struct sched_task *next;
    pthread_cond_t cond;
    int interrupted;
    int wc; /* wait count */
};

#define SCHED_TASK_INITIALIZER {NULL, PTHREAD_COND_INITIALIZER, 0, 0}

extern int
sched_task_init(struct sched_task *task);
extern int
sched_task_destroy(struct sched_task *task);
extern int
sched_task_sleep(struct sched_task *task, lock_t *lock, const struct timespec *abstime);
extern int
sched_task_wakeup(struct sched_task *task);

extern int
sched_init(void);
extern int
sched_run(void);
extern int
sched_shutdown(void);

#endif
