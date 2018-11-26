#define METHOD 1

#include <cstdio>
#define AC 0
#define WA 1
using namespace std;
int main(int argc, char *args[]) {
  FILE *f_in = fopen(args[1], "r");
  FILE *f_out = fopen(args[2], "r");
  FILE *f_user = fopen(args[3], "r");

#ifndef METHOD
  return -1;
#else
#if METHOD == 1
  int ans = 0;
  while (fscanf(f_user, "%d", &ans) != EOF) {
    int a, b;
    if (fscanf(f_in, "%d%d", &a, &b) != EOF) {
      if (a + b != ans) {
        return WA;
      }
    }
  }
#elif METHOD == 2
  int ans = 0;
  while (fscanf(f_user, "%d", &ans) != EOF) {
    int std_ans;
    if (fscanf(f_in, "%d", &std_ans) != EOF) {
      if (ans != std_ans) {
        return WA;
      }
    }
  }
#endif
  return AC;
#endif
}
