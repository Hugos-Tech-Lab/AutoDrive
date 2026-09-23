#include "the_beginner.h"

__attribute__((export_name("main")))
int main(void) {
    const char text[] = "Hello, World!\n";
    
    // Calculate length (subtract 1 to ignore the null terminator '\0')
    uint32_t len = sizeof(text) - 1;

    while (1) {
        print(text, len);
        delay(1000);
    }

    return 0;
}
