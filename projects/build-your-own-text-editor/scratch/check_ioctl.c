#include <stdio.h>
#include <unistd.h>
#include <sys/ioctl.h>

struct editorConfig {
  int screenrows;
  int screencols;
};

struct editorConfig E;

int getWindowSize(int *rows, int *cols) {
  struct winsize ws;
  printf("sizeof winsize %lu\n", sizeof(ws));
  if (ioctl(STDOUT_FILENO, TIOCGWINSZ, &ws) == -1 || ws.ws_col == 0) {
    return -1;
  } else {
    *cols = ws.ws_col;
    *rows = ws.ws_row;
    return 0;
  }
}

// $ cc check_ioctl.c -o check_ioctl
// $ ./check_ioctl

int main() {
    printf("TIOCGWINSZ = %#lx\n", (unsigned long)TIOCGWINSZ);
    if (getWindowSize(&E.screenrows, &E.screencols) == -1) return 1;
    printf("%d, %d\n", E.screenrows, E.screencols);
    
    return 0;
}
