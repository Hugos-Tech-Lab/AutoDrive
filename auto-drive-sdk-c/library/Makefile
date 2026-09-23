CC = clang
AR = ar
CFLAGS = --target=wasm32 -nostdlib -O3 -Wall

LIB = libthe_beginner.a
SRC = the_beginner.c
OBJ = the_beginner.o

all: $(LIB)

$(OBJ): $(SRC)
# -c compiles to an object file (.o) without invoking the linker
	$(CC) $(CFLAGS) -c $< -o $@

$(LIB): $(OBJ)
# llvm-ar bundles the object file(s) into a static library (.a)
	$(AR) rcs $@ $^

clean:
	rm -f $(OBJ) $(LIB)

.PHONY: all clean
