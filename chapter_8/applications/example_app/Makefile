# Makefile for user application

# Specify this directory relative to the current application.
TOCK_USERLAND_BASE_DIR = ../../../libtock-c

# External libraries used
EXTERN_LIBS += ../drivers

# Which files to compile.
C_SRCS := $(wildcard *.c)

# Include path for drivers library
override CFLAGS += -I../drivers

# Include userland master makefile. Contains rules and flags for actually
# building the application.
include $(TOCK_USERLAND_BASE_DIR)/AppMakefile.mk

# Build the drivers
../drivers/build/cortex-m0/drivers.a:
	make -f ../drivers/Makefile

# Clean drivers folder
clean::
	rm -rf ../drivers/build
