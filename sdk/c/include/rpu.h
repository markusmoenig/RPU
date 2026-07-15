#ifndef RPU_H
#define RPU_H

#include <stdint.h>

#define RPU_IMPORT(module, name) \
    __attribute__((import_module(module), import_name(name)))

int32_t rpu_host_arg_count(void) RPU_IMPORT("rpu_system", "arg_count");
int32_t rpu_host_arg_len(int32_t index) RPU_IMPORT("rpu_system", "arg_len");
int32_t rpu_host_arg_read(int32_t index, char *ptr, int32_t len)
    RPU_IMPORT("rpu_system", "arg_read");
void rpu_host_print(const char *ptr, int32_t len) RPU_IMPORT("rpu_system", "print");
void rpu_host_eprint(const char *ptr, int32_t len) RPU_IMPORT("rpu_system", "eprint");
void rpu_host_exit(int32_t code) RPU_IMPORT("rpu_system", "exit");
int32_t rpu_host_now_ms(void) RPU_IMPORT("rpu_system", "now_ms");

void rpu_host_graphics_begin_frame(int32_t width, int32_t height)
    RPU_IMPORT("rpu_graphics", "begin_frame");
void rpu_host_graphics_clear(float red, float green, float blue, float alpha)
    RPU_IMPORT("rpu_graphics", "clear");
void rpu_host_graphics_draw_rect(float x, float y, float width, float height,
                                 float red, float green, float blue, float alpha)
    RPU_IMPORT("rpu_graphics", "draw_rect");
void rpu_host_graphics_end_frame(void) RPU_IMPORT("rpu_graphics", "end_frame");

int32_t rpu_arg_count(void);
int32_t rpu_arg_len(int32_t index);
int32_t rpu_arg_read(int32_t index, char *buffer, int32_t capacity);
void rpu_print(const char *text);
void rpu_eprint(const char *text);
void rpu_exit(int32_t code);
int32_t rpu_now_ms(void);
void rpu_graphics_begin_frame(int32_t width, int32_t height);
void rpu_graphics_clear(float red, float green, float blue, float alpha);
void rpu_graphics_draw_rect(float x, float y, float width, float height,
                            float red, float green, float blue, float alpha);
void rpu_graphics_end_frame(void);

#if defined(RPU_CARTRIDGE_MODULE)
// A module cartridge implements this entry point.
int32_t rpu_module_main(void);
#else
// A CLI cartridge implements this entry point.
int32_t rpu_main(void);
#endif

#endif
