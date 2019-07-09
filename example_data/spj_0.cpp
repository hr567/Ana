#include <cstdio>
#define AC 0
#define WA 1
using namespace std;
int main(int argc, char *args[]) {
  FILE *f_in = fopen(args[1], "r");
  FILE *f_ans = fopen(args[2], "r");
  FILE *f_out = fopen(args[3], "r");
  int a, b;
  while (fscanf(f_in, "%d%d", &a, &b) != EOF) {
    int output;
    if (fscanf(f_out, "%d", &output) == EOF || output != a + b) {
      return WA;
    }
  }
  return AC;
}
