#include "rpu.h"

#define RPU_EXPORT(name) __attribute__((export_name(name), used))

extern unsigned char __heap_base;
static uintptr_t rpu_heap_next;

static int32_t rpu_string_len(const char *text) {
    int32_t length = 0;
    if (text == 0) {
        return 0;
    }
    while (text[length] != '\0') {
        length += 1;
    }
    return length;
}

RPU_EXPORT("rpu_abi_version")
int32_t rpu_abi_version(void) {
    return 1;
}

RPU_EXPORT("rpu_alloc")
int32_t rpu_alloc(int32_t length, int32_t alignment) {
    if (length <= 0) {
        return 0;
    }
    if (rpu_heap_next == 0) {
        rpu_heap_next = (uintptr_t)&__heap_base;
    }
    if (alignment <= 0) {
        alignment = 1;
    }
    uintptr_t mask = (uintptr_t)alignment - 1;
    uintptr_t aligned = (rpu_heap_next + mask) & ~mask;
    rpu_heap_next = aligned + (uintptr_t)length;
    return (int32_t)aligned;
}

RPU_EXPORT("rpu_dealloc")
void rpu_dealloc(int32_t ptr, int32_t length, int32_t alignment) {
    (void)ptr;
    (void)length;
    (void)alignment;
}

#if defined(RPU_CARTRIDGE_MODULE)
RPU_EXPORT("rpu_module_init")
int32_t rpu_module_init(void) {
    return rpu_module_main();
}
#else
RPU_EXPORT("rpu_run")
int32_t rpu_run(void) {
    return rpu_main();
}
#endif

int32_t rpu_arg_count(void) {
    return rpu_host_arg_count();
}

int32_t rpu_arg_len(int32_t index) {
    return rpu_host_arg_len(index);
}

int32_t rpu_arg_read(int32_t index, char *buffer, int32_t capacity) {
    if (buffer == 0 || capacity <= 0) {
        return 0;
    }
    int32_t count = rpu_host_arg_read(index, buffer, capacity - 1);
    if (count < 0) {
        count = 0;
    }
    buffer[count] = '\0';
    return count;
}

void rpu_print(const char *text) {
    rpu_host_print(text, rpu_string_len(text));
}

void rpu_eprint(const char *text) {
    rpu_host_eprint(text, rpu_string_len(text));
}

void rpu_exit(int32_t code) {
    rpu_host_exit(code);
}

int32_t rpu_now_ms(void) {
    return rpu_host_now_ms();
}

void rpu_graphics_begin_frame(int32_t width, int32_t height) {
    rpu_host_graphics_begin_frame(width, height);
}

void rpu_graphics_clear(float red, float green, float blue, float alpha) {
    rpu_host_graphics_clear(red, green, blue, alpha);
}

void rpu_graphics_draw_rect(float x, float y, float width, float height,
                            float red, float green, float blue, float alpha) {
    rpu_host_graphics_draw_rect(x, y, width, height, red, green, blue, alpha);
}

void rpu_graphics_end_frame(void) {
    rpu_host_graphics_end_frame();
}
