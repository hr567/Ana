#include <cstdio>
#define AC 0
#define WA 1
using namespace std;
int main(int argc, char *args[]) {
  FILE *f_in = fopen(args[1], "r");
  FILE *f_out = fopen(args[2], "r");
  FILE *f_user = fopen(args[3], "r");
  int std_ans;
  while (fscanf(f_out, "%d", &std_ans) != EOF) {
    int ans;
    if (fscanf(f_user, "%d", &ans) == EOF || ans != std_ans) {
      return WA;
    }
  }
  return AC;
}
