#ifndef PLATFORM_TIMER_H
#define PLATFORM_TIMER_H

#include <sys/time.h>

extern int
timer_register(struct timeval interval, void (*handler)(void));

extern int
timer_init(void);
extern int
timer_run(void);
extern int
timer_shutdown(void);

#endif
