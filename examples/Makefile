CC = gcc
CFLAGS = -Os -g3
SRCS = $(wildcard *.c)
TARGET = $(SRCS:%.c=%)

.PHONY: all
all: $(TARGET)

%: %.c
	$(CC) $(CFLAGS) -o $@ $<

.PHONY: clean
clean:
	rm -f $(TARGET)
