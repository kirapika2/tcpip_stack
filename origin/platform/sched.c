#include <pthread.h>
#include <time.h>
#include <errno.h>

#include "platform.h"

static lock_t lock = LOCK_INITIALIZER;
static struct sched_task *tasks; /* sleep tasks */

static void
tasks_add(struct sched_task *task)
{
    lock_acquire(&lock);
    task->next = tasks;
    tasks = task;
    lock_release(&lock);
}

static void
tasks_del(struct sched_task *task)
{
    struct sched_task *entry;

    lock_acquire(&lock);
    if (tasks == task) {
        tasks = task->next;
        task->next = NULL;
        lock_release(&lock);
        return;
    }
    for (entry = tasks; entry; entry = entry->next) {
        if (entry->next == task) {
            entry->next = task->next;
            task->next = NULL;
            break;
        }
    }
    lock_release(&lock);
}

int
sched_task_init(struct sched_task *task)
{
    task->next = NULL;
    pthread_cond_init(&task->cond, NULL);
    task->interrupted = 0;
    task->wc = 0;
    return 0;
}

int
sched_task_destroy(struct sched_task *task)
{
    if (task->wc) {
        return -1;
    }
    return pthread_cond_destroy(&task->cond);
}

int
sched_task_sleep(struct sched_task *task, lock_t *lock, const struct timespec *abstime)
{
    int ret;

    if (task->interrupted) {
        errno = EINTR;
        return -1;
    }
    task->wc++;
    tasks_add(task);
    if (abstime) {
        ret = pthread_cond_timedwait(&task->cond, lock, abstime);
    } else {
        ret = pthread_cond_wait(&task->cond, lock);
    }
    tasks_del(task);
    task->wc--;
    if (task->interrupted) {
        if (!task->wc) {
            task->interrupted = 0;
        }
        errno = EINTR;
        return -1;
    }
    return ret;
}

int
sched_task_wakeup(struct sched_task *task)
{
    return pthread_cond_broadcast(&task->cond);
}

static void
sched_irq_handler(unsigned int irq, void *arg)
{
    struct sched_task *task;

    (void)irq;
    (void)arg;
    lock_acquire(&lock);
    for (task = tasks; task; task = task->next) {
        if (!task->interrupted) {
            task->interrupted = 1;
            pthread_cond_broadcast(&task->cond);
        }
    }
    lock_release(&lock);
}

int
sched_init(void)
{
    return intr_register(INTR_IRQ_USER, sched_irq_handler, 0, NULL);
}

int
sched_run(void)
{
    /* do nothing */
    return 0;
}

int
sched_shutdown(void)
{
    /* do nothing */
    return 0;
}
