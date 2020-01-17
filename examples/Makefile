CC = gcc
LD = gcc
CFLAGS = -Os -g3
SRCS = $(wildcard *.c)
OBJS = $(SRCS:%.c=%)
TARGET = simple

.PHONY: all
all: $(TARGET)
simple: $(OBJS)
	$(LD) -o $@ $^

%: %.c
	$(CC) $(CFLAGS) -c -o $@ $<

.PHONY: clean
clean:
	rm -f $(TARGET)
