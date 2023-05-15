EXE = Svart

ifeq ($(OS),Windows_NT)
	NAME := $(EXE).exe
else
	NAME := $(EXE)
endif

rule:
	cargo rustc --release -p svart -- -C target-cpu=native --emit link=$(NAME)