#!/bin/bash

if [ -z "$1" ]; then
	echo "No file supplied"
else
	gdb-multiarch -q $1 -ex "target remote :3333" -ex "monitor arm semihosting enable" -ex "load" -ex "continue"
fi
