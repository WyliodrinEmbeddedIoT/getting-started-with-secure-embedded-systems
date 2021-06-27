# Base folder definitions
TOCK_USERLAND_BASE_DIR ?= ../../../libtock-c
LIBNAME := drivers
$(LIBNAME)_DIR := $(TOCK_USERLAND_BASE_DIR)/../chapter_8/applications/$(LIBNAME)

# List all C and Assembly files
$(LIBNAME)_SRCS  := $(wildcard $($(LIBNAME)_DIR)/*.c)

override CFLAGS += -I$(TOCK_USERLAND_BASE_DIR)/libtock

include $(TOCK_USERLAND_BASE_DIR)/TockLibrary.mk

