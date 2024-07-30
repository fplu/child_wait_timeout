#include <limits.h>
#include <stdint.h>
#include <stdio.h>
#include <stdlib.h>
#include <time.h>
#include <unistd.h>

#include <errno.h>
#include <pthread.h>
#include <sys/types.h>
#include <sys/wait.h>

// Structure to hold the arguments for the wait_for_process function and the
// return value
typedef struct {
  pid_t pid;              // Process ID to wait for
  pid_t return_val;       // Return value of the wait_for_process function
  pthread_cond_t cond;    // Condition variable for notification
  pthread_mutex_t *mutex; // Mutex for condition variable
} proc_info_t;

// The wait_for_process function which waits for the process with the given PID
// to end
void *wait_for_process(void *arg) {
  proc_info_t *proc_info = (proc_info_t *)arg;
  siginfo_t info;

  // Wait for the process without removing it from its zombie state
  if (waitid(P_PID, proc_info->pid, &info, WNOWAIT | WEXITED) == -1) {
    proc_info->return_val = -1;
  } else {
    proc_info->return_val = 0;
  }

  // Notify the waiting thread

  // Locking the mutex: cannot fail if mutex is valid and not already locked by
  // this thread
  (void)pthread_mutex_lock(proc_info->mutex);

  // Signaling the condition variable: cannot fail if cond is valid
  (void)pthread_cond_signal(&proc_info->cond);

  // Unlocking the mutex: cannot fail if mutex was locked by this thread
  (void)pthread_mutex_unlock(proc_info->mutex);

  return NULL;
}

time_t get_max_time_t() {
  return _Generic((time_t)0, signed char
                  : SCHAR_MAX, unsigned char
                  : UCHAR_MAX, signed short
                  : SHRT_MAX, unsigned short
                  : USHRT_MAX, signed int
                  : INT_MAX, unsigned int
                  : UINT_MAX, signed long
                  : LONG_MAX, unsigned long
                  : ULONG_MAX, signed long long
                  : LLONG_MAX, unsigned long long
                  : ULLONG_MAX);
}

// The process_wait_timeout_untraced function
int wait_timeout_untraced_internal_4(proc_info_t *proc_info,
                                     uint32_t timeout_ms) {

  // Wait for the specified timeout or for the wait_for_process function to
  // complete
  struct timespec ts = {0};
  if (clock_gettime(CLOCK_MONOTONIC, &ts) != 0) {
    return -1;
  }

  time_t seconds = (time_t)(timeout_ms / 1000);
  // this one cannot overflow
  long nano_seconds = ts.tv_nsec + ((long)(timeout_ms % 1000) * (long)1000000);

  // this one cannot overflow too
  seconds = seconds + (nano_seconds / 1000000000);
  nano_seconds = nano_seconds % 1000000000;

  // the next one can overflow, we have to check
  if (get_max_time_t() - seconds < ts.tv_sec) {
    return -1;
  }

  ts.tv_sec += seconds;
  ts.tv_nsec = nano_seconds;

  int ret = pthread_cond_timedwait(&proc_info->cond, proc_info->mutex, &ts);

  // Return the result of the wait_for_process function
  return ret;
}

// The process_wait_timeout_untraced function
pid_t wait_timeout_untraced_internal_3(proc_info_t *proc_info,
                                       uint32_t timeout_ms) {
  pthread_t thread;

  // Create the thread to run wait_for_process
  if (pthread_create(&thread, NULL, wait_for_process, proc_info) != 0) {
    return -1;
  }

  // Locking the mutex: cannot fail if mutex is valid and not already locked by
  // this thread
  (void)pthread_mutex_lock(proc_info->mutex);

  int ret = wait_timeout_untraced_internal_4(proc_info, timeout_ms);

  // Unlocking the mutex: cannot fail if mutex was locked by this thread
  (void)pthread_mutex_unlock(proc_info->mutex);

  // If the wait timed out, cancel the thread
  if (ret == ETIMEDOUT) {
    // Canceling the thread: may fail if the thread has already terminated, but
    // this is not an issue in this scenario
    (void)pthread_cancel(thread);
  }

  // Join the thread, the thread is valid and has not be joined yet, it cannot
  // fail
  (void)pthread_join(thread, NULL);

  if (ret == ETIMEDOUT) {
    // set errno in case some error occured, notably in pthread_cancel
    errno = ETIMEDOUT;
    return -1;
  }

  // Return the result of the wait_for_process function
  return proc_info->return_val;
}

pid_t wait_timeout_untraced_internal_2(pid_t pid, uint32_t timeout_ms,
                                       pthread_mutex_t *mutex,
                                       pthread_condattr_t *attr) {

  proc_info_t proc_info;
  proc_info.pid = pid;
  proc_info.return_val = -1;
  proc_info.mutex = mutex;

  if (pthread_condattr_setclock(attr, CLOCK_MONOTONIC) != 0) {
    return -1;
  }

  if (pthread_cond_init(&proc_info.cond, attr) != 0) {
    return -1;
  }

  pid_t result = wait_timeout_untraced_internal_3(&proc_info, timeout_ms);

  (void)pthread_cond_destroy(&proc_info.cond);

  return result;
}

pid_t wait_timeout_untraced_internal_1(pid_t pid, uint32_t timeout_ms,
                                       pthread_mutex_t *mutex) {
  pthread_condattr_t attr;

  if (pthread_condattr_init(&attr) != 0) {
    return -1;
  }

  pid_t result =
      wait_timeout_untraced_internal_2(pid, timeout_ms, mutex, &attr);

  (void)pthread_condattr_destroy(&attr);

  return result;
}

pid_t wait_timeout_untraced(pid_t pid, uint32_t timeout_ms) {
  pthread_mutex_t mutex;

  if (pthread_mutex_init(&mutex, NULL) != 0) {
    return -1;
  }

  pid_t result = wait_timeout_untraced_internal_1(pid, timeout_ms, &mutex);

  (void)pthread_mutex_destroy(&mutex);

  return result;
}