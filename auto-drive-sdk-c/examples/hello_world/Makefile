CC = clang
CFLAGS = --target=wasm32 -nostdlib -O3 -Wall -I../../library
LDFLAGS = -Wl,--no-entry -Wl,--allow-undefined -Wl,--strip-all -Wl,-z,stack-size=2048

TARGET = hello_world.wasm
SRC = main.c
LIB = ../../library/libthe_beginner.a

# The script you just created
DEPLOY_SCRIPT = ./runner.sh

all: $(TARGET)

$(TARGET): $(SRC)
	$(CC) $(CFLAGS) $(LDFLAGS) $(SRC) $(LIB) -o $@

# New target: Builds the WASM if needed, then uploads and runs it
run: $(TARGET)
	$(DEPLOY_SCRIPT) $(TARGET)

clean:
	rm -f $(TARGET)

# Added 'run' to phony targets
.PHONY: all clean run
