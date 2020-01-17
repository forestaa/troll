int sub1(int i, char c) {
  return i;
}

int (*sub2)(int, char) = sub1;

int main() {
  return 0;
}
