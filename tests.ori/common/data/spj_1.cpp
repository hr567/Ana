#include <cstdio>
#define AC 0
#define WA 1
using namespace std;
int main(int argc, char *args[]) {
  FILE *f_in = fopen(args[1], "r");
  FILE *f_ans = fopen(args[2], "r");
  FILE *f_out = fopen(args[3], "r");
  int answer;
  while (fscanf(f_ans, "%d", &answer) != EOF) {
    int output;
    if (fscanf(f_out, "%d", &output) == EOF || output != answer) {
      return WA;
    }
  }
  return AC;
}
