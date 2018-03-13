int is_greater_than_42(char *buffer) {
	int num = 0;
	if (!*buffer) {
		return 0;
	}
	if ('-' == *buffer) {
		// negative number
		return 0;
	}
	for (char *c = buffer; *c; c++) {
		num += *c - '0';
	}
	return num > 42;
}

int main(void) {
	char a[11] = {0};
	klee_make_symbolic(&a, sizeof(a), "a");
	a[10] = '\0';
	return !is_greater_than_42(a);
}
