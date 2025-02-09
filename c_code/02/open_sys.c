#include <fcntl.h>
#include <stdio.h>
#include <errno.h>
#include <unistd.h>
#include <stdlib.h>
#include <string.h>

int main(int argc, char* argv[]) {
	int fd;
	mode_t mode = 0644;

	if ((fd = open("log.txt", O_CREAT | O_TRUNC | O_WRONLY, mode)) < 0) {
		fprintf(stderr, "Error opening file: %s\n", strerror(errno));
		exit(1);
	};

	for (int i = 0; i < argc; i++) {
		char buf[100];  // create a buffer to hold the string
		snprintf(buf, sizeof(buf), "argv[%d]: %s\n", i, argv[i]);
		write(fd, buf, strlen(buf));
	}

	close(fd);
	return 0;
}
