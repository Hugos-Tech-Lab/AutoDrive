CC = clang

# Changed the include path to point to the library folder
CFLAGS = --target=wasm32 -nostdlib -O3 -Wall -I../../library

# Linker flags
LDFLAGS = -Wl,--no-entry -Wl,--allow-undefined -Wl,--strip-all

TARGET = hello_world.wasm
SRC = main.c
# Changed the library path to match where your .a file is
LIB = ../../library/libthe_beginner.a

all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) $(LDFLAGS) $(SRC) $(LIB) -o $@

clean:
	rm -f $(TARGET)

.PHONY: all clean