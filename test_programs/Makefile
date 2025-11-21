# Makefile for MyOS test programs

CC = gcc
LD = ld
CFLAGS = -m64 -nostdlib -fno-builtin -fno-stack-protector -fno-pic -mno-red-zone
LDFLAGS = -m elf_x86_64 -nostdlib -static

all: hello

hello: hello.o
	$(LD) $(LDFLAGS) -o hello hello.o -Ttext=0x400000

hello.o: hello.c
	$(CC) $(CFLAGS) -c hello.c -o hello.o

clean:
	rm -f *.o hello

.PHONY: all clean
