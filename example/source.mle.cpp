#include <iostream>
#include <random>
using namespace std;
#define MLE_SIZE (33554432 * 2) // 32 Mb

int A[MLE_SIZE];
int main() {
    for (int i = 0; i != MLE_SIZE; ++i) {
        A[i] = rand();
    }
    int a, b;
    while (cin >> a >> b) {
        cout << a + b << endl;
    }
    return 0;
}
