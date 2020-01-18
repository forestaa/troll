CC = gcc
LD = gcc
CFLAGS = -Os -g3
READELF = eu-readelf
READELFFLAGS = -winfo
SRCS = $(filter-out many-compilation-units-sub.c, $(wildcard *.c))
BINARIES = $(SRCS:%.c=%)
DEBUGS = $(BINARIES:%=%.debug)

.PHONY: all
all: $(BINARIES) $(DEBUGS)

many-compilation-units: many-compilation-units.c many-compilation-units-sub.c
	$(CC) $(CFLAGS) -o $@ $^

%: %.c
	$(CC) $(CFLAGS) -o $@ $^

%.debug: %
	$(READELF) $(READELFFLAGS) $< > $@

.PHONY: clean
clean:
	rm -f $(BINARIES) $(DEBUGS)
