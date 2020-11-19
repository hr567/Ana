#include <stdio.h>
#define AC 0
#define WA 1
int main(int argc, char *args[]) {
  FILE *f_in = fopen(args[1], "r");
  FILE *f_out = fopen(args[2], "r");
  FILE *f_ans = fopen(args[3], "r");
  int a, b;
  while (fscanf(f_in, "%d%d", &a, &b) != EOF) {
    int output;
    if (fscanf(f_out, "%d", &output) == EOF || output != a + b) {
      return WA;
    }
  }
  return AC;
}
