#ifndef PLATFORM_INTR_H
#define PLATFORM_INTR_H

#include <signal.h>

#define INTR_IRQ_SOFT  SIGUSR1
#define INTR_IRQ_USER  SIGUSR2
#define INTR_IRQ_TIMER SIGALRM

#define INTR_IRQ_BASE (SIGRTMIN+1)

#define INTR_IRQ_SHARED 0x0001

typedef void (*intr_isr_t)(unsigned int irq, void *arg);

extern int
intr_register(unsigned int irq, intr_isr_t isr, int flags, void *arg);
extern int
intr_raise(unsigned int irq);

extern int
intr_init(void);
extern int
intr_run(void);
extern int
intr_shutdown(void);

#endif
