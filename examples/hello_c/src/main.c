#include <rpu.h>

int32_t rpu_main(void) {
    rpu_print("Hello from a C cartridge");

    if (rpu_arg_count() > 0) {
        char first_arg[256];
        rpu_arg_read(0, first_arg, sizeof(first_arg));
        rpu_print("First argument:");
        rpu_print(first_arg);
    } else {
        rpu_print("Pass an argument after -- to echo it");
    }

    return 0;
}
