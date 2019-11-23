#!/bin/sh
gcc $SOURCE_FILE -o $EXECUTABLE_FILE -O2 -fno-asm -Wall -lm -static -std=c99